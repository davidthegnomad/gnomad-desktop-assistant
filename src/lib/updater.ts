import { invoke } from "@tauri-apps/api/core";

export type UpdateChannel = "stable" | "beta";

export interface UpdateCheckResult {
  available: boolean;
  currentVersion: string;
  version?: string;
  notes?: string;
  date?: string;
  channel: string;
}

export async function checkForUpdates(
  channel: UpdateChannel = "stable"
): Promise<UpdateCheckResult> {
  return invoke<UpdateCheckResult>("check_for_updates", { channel });
}

export async function installUpdate(channel: UpdateChannel = "stable"): Promise<void> {
  await invoke("install_update", { channel });
}
