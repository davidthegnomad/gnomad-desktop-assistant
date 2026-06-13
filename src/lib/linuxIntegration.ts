import { invoke } from "@tauri-apps/api/core";

export interface LinuxIntegrationStatus {
  sessionType: string;
  desktopEnvironment: string;
  activeWindowBackend: string;
  clipboardBackend: string;
  sandboxLevel: string;
  wlPasteAvailable: boolean;
  xdotoolAvailable: boolean;
  qdbusAvailable: boolean;
  bwrapAvailable: boolean;
  trayHint: string;
}

export async function getLinuxIntegrationStatus(): Promise<LinuxIntegrationStatus> {
  return invoke<LinuxIntegrationStatus>("get_linux_integration_status");
}
