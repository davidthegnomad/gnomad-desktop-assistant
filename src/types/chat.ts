import type { ChatAttachment } from "../lib/attachments";
import type { AgentErrorPayload } from "../lib/errors";
import type { ShellRunState } from "../lib/shellSession";

export interface CommandResult {
  success: boolean;
  stdout: string;
  stderr: string;
  status_code?: number;
  cwd?: string;
  state?: ShellRunState;
  message?: string;
  duration_ms?: number;
}

export interface Message {
  role: "user" | "assistant";
  text: string;
  attachments?: ChatAttachment[];
  commandExecuted?: string;
  commandResult?: CommandResult;
  errorPayload?: AgentErrorPayload;
  shellCwd?: string;
}

export interface SafetyCheck {
  is_safe: boolean;
  requires_hitl_approval: boolean;
  requires_admin: boolean;
  danger_reason: string | null;
}

export const WELCOME_ONLY =
  "Hey — I'm Gnomad 🍄 I watch your active window and clipboard, run safe shell commands, and help you automate. What's on your mind?";
