import { invoke } from "@tauri-apps/api/core";
import type { ShellRunResult } from "./shellSession";

export interface ToolExecutionResult {
  tool: string;
  success: boolean;
  data: Record<string, unknown>;
  error?: string;
}

export interface AgentToolCall {
  id: string;
  name: string;
  arguments: string;
}

export async function executeAgentTool(
  name: string,
  args: Record<string, unknown>,
  options?: {
    /** @deprecated Use approvalToken */
    hitlApproved?: boolean;
    approvalToken?: string;
    pathApproved?: boolean;
    cwd?: string;
  }
): Promise<ToolExecutionResult> {
  return invoke<ToolExecutionResult>("agent_execute_tool", {
    name,
    arguments: JSON.stringify(args),
    hitlApproved: options?.hitlApproved ?? null,
    approvalToken: options?.approvalToken ?? null,
    pathApproved: options?.pathApproved ?? null,
    cwd: options?.cwd ?? null,
  });
}

export function shellResultFromToolData(data: Record<string, unknown>): ShellRunResult | null {
  if (typeof data.state !== "string") return null;
  return data as unknown as ShellRunResult;
}
