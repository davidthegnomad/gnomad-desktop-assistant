import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";

export type ShellRunState =
  | "completed"
  | "failed"
  | "timeout"
  | "stalled"
  | "incomplete";

export interface ShellRunResult {
  state: ShellRunState;
  success: boolean;
  stdout: string;
  stderr: string;
  status_code: number;
  cwd: string;
  duration_ms: number;
  saw_output: boolean;
  message: string;
}

export interface ShellSessionStatus {
  active: boolean;
  shell: string;
  cwd: string;
}

export interface ShellRunProgress {
  command: string;
  phase: "started" | "running" | "completed" | "failed" | "timeout" | "stalled" | "incomplete";
  exit_code?: number;
  message?: string;
}

/** Human + LLM-readable report from a finished command. */
export function formatShellResultForAgent(res: ShellRunResult, command: string): string {
  const output = (res.stdout || res.stderr || "(no output)").trim();
  return [
    `command: ${command}`,
    `state: ${res.state}`,
    `success: ${res.success}`,
    `exit_code: ${res.status_code}`,
    `duration_ms: ${res.duration_ms}`,
    `saw_terminal_output: ${res.saw_output}`,
    `message: ${res.message}`,
    "--- terminal output ---",
    output,
  ].join("\n");
}

export async function runShellSessionCommand(
  command: string,
  options?: {
    cwd?: string;
    timeoutMs?: number;
    stallMs?: number;
    /** @deprecated Use approvalToken — unsigned boolean bypass is rejected by the backend */
    hitlApproved?: boolean;
    approvalToken?: string;
  }
): Promise<ShellRunResult> {
  return invoke<ShellRunResult>("shell_session_run", {
    command,
    cwd: options?.cwd ?? null,
    timeoutMs: options?.timeoutMs ?? 120_000,
    stallMs: options?.stallMs ?? 45_000,
    hitlApproved: options?.hitlApproved ?? null,
    approvalToken: options?.approvalToken ?? null,
  });
}

export async function validateShellCommand(command: string): Promise<boolean> {
  return invoke<boolean>("validate_shell_command", { command });
}

export async function interruptShellSession(): Promise<void> {
  await invoke("shell_session_interrupt");
}

export async function getShellSessionStatus(): Promise<ShellSessionStatus> {
  return invoke<ShellSessionStatus>("shell_session_status");
}

export async function resetShellSession(): Promise<void> {
  await invoke("shell_session_reset");
}

export function subscribeShellOutput(
  onChunk: (chunk: string) => void
): Promise<() => void> {
  return listen<{ chunk: string }>("shell-output", (event) => {
    if (event.payload.chunk) {
      onChunk(event.payload.chunk);
    }
  }).then((unlisten) => unlisten);
}

export function subscribeShellRunProgress(
  onProgress: (event: ShellRunProgress) => void
): Promise<() => void> {
  return listen<ShellRunProgress>("shell-run-progress", (event) => {
    onProgress(event.payload);
  }).then((unlisten) => unlisten);
}
