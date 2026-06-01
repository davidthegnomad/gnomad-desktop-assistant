import { useCallback, useEffect, useState } from "react";
import { Globe } from "lucide-react";
import {
  CLOUD_API_PRESETS,
  getCloudApiConfig,
  presetForBaseUrl,
  saveCloudApiBaseUrl,
  type CloudApiConfig,
} from "../lib/cloudApi";

interface CloudApiSettingsProps {
  onChanged?: () => void;
}

export function CloudApiSettings({ onChanged }: CloudApiSettingsProps) {
  const [config, setConfig] = useState<CloudApiConfig | null>(null);
  const [presetId, setPresetId] = useState("deepseek");
  const [customUrl, setCustomUrl] = useState("");
  const [status, setStatus] = useState<string | null>(null);
  const [busy, setBusy] = useState(false);

  const refresh = useCallback(async () => {
    const next = await getCloudApiConfig();
    setConfig(next);
    const match = presetForBaseUrl(next.baseUrl);
    if (match) {
      setPresetId(match.id);
      setCustomUrl(match.baseUrl);
    } else {
      setPresetId("custom");
      setCustomUrl(next.baseUrl);
    }
  }, []);

  useEffect(() => {
    void refresh();
  }, [refresh]);

  const applyPreset = async (id: string) => {
    setPresetId(id);
    if (id === "custom") return;
    const preset = CLOUD_API_PRESETS.find((p) => p.id === id);
    if (!preset) return;
    setCustomUrl(preset.baseUrl);
    setBusy(true);
    setStatus(null);
    try {
      await saveCloudApiBaseUrl(preset.baseUrl);
      await refresh();
      onChanged?.();
      setStatus(`Endpoint set to ${preset.label}.`);
    } catch (err) {
      setStatus(err instanceof Error ? err.message : String(err));
    } finally {
      setBusy(false);
    }
  };

  const saveCustom = async () => {
    setBusy(true);
    setStatus(null);
    try {
      await saveCloudApiBaseUrl(customUrl);
      await refresh();
      onChanged?.();
      setStatus("Custom endpoint saved.");
    } catch (err) {
      setStatus(err instanceof Error ? err.message : String(err));
    } finally {
      setBusy(false);
    }
  };

  if (!config) return null;

  return (
    <div className="settings-section">
      <h4 className="section-title">
        <Globe size={16} />
        Cloud API endpoint
      </h4>
      <p className="knowledge-muted">
        OpenAI-compatible chat completions URL. DeepSeek is the default; switch for OpenAI, Groq,
        Together, or a local LM Studio server.
      </p>
      {config.baseUrlFromEnv && (
        <p className="knowledge-muted">
          Base URL is locked by <code>CLOUD_API_BASE_URL</code> or <code>OPENAI_BASE_URL</code> in
          .env.
        </p>
      )}
      <div className="settings-row">
        <span className="settings-label">Provider</span>
        <select
          className="settings-select"
          value={presetId}
          disabled={busy || config.baseUrlFromEnv}
          onChange={(e) => void applyPreset(e.target.value)}
        >
          {CLOUD_API_PRESETS.map((p) => (
            <option key={p.id} value={p.id}>
              {p.label}
            </option>
          ))}
          <option value="custom">Custom URL</option>
        </select>
      </div>
      {(presetId === "custom" || !presetForBaseUrl(config.baseUrl)) && (
        <>
          <div className="settings-row">
            <span className="settings-label">Base URL</span>
            <input
              className="settings-input"
              value={customUrl}
              disabled={busy || config.baseUrlFromEnv}
              onChange={(e) => setCustomUrl(e.target.value)}
              placeholder="https://api.example.com/v1"
            />
          </div>
          {!config.baseUrlFromEnv && (
            <button
              type="button"
              className="btn secondary btn-sm"
              disabled={busy || !customUrl.trim()}
              onClick={() => void saveCustom()}
            >
              Save endpoint
            </button>
          )}
        </>
      )}
      <p className="knowledge-muted">
        Active: <code>{config.baseUrl}</code>
        {config.isDefaultDeepseek ? " (DeepSeek models)" : " (enter model name manually below)"}
      </p>
      {status && <p className="knowledge-muted update-status">{status}</p>}
    </div>
  );
}
