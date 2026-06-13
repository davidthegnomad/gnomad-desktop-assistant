import {
  Settings as SettingsIcon,
  ShieldAlert,
  Terminal,
  X,
} from "lucide-react";
import { ApiKeysSettings } from "./ApiKeysSettings";
import { AgentAccessSettings } from "./AgentAccessSettings";
import { CloudApiSettings } from "./CloudApiSettings";
import { PrivacySettings } from "./PrivacySettings";
import { UpdateSettings } from "./UpdateSettings";
import { StudioLink } from "./StudioLink";
import { LinuxIntegrationSettings } from "./LinuxIntegrationSettings";
import { APP_NAME, MUSHROOM, VERSION } from "../lib/brand";
import { resetShellSession } from "../lib/shellSession";
import type { PlatformInfo } from "../lib/platform";
import type { LlmAvailability } from "../lib/models";
import type { ProviderMode } from "../lib/preferences";

interface SettingsPanelProps {
  open: boolean;
  onClose: () => void;
  onOpenAbout: () => void;
  onOpenKnowledge: () => void;
  onRerunOnboarding: () => void;
  apiType: ProviderMode;
  headerModelValue: string;
  ollamaUrl: string;
  localModel: string;
  llmAvailability: LlmAvailability;
  shellCwd: string | null;
  onShellReset: () => void;
  onProviderChange: (mode: ProviderMode) => void;
  onModelChange: (model: string) => void;
  onOllamaUrlChange: (url: string) => void;
  onKeysChanged: () => void;
  onAgentSettingsChanged?: () => void;
  platformInfo: PlatformInfo | null;
  accessibilityGranted: boolean;
  onRequestPermissions: () => void;
  onTestElevation: () => void;
  resolvedTheme: string;
}

export function SettingsPanel({
  open,
  onClose,
  onOpenAbout,
  onOpenKnowledge,
  onRerunOnboarding,
  apiType,
  headerModelValue,
  ollamaUrl,
  localModel,
  llmAvailability,
  shellCwd,
  onShellReset,
  onProviderChange,
  onModelChange,
  onOllamaUrlChange,
  onKeysChanged,
  onAgentSettingsChanged,
  platformInfo,
  accessibilityGranted,
  onRequestPermissions,
  onTestElevation,
  resolvedTheme,
}: SettingsPanelProps) {
  if (!open) return null;

  const cloudModelOptions = llmAvailability.cloudModels;

  return (
    <div className="settings-pane">
      <header className="app-header">
        <div className="header-left">
          <SettingsIcon size={16} style={{ color: "var(--color-accent)" }} />
          <span className="app-title">Settings</span>
        </div>
        <button type="button" className="icon-btn" onClick={onClose} aria-label="Close settings">
          <X size={16} />
        </button>
      </header>

      <div className="settings-scroll">
        <div className="settings-section">
          <h4 className="section-title">Model & API</h4>
          <div className="settings-row">
            <span className="settings-label">Provider</span>
            <select
              className="settings-select"
              value={apiType}
              onChange={(e) => onProviderChange(e.target.value as ProviderMode)}
            >
              <option value="cloud" disabled={!llmAvailability.cloudConfigured}>
                Cloud API key
              </option>
              <option value="local" disabled={!llmAvailability.localConfigured}>
                Local Ollama
              </option>
            </select>
          </div>
          {apiType === "cloud" ? (
            <>
              <div className="settings-row">
                <span className="settings-label">Model</span>
                {llmAvailability.cloudUsesCustomEndpoint ? (
                  <input
                    className="settings-input"
                    value={headerModelValue}
                    onChange={(e) => onModelChange(e.target.value)}
                    placeholder="gpt-4o-mini"
                  />
                ) : (
                  <select
                    className="settings-select"
                    value={headerModelValue}
                    onChange={(e) => onModelChange(e.target.value)}
                    disabled={cloudModelOptions.length === 0}
                  >
                    {cloudModelOptions.length > 0 ? (
                      cloudModelOptions.map((m) => (
                        <option key={m.value} value={m.value}>
                          {m.label}
                        </option>
                      ))
                    ) : (
                      <option value="">Add a cloud API key (API keys section)</option>
                    )}
                  </select>
                )}
              </div>
              <button type="button" className="btn secondary" onClick={onRerunOnboarding}>
                Re-run setup wizard
              </button>
            </>
          ) : (
            <>
              <div className="settings-row">
                <span className="settings-label">Ollama URL</span>
                <input
                  className="settings-input"
                  value={ollamaUrl}
                  onChange={(e) => onOllamaUrlChange(e.target.value)}
                />
              </div>
              <div className="settings-row">
                <span className="settings-label">Model</span>
                {llmAvailability.localModels.length > 0 ? (
                  <select
                    className="settings-select"
                    value={headerModelValue}
                    onChange={(e) => onModelChange(e.target.value)}
                  >
                    {llmAvailability.localModels.map((m) => (
                      <option key={m.value} value={m.value}>
                        {m.label}
                      </option>
                    ))}
                  </select>
                ) : (
                  <input
                    className="settings-input"
                    value={localModel}
                    onChange={(e) => onModelChange(e.target.value)}
                    placeholder={
                      llmAvailability.ollamaReachable
                        ? "No chat models installed"
                        : "http://localhost:11434 unreachable"
                    }
                  />
                )}
              </div>
            </>
          )}
        </div>

        <ApiKeysSettings onKeysChanged={onKeysChanged} />
        <CloudApiSettings onChanged={onKeysChanged} />
        <AgentAccessSettings onSettingsChanged={onAgentSettingsChanged} />
        <UpdateSettings />
        <PrivacySettings />

        <div className="settings-section">
          <h4 className="section-title">
            <Terminal size={16} />
            Agent shell
          </h4>
          <p className="knowledge-muted">
            Commands run in a persistent shell session (working directory and environment carry over).
            Output streams into chat — no embedded terminal window.
          </p>
          {shellCwd && (
            <p className="shell-settings-cwd">
              <span className="settings-label">Working directory</span>
              <code>{shellCwd}</code>
            </p>
          )}
          <button
            type="button"
            className="btn secondary"
            onClick={() => void resetShellSession().then(onShellReset)}
          >
            Reset shell session
          </button>
        </div>

        <div className="settings-section">
          <h4 className="section-title">
            <span aria-hidden>🍄</span>
            Knowledge & skills
          </h4>
          <p className="knowledge-muted" style={{ marginBottom: 8 }}>
            Open the <strong>book icon</strong> in the header or use{" "}
            <strong>Knowledge & skills</strong> in the left sidebar to add files.
          </p>
          <button type="button" className="btn secondary" onClick={onOpenKnowledge}>
            Open knowledge library
          </button>
        </div>

        <div className="settings-section">
          <h4 className="section-title">
            <ShieldAlert size={16} style={{ color: "var(--color-purple)" }} />
            Permissions
          </h4>
          {platformInfo?.os === "linux" && <LinuxIntegrationSettings />}
          {platformInfo?.linuxSessionType === "wayland" && (
            <p className="knowledge-muted platform-wayland-hint">
              Wayland detected — <strong>left-click</strong> tray for menu,{" "}
              <strong>right-click</strong> to toggle the panel.
            </p>
          )}
          {platformInfo?.supportsAccessibilitySettings ? (
            <div className="settings-row">
              <span className="settings-label">Accessibility</span>
              <span
                style={{
                  fontSize: "0.8rem",
                  fontWeight: 600,
                  color: accessibilityGranted ? "var(--color-success)" : "var(--color-warning)",
                }}
              >
                {accessibilityGranted ? "Granted" : "Required"}
              </span>
              {!accessibilityGranted && (
                <button type="button" className="btn secondary" onClick={onRequestPermissions}>
                  Open System Settings
                </button>
              )}
            </div>
          ) : (
            <p className="knowledge-muted">
              Accessibility permission is managed by macOS. On{" "}
              {platformInfo?.os === "windows" ? "Windows" : "Linux"}, use your system settings if
              automation tools need extra access.
            </p>
          )}
          {(platformInfo?.os === "macos" || platformInfo?.os === "linux") && (
            <div className="settings-row">
              <span className="settings-label">Test elevation</span>
              <button type="button" className="btn primary" onClick={onTestElevation}>
                Trigger admin prompt
              </button>
            </div>
          )}
          {platformInfo?.os === "windows" && (
            <p className="knowledge-muted">
              Elevated commands on Windows must be approved in an elevated terminal (Run as
              administrator).
            </p>
          )}
        </div>

        <div className="settings-section">
          <h4 className="section-title">
            <span aria-hidden>🍄</span>
            About
          </h4>
          <p className="about-credit" style={{ fontSize: "0.85rem" }}>
            Built with ❤️ by <StudioLink className="about-studio-inline" />
          </p>
          <p style={{ fontSize: "0.8rem", color: "var(--color-muted)" }}>
            {APP_NAME} {MUSHROOM} v{VERSION}
          </p>
          <button type="button" className="btn secondary" onClick={onOpenAbout}>
            About Gnomad
          </button>
        </div>

        <div className="settings-section" style={{ border: "none", background: "transparent" }}>
          <p style={{ fontSize: "0.75rem", color: "var(--color-muted)", textAlign: "center" }}>
            {APP_NAME} v{VERSION} · Theme: {resolvedTheme}
          </p>
        </div>
      </div>
    </div>
  );
}
