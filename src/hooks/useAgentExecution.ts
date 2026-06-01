import { useCallback, useRef, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import type { AgentSettings } from "../lib/agentSettings";
import { tryPlanInvalidCommand } from "../lib/commandPlanner";
import { executionFailedLabel, parseInvokeError } from "../lib/errors";
import { issueHitlApprovalToken, type HitlScope } from "../lib/hitlToken";
import { issuePathGateToken, type PathScope } from "../lib/pathToken";
import { isValidShellCommand } from "../lib/agentShell";
import { runShellSessionCommand, type ShellRunState } from "../lib/shellSession";
import type { Message, SafetyCheck } from "../types/chat";

export function useAgentExecution(options: {
  ollamaUrl: string;
  apiType: "cloud" | "local";
  localModel: string;
  setThinkingText: (t: string) => void;
  shellOutputBufRef: React.MutableRefObject<string>;
}) {
  const [shellCwd, setShellCwd] = useState<string | null>(null);
  const [sudoGateOpen, setSudoGateOpen] = useState(false);
  const [sudoGateCommand, setSudoGateCommand] = useState("");
  const [sudoGateReason, setSudoGateReason] = useState("");
  const [pathGateOpen, setPathGateOpen] = useState(false);
  const [pathGateTarget, setPathGateTarget] = useState("");
  const [pathGateReason, setPathGateReason] = useState("");
  const [pathGateScope, setPathGateScope] = useState<PathScope>("read");

  const sudoResolveRef = useRef<((approved: boolean) => void) | null>(null);

  const mintApprovalToken = useCallback(
    async (command: string, requiresAdmin: boolean): Promise<string> => {
      const scope: HitlScope = requiresAdmin ? "elevated" : "shell_run";
      return issueHitlApprovalToken(command, scope);
    },
    []
  );
  const pathResolveRef = useRef<((token: string | null) => void) | null>(null);

  const resolveSudoGate = useCallback((approved: boolean) => {
    sudoResolveRef.current?.(approved);
    sudoResolveRef.current = null;
    setSudoGateOpen(false);
  }, []);

  const resolvePathGate = useCallback((approved: boolean) => {
    if (!approved) {
      pathResolveRef.current?.(null);
      pathResolveRef.current = null;
      setPathGateOpen(false);
      return;
    }
    const path = pathGateTarget;
    const scope: PathScope = pathGateScope;
    void issuePathGateToken(path, scope)
      .then((token) => pathResolveRef.current?.(token))
      .catch(() => pathResolveRef.current?.(null))
      .finally(() => {
        pathResolveRef.current = null;
        setPathGateOpen(false);
      });
  }, [pathGateTarget, pathGateScope]);

  const requestHitlApproval = useCallback((command: string, reason: string) => {
    return new Promise<string | null>((resolve) => {
      sudoResolveRef.current = async (approved: boolean) => {
        if (!approved) {
          resolve(null);
          return;
        }
        try {
          const safety = await invoke<SafetyCheck>("check_command_safety", { command });
          const token = await mintApprovalToken(command, safety.requires_admin);
          resolve(token);
        } catch {
          resolve(null);
        }
      };
      setSudoGateCommand(command);
      setSudoGateReason(reason);
      setSudoGateOpen(true);
    });
  }, [mintApprovalToken]);

  const requestPathApproval = useCallback((path: string, reason: string, scope: PathScope = "read") => {
    return new Promise<string | null>((resolve) => {
      pathResolveRef.current = resolve;
      setPathGateTarget(path);
      setPathGateReason(reason);
      setPathGateScope(scope);
      setPathGateOpen(true);
    });
  }, []);

  const executeCommandSafely = useCallback(
    async (command: string) => {
      const safety = await invoke<SafetyCheck>("check_command_safety", { command });

      if (safety.requires_hitl_approval) {
        setSudoGateCommand(command);
        setSudoGateReason(safety.danger_reason || "Safety review requested.");
        setSudoGateOpen(true);

        return new Promise<{
          success: boolean;
          stdout: string;
          stderr: string;
          status_code?: number;
          cwd?: string;
          state?: ShellRunState;
          message?: string;
          duration_ms?: number;
        }>((resolve, reject) => {
          sudoResolveRef.current = async (approved: boolean) => {
            if (!approved) {
              reject(new Error("Command blocked by user in Sudo Gate review."));
              return;
            }
            try {
              const token = await mintApprovalToken(command, safety.requires_admin);
              if (safety.requires_admin) {
                const out = await invoke<string>("execute_elevated_command", {
                  command,
                  approvalToken: token,
                });
                resolve({
                  success: true,
                  stdout: out,
                  stderr: "",
                  cwd: shellCwd ?? undefined,
                  state: "completed",
                  message: "Elevated command completed.",
                });
              } else {
                const res = await runShellSessionCommand(command, {
                  cwd: shellCwd ?? undefined,
                  approvalToken: token,
                });
                setShellCwd(res.cwd);
                resolve({
                  success: res.success,
                  stdout: res.stdout,
                  stderr: res.stderr,
                  status_code: res.status_code,
                  cwd: res.cwd,
                  state: res.state,
                  message: res.message,
                  duration_ms: res.duration_ms,
                });
              }
            } catch (err) {
              reject(err);
            }
          };
        });
      }

      options.shellOutputBufRef.current = "";
      const res = await runShellSessionCommand(command, { cwd: shellCwd ?? undefined });
      setShellCwd(res.cwd);
      return {
        success: res.success,
        stdout: res.stdout,
        stderr: res.stderr,
        status_code: res.status_code,
        cwd: res.cwd,
        state: res.state,
        message: res.message,
        duration_ms: res.duration_ms,
      };
    },
    [shellCwd, options.shellOutputBufRef, mintApprovalToken]
  );

  const runShellCommandsInChat = useCallback(
    async (
      commands: string[],
      baseMessages: Message[],
      invalidFromModel: string[] = [],
      agentSettings?: AgentSettings
    ): Promise<Message[]> => {
      let next = [...baseMessages];
      const recoveredCommands: string[] = [];
      for (const bad of invalidFromModel) {
        if (agentSettings) {
          const planned = await tryPlanInvalidCommand(
            bad,
            agentSettings,
            options.ollamaUrl,
            options.apiType === "local" ? options.localModel : undefined
          );
          if (planned) {
            recoveredCommands.push(planned);
            next = [
              ...next,
              {
                role: "assistant",
                text: `Command planner converted intent to: \`${planned}\``,
              },
            ];
            continue;
          }
        }
        next = [
          ...next,
          {
            role: "assistant",
            text: `Skipped invalid shell command (use a real CLI command, not English): "${bad}"`,
          },
        ];
      }
      const allCommands = [...commands, ...recoveredCommands];
      for (const command of allCommands) {
        if (!isValidShellCommand(command)) {
          next = [
            ...next,
            { role: "assistant", text: `Skipped invalid shell command: "${command}"` },
          ];
          continue;
        }
        options.shellOutputBufRef.current = "";
        options.setThinkingText(`Running: ${command}`);
        try {
          const res = await executeCommandSafely(command);
          if (res.cwd) setShellCwd(res.cwd);
          next = [
            ...next,
            {
              role: "assistant",
              text: res.message ?? (res.success ? "Command finished." : "Command failed."),
              commandExecuted: command,
              commandResult: {
                success: res.success,
                stdout: res.stdout,
                stderr: res.stderr,
                status_code: res.status_code,
                cwd: res.cwd,
                state: res.state,
                message: res.message,
                duration_ms: res.duration_ms,
              },
            },
          ];
        } catch (err: unknown) {
          next = [
            ...next,
            {
              role: "assistant",
              text: executionFailedLabel(err),
              commandExecuted: command,
              errorPayload: parseInvokeError(err) ?? undefined,
            },
          ];
        }
      }
      return next;
    },
    [executeCommandSafely, options]
  );

  return {
    shellCwd,
    setShellCwd,
    sudoGateOpen,
    sudoGateCommand,
    sudoGateReason,
    pathGateOpen,
    pathGateTarget,
    pathGateReason,
    resolveSudoGate,
    resolvePathGate,
    requestHitlApproval,
    requestPathApproval,
    executeCommandSafely,
    runShellCommandsInChat,
  };
}
