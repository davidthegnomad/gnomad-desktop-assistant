import { useState, useEffect } from "react";
import { getEnvLlmConfig } from "../lib/envConfig";
import { Eye, EyeOff } from "lucide-react";
import { GnomadLogo } from "./GnomadLogo";
import { APP_NAME, LLAMA, MUSHROOM } from "../lib/brand";
import type { ProviderMode } from "../lib/preferences";
import { DEEPSEEK_MODELS, normalizeCloudModel } from "../lib/models";
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
    apiKey: string;
    ollamaUrl: string;
    cloudModel: string;
    localModel: string;
  }) => void;
}

export function OnboardingModal({ onComplete }: OnboardingModalProps) {
  const [step, setStep] = useState<1 | 2>(1);
  const [provider, setProvider] = useState<ProviderMode>("cloud");
  const [apiKey, setApiKey] = useState("");
  const [apiKeyVisible, setApiKeyVisible] = useState(false);
  const [ollamaUrl, setOllamaUrl] = useState("http://localhost:11434");
  const [cloudModel, setCloudModel] = useState("deepseek-chat");
  const [localModel, setLocalModel] = useState("llama3.2");
  const [error, setError] = useState("");
  const [saving, setSaving] = useState(false);
  const [envLoaded, setEnvLoaded] = useState(false);

  useEffect(() => {
    getEnvLlmConfig().then((env) => {
      if (env.deepseek_api_key) {
        setApiKey(env.deepseek_api_key);
        setCloudModel(normalizeCloudModel("deepseek-chat", DEEPSEEK_MODELS));
        setEnvLoaded(true);
      }
    });
  }, []);

  const finish = async () => {
    setError("");
    if (provider === "cloud" && !apiKey.trim() && !envLoaded) {
      setError("Enter your API key or add DeepSeek_API_KEY to .env");
      return;
    }
    if (provider === "local" && !ollamaUrl.trim()) {
      setError("Enter your Ollama server URL.");
      return;
    }

    setSaving(true);
    try {
      setStoredProvider(provider);
      if (provider === "cloud" && !envLoaded) {
        await saveCredential("llm_api_key", apiKey.trim());
        setStoredModel(cloudModel);
      } else if (provider === "cloud") {
        setStoredModel(cloudModel);
      } else {
        await saveCredential("ollama_url", ollamaUrl.trim());
        setStoredLocalModel(localModel);
      }
      setOnboardingComplete();
      onComplete({
        provider,
        apiKey: apiKey.trim(),
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
    <div className="modal-overlay onboarding-overlay">
      <div className="onboarding-card">
        <div className="onboarding-spark">
          <GnomadLogo size="lg" />
        </div>
        <h2 className="onboarding-title">Welcome to {APP_NAME}</h2>
        <p className="onboarding-subtitle">
          Connect a cloud API or a local model to get started. You can change this anytime in settings.
        </p>

        {step === 1 && (
          <div className="provider-cards">
            <button
              type="button"
              className={`provider-card ${provider === "cloud" ? "selected" : ""}`}
              onClick={() => setProvider("cloud")}
            >
              <span className="provider-card-emoji" aria-hidden>{MUSHROOM}</span>
              <span className="provider-card-title">Cloud API</span>
              <span className="provider-card-desc">Gemini, Claude, or OpenAI</span>
            </button>
            <button
              type="button"
              className={`provider-card ${provider === "local" ? "selected" : ""}`}
              onClick={() => setProvider("local")}
            >
              <span className="provider-card-emoji" aria-hidden>{LLAMA}</span>
              <span className="provider-card-title">Local model</span>
              <span className="provider-card-desc">Ollama llama on your machine</span>
            </button>
          </div>
        )}

        {step === 2 && provider === "cloud" && (
          <div className="onboarding-form">
            <label className="settings-label">API key</label>
            <div className="input-with-icon">
              <input
                type={apiKeyVisible ? "text" : "password"}
                className="settings-input"
                value={apiKey}
                onChange={(e) => setApiKey(e.target.value)}
                placeholder="AIza... or sk-..."
                autoFocus
              />
              <button
                type="button"
                className="icon-btn input-icon-btn"
                onClick={() => setApiKeyVisible(!apiKeyVisible)}
                aria-label="Toggle key visibility"
              >
                {apiKeyVisible ? <EyeOff size={16} /> : <Eye size={16} />}
              </button>
            </div>
            <label className="settings-label">Default model</label>
            <select
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
            <p className="onboarding-hint">
              {envLoaded
                ? "Using DeepSeek_API_KEY from your project .env file."
                : "Stored securely in your system keychain, or use .env for testing."}
            </p>
          </div>
        )}

        {step === 2 && provider === "local" && (
          <div className="onboarding-form">
            <label className="settings-label">Ollama server URL</label>
            <input
              className="settings-input"
              value={ollamaUrl}
              onChange={(e) => setOllamaUrl(e.target.value)}
              placeholder="http://localhost:11434"
              autoFocus
            />
            <label className="settings-label">Model name</label>
            <input
              className="settings-input"
              value={localModel}
              onChange={(e) => setLocalModel(e.target.value)}
              placeholder="llama3.2"
            />
            <p className="onboarding-hint">Run <code>ollama pull {localModel || "llama3.2"}</code> if the model is not installed yet.</p>
          </div>
        )}

        {error && <p className="onboarding-error">{error}</p>}

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
            <button type="button" className="btn primary" onClick={finish} disabled={saving}>
              {saving ? "Saving…" : "Get started"}
            </button>
          )}
        </div>
      </div>
    </div>
  );
}
