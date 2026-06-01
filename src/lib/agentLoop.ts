import { invoke } from "@tauri-apps/api/core";
import { chatCompletionTurn, type LlmChatMessage } from "./llm";
import { executeAgentTool, shellResultFromToolData, type AgentToolCall } from "./agentRuntime";
import type { AgentSettings } from "./agentSettings";
import { tryPlanInvalidCommand } from "./commandPlanner";
import type { ProviderMode } from "./preferences";
import type { AgentErrorPayload } from "./errors";
import { executionFailedLabel, parseInvokeError } from "./errors";
import type { ShellRunState } from "./shellSession";

export const AGENT_SYSTEM_TOOLS = `
## Agent tools
You can call tools to run commands and manage files. Use tools instead of describing actions.
- shell_run: run a one-line shell command
- workspace_info: get workspace root and shell cwd
- fs_list, fs_read, fs_write, fs_search: file operations relative to workspace
After tools run, summarize results for the user. Do not claim success without tool output.
`.trim();

const MAX_AGENT_STEPS = 10;

export interface AgentActionRecord {
  tool: string;
  label: string;
  commandExecuted?: string;
  commandResult?: {
    success: boolean;
    stdout: string;
    stderr: string;
    status_code?: number;
    cwd?: string;
    state?: ShellRunState;
    message?: string;
    duration_ms?: number;
  };
  toolData?: Record<string, unknown>;
  errorPayload?: AgentErrorPayload;
}

export interface AgentLoopResult {
  finalText: string;
  actions: AgentActionRecord[];
}

export type SafetyCheck = {
  is_safe: boolean;
  requires_hitl_approval: boolean;
  requires_admin: boolean;
  danger_reason?: string;
};

export interface AgentLoopCallbacks {
  onStep?: (step: number, label: string) => void;
  /** Resolves with signed approval token on approve, null on deny. */
  requestHitlApproval: (command: string, reason: string) => Promise<string | null>;
  requestPathApproval: (path: string, reason: string) => Promise<boolean>;
  executeElevated?: (command: string, approvalToken: string) => Promise<string>;
}

export async function runAgentLoop(params: {
  provider: ProviderMode;
  model: string;
  ollamaUrl?: string;
  systemContext: string;
  messages: LlmChatMessage[];
  shellCwd?: string;
  enableTools: boolean;
  agentSettings?: AgentSettings;
  chatLocalModel?: string;
  callbacks: AgentLoopCallbacks;
}): Promise<AgentLoopResult> {
  const apiMessages: LlmChatMessage[] = [...params.messages];
  const actions: AgentActionRecord[] = [];

  for (let step = 0; step < MAX_AGENT_STEPS; step++) {
    params.callbacks.onStep?.(step + 1, "Thinking…");
    const turn = await chatCompletionTurn({
      provider: params.provider,
      model: params.model,
      messages: apiMessages,
      ollamaUrl: params.ollamaUrl,
      systemContext: params.systemContext,
      enableTools: params.enableTools,
    });

    if (!turn.toolCalls?.length) {
      return {
        finalText: turn.content?.trim() || "Done.",
        actions,
      };
    }

    apiMessages.push({
      role: "assistant",
      content: turn.content ?? "",
      toolCalls: turn.toolCalls,
    });

    for (const tc of turn.toolCalls) {
      const record = await runOneTool(tc, {
        ...params,
        agentSettings: params.agentSettings,
        ollamaUrl: params.ollamaUrl,
        chatLocalModel: params.chatLocalModel,
      }, actions.length);
      actions.push(record);
      apiMessages.push({
        role: "tool",
        toolCallId: tc.id,
        content: JSON.stringify(record.toolData ?? { error: record.label }),
      });
    }
  }

  return {
    finalText: "Reached the maximum number of agent steps. Review the actions above.",
    actions,
  };
}

async function runOneTool(
  tc: AgentToolCall,
  params: {
    shellCwd?: string;
    callbacks: AgentLoopCallbacks;
    agentSettings?: AgentSettings;
    ollamaUrl?: string;
    chatLocalModel?: string;
  },
  index: number
): Promise<AgentActionRecord> {
  let args: Record<string, unknown> = {};
  try {
    args = JSON.parse(tc.arguments || "{}") as Record<string, unknown>;
  } catch {
    return { tool: tc.name, label: `Invalid tool arguments for ${tc.name}` };
  }

  params.callbacks.onStep?.(index + 1, `${tc.name}…`);

  if (tc.name === "shell_run") {
    let command = String(args.command ?? "").trim();
    if (!command) {
      return { tool: tc.name, label: "Empty shell command" };
    }
    let valid = await invoke<boolean>("validate_shell_command", { command });
    if (!valid && params.agentSettings) {
      const planned = await tryPlanInvalidCommand(
        command,
        params.agentSettings,
        params.ollamaUrl ?? "http://localhost:11434",
        params.chatLocalModel
      );
      if (planned) {
        command = planned;
        valid = await invoke<boolean>("validate_shell_command", { command });
      }
    }
    if (!valid) {
      const hint = params.agentSettings?.commandPlannerEnabled
        ? `Invalid shell command (planner could not fix): ${command}`
        : `Invalid shell command: ${command}`;
      return { tool: tc.name, label: hint };
    }
    const safety = await invoke<SafetyCheck>("check_command_safety", { command });
    let approvalToken: string | undefined;
    if (safety.requires_hitl_approval) {
      const token = await params.callbacks.requestHitlApproval(
        command,
        safety.danger_reason || "Safety review requested."
      );
      if (!token) {
        return { tool: tc.name, label: "Command blocked by user", commandExecuted: command };
      }
      approvalToken = token;
      if (safety.requires_admin && params.callbacks.executeElevated) {
        try {
          const out = await params.callbacks.executeElevated(command, token);
          return {
            tool: tc.name,
            label: command,
            commandExecuted: command,
            commandResult: {
              success: true,
              stdout: out,
              stderr: "",
              state: "completed",
              message: "Elevated command completed.",
            },
            toolData: { stdout: out, success: true },
          };
        } catch (err) {
          return {
            tool: tc.name,
            label: executionFailedLabel(err),
            commandExecuted: command,
            errorPayload: parseInvokeError(err) ?? undefined,
          };
        }
      }
    }
    try {
      const res = await executeAgentTool(
        "shell_run",
        { command },
        { approvalToken, cwd: params.shellCwd }
      );
      const shell = shellResultFromToolData(res.data);
      return {
        tool: tc.name,
        label: command,
        commandExecuted: command,
        commandResult: shell
          ? {
              success: shell.success,
              stdout: shell.stdout,
              stderr: shell.stderr,
              status_code: shell.status_code,
              cwd: shell.cwd,
              state: shell.state,
              message: shell.message,
              duration_ms: shell.duration_ms,
            }
          : undefined,
        toolData: res.data,
      };
    } catch (err) {
      return {
        tool: tc.name,
        label: executionFailedLabel(err),
        commandExecuted: command,
        errorPayload: parseInvokeError(err) ?? undefined,
      };
    }
  }

  let pathApproved = false;
  const runFs = async (retry: boolean) => {
    try {
      return await executeAgentTool(tc.name, args, {
        pathApproved: retry || pathApproved,
        cwd: params.shellCwd,
      });
    } catch (err) {
      const msg = err instanceof Error ? err.message : String(err);
      if (!retry && msg.includes("outside workspace")) {
        const pathHint = String(args.path ?? args.query ?? "");
        const ok = await params.callbacks.requestPathApproval(
          pathHint,
          msg
        );
        if (ok) {
          pathApproved = true;
          return runFs(true);
        }
      }
      throw err;
    }
  };

  try {
    const res = await runFs(false);
    return {
      tool: tc.name,
      label: `${tc.name} completed`,
      toolData: res.data,
    };
  } catch (err) {
    return {
      tool: tc.name,
      label: executionFailedLabel(err),
      errorPayload: parseInvokeError(err) ?? undefined,
    };
  }
}
