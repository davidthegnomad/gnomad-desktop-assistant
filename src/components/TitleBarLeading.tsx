import { getCurrentWindow } from "@tauri-apps/api/window";
import { Minus, Square, X } from "lucide-react";
import type { PlatformOs } from "../lib/platform";

interface TitleBarLeadingProps {
  platform: PlatformOs;
}

/** macOS: empty spacer for native traffic lights. Win/Linux: window controls. */
export function TitleBarLeading({ platform }: TitleBarLeadingProps) {
  if (platform === "macos") {
    return <div className="titlebar-leading titlebar-leading-macos" aria-hidden />;
  }

  // Linux uses native KDE/GTK window controls — duplicate buttons crash/conflict with Wayland.
  if (platform === "linux") {
    return null;
  }

  if (platform !== "windows") {
    return null;
  }

  const win = getCurrentWindow();

  return (
    <div className="titlebar-leading titlebar-leading-controls" role="group" aria-label="Window">
      <button
        type="button"
        className="titlebar-window-btn"
        title="Minimize"
        aria-label="Minimize"
        onClick={() => void win.minimize()}
      >
        <Minus size={14} />
      </button>
      <button
        type="button"
        className="titlebar-window-btn"
        title="Maximize"
        aria-label="Maximize"
        onClick={() => void win.toggleMaximize()}
      >
        <Square size={12} />
      </button>
      <button
        type="button"
        className="titlebar-window-btn titlebar-window-btn-close"
        title="Hide to system tray"
        aria-label="Hide to system tray"
        onClick={() => void win.hide()}
      >
        <X size={14} />
      </button>
    </div>
  );
}
