import { PanelTop, AppWindow, Maximize2, PictureInPicture2 } from "lucide-react";
import type { WindowDisplayMode } from "../lib/windowMode";
import { setWindowMode } from "../lib/windowMode";

interface WindowModeBarProps {
  mode: WindowDisplayMode;
  onModeChange: (mode: WindowDisplayMode) => void;
  /** Icon-only chips for a minimal header (Gemini-style). */
  compact?: boolean;
}

const MODES: {
  id: WindowDisplayMode;
  label: string;
  icon: React.ReactNode;
  title: string;
}[] = [
  {
    id: "panel",
    label: "Panel",
    icon: <PanelTop size={14} />,
    title: "Drop down from menu bar",
  },
  {
    id: "floating",
    label: "Pop out",
    icon: <PictureInPicture2 size={14} />,
    title: "Movable floating window",
  },
  {
    id: "windowed",
    label: "Window",
    icon: <AppWindow size={14} />,
    title: "Standard resizable window",
  },
  {
    id: "fullscreen",
    label: "Full",
    icon: <Maximize2 size={14} />,
    title: "Fullscreen",
  },
];

export function WindowModeBar({ mode, onModeChange, compact = false }: WindowModeBarProps) {
  return (
    <div
      className={`window-mode-bar ${compact ? "window-mode-bar-compact" : ""}`}
      role="toolbar"
      aria-label="Window layout"
    >
      {MODES.map((m) => (
        <button
          key={m.id}
          type="button"
          className={`mode-chip ${mode === m.id ? "active" : ""}`}
          title={compact ? `${m.label} — ${m.title}` : m.title}
          aria-label={m.label}
          onClick={async () => {
            await setWindowMode(m.id);
            onModeChange(m.id);
          }}
        >
          {m.icon}
          {!compact && <span>{m.label}</span>}
        </button>
      ))}
    </div>
  );
}
