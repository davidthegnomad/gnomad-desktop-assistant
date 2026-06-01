import { invoke } from "@tauri-apps/api/core";

export type PlatformOs = "macos" | "windows" | "linux" | "unknown";

export interface PlatformInfo {
  os: PlatformOs;
  trayRegionLabel: string;
  panelModeMenuLabel: string;
  hideToTrayLabel: string;
  trayTooltip: string;
  usesOverlayTitlebar: boolean;
  hideInAppTitlebarWhenWindowed: boolean;
  supportsActiveWindowContext: boolean;
  supportsClipboardContext: boolean;
  supportsAccessibilitySettings: boolean;
}

export async function getPlatformInfo(): Promise<PlatformInfo> {
  return invoke<PlatformInfo>("get_platform_info");
}
