import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";

export type WindowDisplayMode = "panel" | "floating" | "windowed" | "fullscreen";

export async function setWindowMode(mode: WindowDisplayMode): Promise<void> {
  await invoke("set_window_mode", { mode, anchorTray: mode === "panel" });
}

export async function fitWindowToContent(
  contentWidth: number,
  contentHeight: number,
  force = false
): Promise<void> {
  await invoke("fit_window_to_content", { contentWidth, contentHeight, force });
}

export async function getWindowMode(): Promise<WindowDisplayMode> {
  const mode = await invoke<string>("get_window_mode");
  if (mode === "floating" || mode === "windowed" || mode === "fullscreen") {
    return mode;
  }
  return "panel";
}

export function subscribeWindowMode(
  onMode: (mode: WindowDisplayMode) => void
): Promise<() => void> {
  return listen<string>("window-mode-changed", (event) => {
    const m = event.payload;
    if (m === "panel" || m === "floating" || m === "windowed" || m === "fullscreen") {
      onMode(m);
    }
  }).then((un) => un);
}
