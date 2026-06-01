import { useRef, useState, useEffect } from "react";
import { getEnvLlmConfig } from "../lib/envConfig";
import { Lock } from "lucide-react";
import { GnomadLogo } from "./GnomadLogo";
import { APP_NAME, MUSHROOM } from "../lib/brand";
import type { ProviderMode } from "../lib/preferences";
import { DEEPSEEK_MODELS, normalizeCloudModel } from "../lib/models";
import { storeApiKey, DEEPSEEK_API_KEY_SLOT } from "../lib/apiKeys";
import { useFocusTrap } from "../hooks/useFocusTrap";
import {
  saveCredential,
  setOnboardingComplete,
  setStoredLocalModel,
  setStoredModel,
  setStoredProvider,
} from "../lib/preferences";

interface OnboardingModalProps {
  onComplete: (config: {
    provider: ProviderMode;
    ollamaUrl: string;
    cloudModel: string;
    localModel: string;
  }) => void;
}

export function OnboardingModal({ onComplete }: OnboardingModalProps) {
  const dialogRef = useRef<HTMLDivElement>(null);
  useFocusTrap(dialogRef, true);

  const [step, setStep] = useState<1 | 2>(1);
  const [provider, setProvider] = useState<ProviderMode>("cloud");
  const [apiKeyDraft, setApiKeyDraft] = useState("");
  const [ollamaUrl, setOllamaUrl] = useState("http://localhost:11434");
  const [cloudModel, setCloudModel] = useState("deepseek-chat");
  const [localModel, setLocalModel] = useState("llama3.2");
  const [error, setError] = useState("");
  const [saving, setSaving] = useState(false);
  const [envKeyConfigured, setEnvKeyConfigured] = useState(false);

  useEffect(() => {
    getEnvLlmConfig().then((env) => {
      if (env.deepseek_configured) {
        setCloudModel(normalizeCloudModel("deepseek-chat", DEEPSEEK_MODELS));
        setEnvKeyConfigured(true);
      }
    });
  }, []);

  const finish = async () => {
    setError("");
    if (provider === "cloud" && !apiKeyDraft.trim() && !envKeyConfigured) {
      setError("Enter your API key or add OPENAI_API_KEY / DeepSeek_API_KEY to .env");
      return;
    }
    if (provider === "local" && !ollamaUrl.trim()) {
      setError("Enter your Ollama server URL.");
      return;
    }

    setSaving(true);
    try {
      setStoredProvider(provider);
      if (provider === "cloud") {
        if (!envKeyConfigured && apiKeyDraft.trim()) {
          await storeApiKey(DEEPSEEK_API_KEY_SLOT, apiKeyDraft.trim());
        }
        setStoredModel(cloudModel);
      } else {
        await saveCredential("ollama_url", ollamaUrl.trim());
        setStoredLocalModel(localModel);
      }
      setOnboardingComplete();
      onComplete({
        provider,
        ollamaUrl: ollamaUrl.trim(),
        cloudModel,
        localModel,
      });
    } catch (e) {
      setError(String(e));
    } finally {
      setSaving(false);
    }
  };

  return (
    <div className="modal-overlay onboarding-overlay" role="presentation">
      <div
        ref={dialogRef}
        className="onboarding-card"
        role="dialog"
        aria-modal="true"
        aria-labelledby="onboarding-title"
      >
        <div className="onboarding-spark">
          <GnomadLogo size="lg" />
        </div>
        <h2 id="onboarding-title" className="onboarding-title">
          Welcome to {APP_NAME}
        </h2>
        <p className="onboarding-subtitle">
          Connect a cloud API (DeepSeek, OpenAI-compatible) or a local Ollama model. Change anytime in Settings.
        </p>

        {step === 1 && (
          <div className="provider-cards" role="group" aria-label="Choose provider">
            <button
              type="button"
              className={`provider-card ${provider === "cloud" ? "selected" : ""}`}
              aria-pressed={provider === "cloud"}
              onClick={() => setProvider("cloud")}
            >
              <span className="provider-card-emoji" aria-hidden>{MUSHROOM}</span>
              <span className="provider-card-title">Cloud API</span>
              <span className="provider-card-desc">OpenAI-compatible endpoints</span>
            </button>
            <button
              type="button"
              className={`provider-card ${provider === "local" ? "selected" : ""}`}
              aria-pressed={provider === "local"}
              onClick={() => setProvider("local")}
            >
              <span className="provider-card-emoji" aria-hidden>🖥️</span>
              <span className="provider-card-title">Local model</span>
              <span className="provider-card-desc">Ollama on your machine</span>
            </button>
          </div>
        )}

        {step === 2 && provider === "cloud" && (
          <div className="onboarding-form">
            {envKeyConfigured ? (
              <>
                <p className="onboarding-hint api-key-env-note">
                  <Lock size={14} aria-hidden />
                  Cloud API key is configured via <code>.env</code>. The value is not shown here.
                </p>
              </>
            ) : (
              <>
                <label className="settings-label" htmlFor="onboarding-api-key">
                  API key
                </label>
                <input
                  id="onboarding-api-key"
                  type="password"
                  className="settings-input"
                  value={apiKeyDraft}
                  onChange={(e) => setApiKeyDraft(e.target.value)}
                  placeholder="Paste your cloud API key"
                  autoComplete="off"
                  autoFocus
                  spellCheck={false}
                />
                <p className="onboarding-hint">
                  Saved to your system keychain. Add or replace keys in Settings → API keys; endpoint in Cloud API.
                </p>
              </>
            )}
            <label className="settings-label" htmlFor="onboarding-cloud-model">
              Default model
            </label>
            <select
              id="onboarding-cloud-model"
              className="settings-select"
              value={cloudModel}
              onChange={(e) => setCloudModel(e.target.value)}
            >
              {DEEPSEEK_MODELS.map((m) => (
                <option key={m.value} value={m.value}>
                  {m.label}
                </option>
              ))}
            </select>
          </div>
        )}

        {step === 2 && provider === "local" && (
          <div className="onboarding-form">
            <label className="settings-label" htmlFor="onboarding-ollama-url">
              Ollama server URL
            </label>
            <input
              id="onboarding-ollama-url"
              className="settings-input"
              value={ollamaUrl}
              onChange={(e) => setOllamaUrl(e.target.value)}
              placeholder="http://localhost:11434"
              autoFocus
            />
            <label className="settings-label" htmlFor="onboarding-local-model">
              Model name
            </label>
            <input
              id="onboarding-local-model"
              className="settings-input"
              value={localModel}
              onChange={(e) => setLocalModel(e.target.value)}
              placeholder="llama3.2"
            />
            <p className="onboarding-hint">
              Run <code>ollama pull {localModel || "llama3.2"}</code> if the model is not installed yet.
            </p>
          </div>
        )}

        {error && (
          <p className="onboarding-error" role="alert">
            {error}
          </p>
        )}

        <div className="onboarding-actions">
          {step === 2 && (
            <button type="button" className="btn secondary" onClick={() => setStep(1)}>
              Back
            </button>
          )}
          {step === 1 ? (
            <button type="button" className="btn primary" onClick={() => setStep(2)}>
              Continue
            </button>
          ) : (
            <button type="button" className="btn primary" onClick={() => void finish()} disabled={saving}>
              {saving ? "Saving…" : "Get started"}
            </button>
          )}
        </div>
      </div>
    </div>
  );
}
