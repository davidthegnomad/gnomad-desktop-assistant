import { invoke } from "@tauri-apps/api/core";
import { getEnvLlmConfig } from "./envConfig";
import { getCloudApiConfig } from "./cloudApi";
import { getAgentSettings } from "./agentSettings";
import { getEmbeddedLlmStatus } from "./embeddedLlm";
import { hasCredential, loadCredential } from "./preferences";

export interface ModelOption {
  value: string;
  label: string;
}

export const DEFAULT_OLLAMA_URL = "http://localhost:11434";

/** Models served via DeepSeek API (requires DeepSeek API key). */
export const DEEPSEEK_MODELS: ModelOption[] = [
  { value: "deepseek-chat", label: "DeepSeek Chat" },
  { value: "deepseek-reasoner", label: "DeepSeek Reasoner" },
];

export const EMBEDDED_GGUF_MODEL: ModelOption = {
  value: "embedded-gguf",
  label: "Embedded GGUF (local file)",
};

/** Common OpenAI-compatible model IDs when not on DeepSeek. */
export const OPENAI_COMPAT_MODELS: ModelOption[] = [
  { value: "gpt-4o-mini", label: "gpt-4o-mini" },
  { value: "gpt-4o", label: "gpt-4o" },
  { value: "llama-3.3-70b-versatile", label: "llama-3.3-70b (Groq)" },
];

export interface LlmAvailability {
  cloudConfigured: boolean;
  localConfigured: boolean;
  cloudModels: ModelOption[];
  localModels: ModelOption[];
  cloudUsesCustomEndpoint: boolean;
  ollamaReachable: boolean;
}

export function isCloudModelValue(value: string, models: ModelOption[]): boolean {
  return models.some((m) => m.value === value);
}

export function pickDefaultCloudModel(models: ModelOption[]): string {
  return models[0]?.value ?? "deepseek-chat";
}

export function pickDefaultLocalModel(models: ModelOption[]): string {
  if (models.length === 0) return "";
  const preferCoder = models.find((m) => m.value.toLowerCase().includes("coder"));
  if (preferCoder) return preferCoder.value;
  const preferInstruct = models.find((m) => m.value.toLowerCase().includes("instruct"));
  if (preferInstruct) return preferInstruct.value;
  return models[0].value;
}

export function normalizeCloudModel(
  stored: string,
  available: ModelOption[],
  allowCustom = false
): string {
  if (stored.trim() && (allowCustom || available.some((m) => m.value === stored))) {
    return stored;
  }
  return pickDefaultCloudModel(available);
}

export function normalizeLocalModel(stored: string, available: ModelOption[]): string {
  if (stored.trim() && available.some((m) => m.value === stored)) {
    return stored;
  }
  return pickDefaultLocalModel(available);
}

/** Query Ollama for installed chat models (excludes embedding models). */
export async function fetchOllamaModels(ollamaUrl: string): Promise<ModelOption[]> {
  const url = ollamaUrl.trim() || DEFAULT_OLLAMA_URL;
  return invoke<ModelOption[]>("list_ollama_models", { ollamaUrl: url });
}

/** Resolve which providers/models are usable from env, keychain, and live Ollama. */
export async function resolveLlmAvailability(options?: {
  ollamaUrl?: string;
}): Promise<LlmAvailability> {
  const env = await getEnvLlmConfig();
  const cloudConfig = await getCloudApiConfig();
  const keychainKeySet = await hasCredential("llm_api_key");
  const cloudConfigured = env.deepseek_configured || keychainKeySet;

  const keychainOllama = await loadCredential("ollama_url");
  const ollamaUrl = (options?.ollamaUrl ?? keychainOllama ?? DEFAULT_OLLAMA_URL).trim();

  let ggufConfigured = false;
  try {
    const agent = await getAgentSettings();
    const ggufPath = agent.commandPlannerGgufPath?.trim() ?? "";
    if (agent.useGgufForLocalChat && ggufPath.length > 0) {
      const embedded = await getEmbeddedLlmStatus();
      ggufConfigured = embedded.available;
    }
  } catch {
    ggufConfigured = false;
  }

  let ollamaReachable = false;
  let ollamaModels: ModelOption[] = [];
  try {
    ollamaModels = await fetchOllamaModels(ollamaUrl);
    ollamaReachable = true;
  } catch {
    ollamaReachable = false;
    ollamaModels = [];
  }

  const localModels: ModelOption[] = [];
  if (ggufConfigured) {
    localModels.push(EMBEDDED_GGUF_MODEL);
  }
  localModels.push(...ollamaModels);

  const localConfigured = (ollamaReachable && ollamaModels.length > 0) || ggufConfigured;

  const cloudUsesCustomEndpoint = cloudConfigured && !cloudConfig.isDefaultDeepseek;
  const cloudModels: ModelOption[] = cloudConfigured
    ? cloudConfig.isDefaultDeepseek
      ? [...DEEPSEEK_MODELS]
      : [...OPENAI_COMPAT_MODELS]
    : [];

  return {
    cloudConfigured,
    localConfigured,
    cloudModels,
    localModels,
    cloudUsesCustomEndpoint,
    ollamaReachable,
  };
}
