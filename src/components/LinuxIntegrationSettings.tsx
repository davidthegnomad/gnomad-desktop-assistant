import { useEffect, useState } from "react";
import { Monitor } from "lucide-react";
import {
  getLinuxIntegrationStatus,
  type LinuxIntegrationStatus,
} from "../lib/linuxIntegration";

function statusLabel(value: string): string {
  return value.replace(/-/g, " ");
}

export function LinuxIntegrationSettings() {
  const [status, setStatus] = useState<LinuxIntegrationStatus | null>(null);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    void getLinuxIntegrationStatus()
      .then(setStatus)
      .catch((err: unknown) => {
        setError(err instanceof Error ? err.message : String(err));
      });
  }, []);

  if (error) {
    return (
      <div className="settings-section">
        <h4 className="section-title">
          <Monitor size={16} />
          Linux integration
        </h4>
        <p className="knowledge-muted">Could not read integration status: {error}</p>
      </div>
    );
  }

  if (!status) {
    return (
      <div className="settings-section">
        <h4 className="section-title">
          <Monitor size={16} />
          Linux integration
        </h4>
        <p className="knowledge-muted">Checking session and optional packages…</p>
      </div>
    );
  }

  const missingClipboard = status.clipboardBackend.startsWith("missing");
  const missingWindow = status.activeWindowBackend === "unavailable";

  return (
    <div className="settings-section">
      <h4 className="section-title">
        <Monitor size={16} />
        Linux integration
      </h4>
      <p className="knowledge-muted">{status.trayHint}</p>
      <div className="settings-row">
        <span className="settings-label">Session</span>
        <span>{status.sessionType}</span>
      </div>
      <div className="settings-row">
        <span className="settings-label">Desktop</span>
        <span>{status.desktopEnvironment}</span>
      </div>
      <div className="settings-row">
        <span className="settings-label">Active window</span>
        <span>{statusLabel(status.activeWindowBackend)}</span>
      </div>
      <div className="settings-row">
        <span className="settings-label">Clipboard</span>
        <span>{statusLabel(status.clipboardBackend)}</span>
      </div>
      <div className="settings-row">
        <span className="settings-label">YOLO sandbox</span>
        <span>{status.sandboxLevel === "none" ? "off (install bubblewrap)" : status.sandboxLevel}</span>
      </div>
      {missingClipboard && (
        <p className="knowledge-muted platform-wayland-hint">
          Install clipboard support: <code>sudo dnf install wl-clipboard</code> (Wayland) or{" "}
          <code>xclip</code> (X11).
        </p>
      )}
      {missingWindow && status.sessionType === "x11" && (
        <p className="knowledge-muted platform-wayland-hint">
          Install window context on X11: <code>sudo dnf install xdotool</code>.
        </p>
      )}
      {missingWindow && status.sessionType === "wayland" && status.desktopEnvironment === "other" && (
        <p className="knowledge-muted platform-wayland-hint">
          Active window context is not available on this Wayland desktop yet. KDE, GNOME, and
          Hyprland are supported.
        </p>
      )}
    </div>
  );
}
