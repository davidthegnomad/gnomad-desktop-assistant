import {
  Settings as SettingsIcon,
  BookOpen,
  Sun,
  Moon,
  Monitor,
} from "lucide-react";
import { WindowModeBar } from "./WindowModeBar";
import { TitleBarLeading } from "./TitleBarLeading";
import { APP_NAME } from "../lib/brand";
import type { WindowDisplayMode } from "../lib/windowMode";
import type { ThemeMode } from "../lib/preferences";
import type { PlatformOs } from "../lib/platform";

interface AppToolbarProps {
  platform: PlatformOs;
  windowMode: WindowDisplayMode;
  panelModeTitle?: string;
  onWindowModeChange: (mode: WindowDisplayMode) => void;
  themeMode: ThemeMode;
  onCycleTheme: () => void;
  knowledgeOpen: boolean;
  onToggleKnowledge: () => void;
  settingsOpen: boolean;
  onToggleSettings: () => void;
}

/** Cursor-style title bar: centered draggable title, tools on the right. */
export function AppToolbar({
  platform,
  windowMode,
  panelModeTitle,
  onWindowModeChange,
  themeMode,
  onCycleTheme,
  knowledgeOpen,
  onToggleKnowledge,
  settingsOpen,
  onToggleSettings,
}: AppToolbarProps) {
  const themeIcon =
    themeMode === "dark" ? (
      <Moon size={16} />
    ) : themeMode === "light" ? (
      <Sun size={16} />
    ) : (
      <Monitor size={16} />
    );

  return (
    <header className="app-titlebar" aria-label="Window toolbar">
      <TitleBarLeading platform={platform} />
      <div className="titlebar-drag-zone" data-tauri-drag-region>
        <span className="titlebar-title">{APP_NAME}</span>
      </div>

      <div className="titlebar-trailing">
        <WindowModeBar
          mode={windowMode}
          onModeChange={onWindowModeChange}
          compact
          panelModeTitle={panelModeTitle}
        />
        <div className="header-actions">
          <button
            type="button"
            className="icon-btn"
            onClick={onCycleTheme}
            title={`Theme: ${themeMode}`}
            aria-label="Toggle theme"
          >
            {themeIcon}
          </button>
          <button
            type="button"
            className={`icon-btn ${knowledgeOpen ? "active" : ""}`}
            onClick={onToggleKnowledge}
            title="Knowledge & skills"
            aria-label="Knowledge and skills"
          >
            <BookOpen size={16} />
          </button>
          <button
            type="button"
            className={`icon-btn ${settingsOpen ? "active" : ""}`}
            onClick={onToggleSettings}
            title="Settings"
            aria-label="Settings"
          >
            <SettingsIcon size={16} />
          </button>
        </div>
      </div>
    </header>
  );
}
