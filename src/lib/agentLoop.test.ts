import { beforeEach, describe, expect, it, vi } from "vitest";

const { chatCompletionTurn, executeAgentTool, invoke } = vi.hoisted(() => ({
  chatCompletionTurn: vi.fn(),
  executeAgentTool: vi.fn(),
  invoke: vi.fn(),
}));

vi.mock("@tauri-apps/api/core", () => ({ invoke }));
vi.mock("./llm", () => ({ chatCompletionTurn }));
vi.mock("./agentRuntime", () => ({
  executeAgentTool,
  shellResultFromToolData: (data: Record<string, unknown>) =>
    typeof data.state === "string" ? data : null,
}));

import { runAgentLoop } from "./agentLoop";

const baseParams = {
  provider: "cloud" as const,
  model: "deepseek-chat",
  systemContext: "test context",
  messages: [{ role: "user" as const, content: "run echo hi" }],
  enableTools: true,
  callbacks: {
    requestHitlApproval: vi.fn().mockResolvedValue(null),
    requestPathApproval: vi.fn().mockResolvedValue(null),
  },
};

describe("runAgentLoop", () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it("returns assistant text when LLM emits no tool calls", async () => {
    chatCompletionTurn.mockResolvedValueOnce({
      content: "Hello from cloud.",
      toolCalls: [],
    });

    const result = await runAgentLoop(baseParams);

    expect(result.finalText).toBe("Hello from cloud.");
    expect(result.actions).toHaveLength(0);
    expect(chatCompletionTurn).toHaveBeenCalledTimes(1);
  });

  it("executes shell_run and completes on second LLM turn", async () => {
    chatCompletionTurn
      .mockResolvedValueOnce({
        content: "",
        toolCalls: [
          {
            id: "call-1",
            name: "shell_run",
            arguments: JSON.stringify({ command: "echo hi" }),
          },
        ],
      })
      .mockResolvedValueOnce({
        content: "Command finished.",
        toolCalls: [],
      });

    invoke.mockImplementation((cmd: string) => {
      if (cmd === "validate_shell_command") return Promise.resolve(true);
      if (cmd === "check_command_safety") {
        return Promise.resolve({
          is_safe: true,
          requires_hitl_approval: false,
          requires_admin: false,
        });
      }
      return Promise.reject(new Error(`unexpected invoke: ${cmd}`));
    });

    executeAgentTool.mockResolvedValueOnce({
      tool: "shell_run",
      success: true,
      data: {
        success: true,
        stdout: "hi\n",
        stderr: "",
        state: "completed",
        status_code: 0,
      },
    });

    const result = await runAgentLoop(baseParams);

    expect(result.finalText).toBe("Command finished.");
    expect(result.actions).toHaveLength(1);
    expect(result.actions[0].commandExecuted).toBe("echo hi");
    expect(result.actions[0].commandResult?.stdout).toBe("hi\n");
    expect(executeAgentTool).toHaveBeenCalledWith(
      "shell_run",
      { command: "echo hi" },
      expect.objectContaining({ cwd: undefined })
    );
    expect(chatCompletionTurn).toHaveBeenCalledTimes(2);
  });

  it("blocks shell_run when HITL approval is denied", async () => {
    chatCompletionTurn
      .mockResolvedValueOnce({
        content: "",
        toolCalls: [
          {
            id: "call-2",
            name: "shell_run",
            arguments: JSON.stringify({ command: "rm -rf /tmp/test" }),
          },
        ],
      })
      .mockResolvedValueOnce({
        content: "Stopped after blocked command.",
        toolCalls: [],
      });

    invoke.mockImplementation((cmd: string) => {
      if (cmd === "validate_shell_command") return Promise.resolve(true);
      if (cmd === "check_command_safety") {
        return Promise.resolve({
          is_safe: true,
          requires_hitl_approval: true,
          requires_admin: false,
          danger_reason: "Destructive file deletion detected.",
        });
      }
      return Promise.reject(new Error(`unexpected invoke: ${cmd}`));
    });

    const requestHitlApproval = vi.fn().mockResolvedValue(null);

    const result = await runAgentLoop({
      ...baseParams,
      callbacks: {
        ...baseParams.callbacks,
        requestHitlApproval,
      },
    });

    expect(requestHitlApproval).toHaveBeenCalled();
    expect(result.actions[0].label).toBe("Command blocked by user");
    expect(executeAgentTool).not.toHaveBeenCalled();
    expect(result.finalText).toBe("Stopped after blocked command.");
  });
});
