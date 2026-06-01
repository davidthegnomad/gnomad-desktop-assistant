import { useCallback, useState } from "react";
import { Download } from "lucide-react";
import { checkForUpdates, installUpdate, type UpdateCheckResult } from "../lib/updater";
import {
  getStoredAutoCheckUpdates,
  getStoredUpdateChannel,
  setStoredAutoCheckUpdates,
  setStoredUpdateChannel,
  type UpdateChannel,
} from "../lib/preferences";

export function UpdateSettings() {
  const [channel, setChannel] = useState<UpdateChannel>(getStoredUpdateChannel());
  const [autoCheck, setAutoCheck] = useState(getStoredAutoCheckUpdates());
  const [status, setStatus] = useState<string | null>(null);
  const [result, setResult] = useState<UpdateCheckResult | null>(null);
  const [busy, setBusy] = useState(false);

  const onChannelChange = (next: UpdateChannel) => {
    setChannel(next);
    setStoredUpdateChannel(next);
    setResult(null);
    setStatus(null);
  };

  const onAutoCheckChange = (enabled: boolean) => {
    setAutoCheck(enabled);
    setStoredAutoCheckUpdates(enabled);
  };

  const runCheck = useCallback(async () => {
    setBusy(true);
    setStatus("Checking for updates…");
    try {
      const info = await checkForUpdates(channel);
      setResult(info);
      if (info.available) {
        setStatus(`Update available: v${info.version ?? "?"}`);
      } else {
        setStatus(`You're on the latest ${channel} build (v${info.currentVersion}).`);
      }
    } catch (err) {
      setResult(null);
      setStatus(err instanceof Error ? err.message : String(err));
    } finally {
      setBusy(false);
    }
  }, [channel]);

  const runInstall = async () => {
    setBusy(true);
    setStatus("Downloading update…");
    try {
      await installUpdate(channel);
    } catch (err) {
      setStatus(err instanceof Error ? err.message : String(err));
      setBusy(false);
    }
  };

  return (
    <div className="settings-section">
      <h4 className="section-title">
        <Download size={16} />
        Updates
      </h4>
      <p className="knowledge-muted">
        Check GitHub Releases for signed installers. Beta channel includes pre-releases.
      </p>
      <div className="agent-trust-options">
        <label className="agent-trust-option">
          <input
            type="radio"
            name="updateChannel"
            checked={channel === "stable"}
            onChange={() => onChannelChange("stable")}
          />
          <span>
            <strong>Stable</strong> — latest release
          </span>
        </label>
        <label className="agent-trust-option">
          <input
            type="radio"
            name="updateChannel"
            checked={channel === "beta"}
            onChange={() => onChannelChange("beta")}
          />
          <span>
            <strong>Beta</strong> — alpha / pre-release builds
          </span>
        </label>
      </div>
      <label className="agent-trust-option">
        <input
          type="checkbox"
          checked={autoCheck}
          onChange={(e) => onAutoCheckChange(e.target.checked)}
        />
        <span>
          <strong>Check for updates on startup</strong> — silent background check when the app opens
        </span>
      </label>
      <div className="settings-row update-actions">
        <button type="button" className="btn-secondary btn-sm" disabled={busy} onClick={() => void runCheck()}>
          Check for updates
        </button>
        {result?.available && (
          <button type="button" className="btn primary btn-sm" disabled={busy} onClick={() => void runInstall()}>
            Install v{result.version}
          </button>
        )}
      </div>
      {status && <p className="knowledge-muted update-status">{status}</p>}
    </div>
  );
}
