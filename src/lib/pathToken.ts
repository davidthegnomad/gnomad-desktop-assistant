import { invoke } from "@tauri-apps/api/core";

export type PathScope = "read" | "write";

export async function issuePathGateToken(
  path: string,
  scope: PathScope
): Promise<string> {
  return invoke<string>("issue_path_gate_token", { path, scope });
}
