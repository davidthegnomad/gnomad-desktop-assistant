import { useCallback, useState } from "react";
import { appendUserPreference } from "../lib/knowledge";
import { getEnvLlmConfig } from "../lib/envConfig";
import {
  getOnboardingComplete,
  getStoredLocalModel,
  getStoredModel,
  getStoredProvider,
  hasProviderConfigured,
  loadCredential,
  saveCredential,
  setStoredLocalModel,
  setStoredModel,
  setStoredProvider,
  type ProviderMode,
} from "../lib/preferences";
import {
  normalizeCloudModel,
  resolveLlmAvailability,
  type LlmAvailability,
} from "../lib/models";

export function useLlmSettings() {
  const [apiType, setApiType] = useState<ProviderMode>("cloud");
  const [selectedModel, setSelectedModel] = useState("deepseek-chat");
  const [llmAvailability, setLlmAvailability] = useState<LlmAvailability>({
    cloudConfigured: false,
    localConfigured: false,
    cloudModels: [],
    localModels: [],
  });
  const [localModel, setLocalModel] = useState("llama3.2");
  const [ollamaUrl, setOllamaUrl] = useState("http://localhost:11434");
  const [showOnboarding, setShowOnboarding] = useState(false);
  const [prefsLoaded, setPrefsLoaded] = useState(false);

  const refreshLlmAvailability = useCallback(
    async (overrides?: { ollamaUrl?: string }) => {
      const savedOllama = overrides?.ollamaUrl ?? (await loadCredential("ollama_url"));
      const availability = await resolveLlmAvailability({ ollamaUrl: savedOllama });
      setLlmAvailability(availability);

      const storedModel = getStoredModel();
      const cloudModel = normalizeCloudModel(storedModel, availability.cloudModels);
      if (cloudModel !== storedModel) {
        setSelectedModel(cloudModel);
        setStoredModel(cloudModel);
      } else {
        setSelectedModel(cloudModel);
      }

      setApiType((current) => {
        if (current === "cloud" && !availability.cloudConfigured && availability.localConfigured) {
          setStoredProvider("local");
          return "local";
        }
        if (current === "local" && !availability.localConfigured && availability.cloudConfigured) {
          setStoredProvider("cloud");
          setSelectedModel(cloudModel);
          return "cloud";
        }
        return current;
      });

      return availability;
    },
    []
  );

  const initLlmPrefs = async () => {
    const provider = getStoredProvider();
    setApiType(provider);
    setSelectedModel(getStoredModel());
    setLocalModel(getStoredLocalModel());

    const env = await getEnvLlmConfig();
    if (env.deepseek_configured) {
      setApiType("cloud");
      setStoredProvider("cloud");
    }

    const url = await loadCredential("ollama_url");
    if (url) setOllamaUrl(url);

    const availability = await resolveLlmAvailability({ ollamaUrl: url });
    setLlmAvailability(availability);
    const cloudModel = normalizeCloudModel(getStoredModel(), availability.cloudModels);
    setSelectedModel(cloudModel);
    setStoredModel(cloudModel);

    if (provider === "cloud" && !availability.cloudConfigured && availability.localConfigured) {
      setApiType("local");
      setStoredProvider("local");
    } else if (provider === "local" && !availability.localConfigured && availability.cloudConfigured) {
      setApiType("cloud");
      setStoredProvider("cloud");
    }

    const configured = await hasProviderConfigured();
    if (!configured && !getOnboardingComplete()) {
      setShowOnboarding(true);
    }

    setPrefsLoaded(true);
    return availability;
  };

  const saveOllamaUrl = async (url: string) => {
    setOllamaUrl(url);
    await saveCredential("ollama_url", url);
    await refreshLlmAvailability({ ollamaUrl: url });
  };

  const handleProviderChange = (mode: ProviderMode) => {
    if (mode === "cloud" && !llmAvailability.cloudConfigured) return;
    if (mode === "local" && !llmAvailability.localConfigured) return;
    setApiType(mode);
    setStoredProvider(mode);
    void appendUserPreference(`LLM provider set to ${mode}`, "settings");
  };

  const handleModelChange = (model: string) => {
    if (!model) return;
    if (apiType === "cloud") {
      if (!llmAvailability.cloudModels.some((m) => m.value === model)) return;
      setSelectedModel(model);
      setStoredModel(model);
      void appendUserPreference(`Cloud model: ${model}`, "settings");
    } else {
      setLocalModel(model);
      setStoredLocalModel(model);
      void appendUserPreference(`Local Ollama model: ${model}`, "settings");
    }
  };

  const headerModelValue =
    apiType === "cloud"
      ? llmAvailability.cloudModels.some((m) => m.value === selectedModel)
        ? selectedModel
        : llmAvailability.cloudModels[0]?.value ?? ""
      : llmAvailability.localModels.some((m) => m.value === localModel)
        ? localModel
        : llmAvailability.localModels[0]?.value ?? "";

  return {
    apiType,
    selectedModel,
    localModel,
    ollamaUrl,
    llmAvailability,
    showOnboarding,
    setShowOnboarding,
    prefsLoaded,
    refreshLlmAvailability,
    initLlmPrefs,
    saveOllamaUrl,
    handleProviderChange,
    handleModelChange,
    headerModelValue,
    setApiType,
    setOllamaUrl,
    setSelectedModel,
    setLocalModel,
  };
}
