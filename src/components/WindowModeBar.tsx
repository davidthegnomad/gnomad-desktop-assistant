import { PanelTop, AppWindow, Maximize2, PictureInPicture2 } from "lucide-react";
import type { WindowDisplayMode } from "../lib/windowMode";
import { setWindowMode } from "../lib/windowMode";

interface WindowModeBarProps {
  mode: WindowDisplayMode;
  onModeChange: (mode: WindowDisplayMode) => void;
  /** Icon-only chips for a minimal header (Gemini-style). */
  compact?: boolean;
  panelModeTitle?: string;
}

function buildModes(panelModeTitle: string) {
  return [
  {
    id: "panel" as const,
    label: "Panel",
    icon: <PanelTop size={14} />,
    title: panelModeTitle,
  },
  {
    id: "floating" as const,
    label: "Pop out",
    icon: <PictureInPicture2 size={14} />,
    title: "Movable floating window",
  },
  {
    id: "windowed" as const,
    label: "Window",
    icon: <AppWindow size={14} />,
    title: "Standard resizable window with app menus",
  },
  {
    id: "fullscreen" as const,
    label: "Full",
    icon: <Maximize2 size={14} />,
    title: "Fullscreen",
  },
];
}

export function WindowModeBar({
  mode,
  onModeChange,
  compact = false,
  panelModeTitle = "Drop down from menu bar",
}: WindowModeBarProps) {
  const modes = buildModes(panelModeTitle);
  return (
    <div
      className={`window-mode-bar ${compact ? "window-mode-bar-compact" : ""}`}
      role="toolbar"
      aria-label="Window layout"
    >
      {modes.map((m) => (
        <button
          key={m.id}
          type="button"
          className={`mode-chip ${mode === m.id ? "active" : ""}`}
          title={compact ? `${m.label} — ${m.title}` : m.title}
          aria-label={m.label}
          onClick={async () => {
            try {
              const next =
                mode === m.id && m.id === "fullscreen" ? "floating" : m.id;
              await setWindowMode(next);
              onModeChange(next);
            } catch (err) {
              console.error("Window mode change failed:", err);
            }
          }}
        >
          {m.icon}
          {!compact && <span>{m.label}</span>}
        </button>
      ))}
    </div>
  );
}
