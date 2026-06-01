import { invoke } from "@tauri-apps/api/core";

export type HitlScope = "shell_run" | "elevated";

export async function issueHitlApprovalToken(
  command: string,
  scope: HitlScope
): Promise<string> {
  return invoke<string>("issue_hitl_approval_token", { command, scope });
}
