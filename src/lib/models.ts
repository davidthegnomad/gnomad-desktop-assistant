import { getEnvLlmConfig } from "./envConfig";
import { getCloudApiConfig } from "./cloudApi";
import { getAgentSettings } from "./agentSettings";
import { getEmbeddedLlmStatus } from "./embeddedLlm";
import { hasCredential, loadCredential } from "./preferences";

export interface ModelOption {
  value: string;
  label: string;
}

/** Models served via DeepSeek API (requires DeepSeek API key). */
export const DEEPSEEK_MODELS: ModelOption[] = [
  { value: "deepseek-chat", label: "DeepSeek Chat" },
  { value: "deepseek-reasoner", label: "DeepSeek Reasoner" },
];

/** Common Ollama model tags (requires Ollama server URL). */
export const OLLAMA_MODELS: ModelOption[] = [
  { value: "llama3.2", label: "llama3.2" },
  { value: "llama3", label: "llama3" },
  { value: "mistral", label: "mistral" },
  { value: "qwen2.5-coder", label: "qwen2.5-coder" },
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
}

export function isCloudModelValue(value: string, models: ModelOption[]): boolean {
  return models.some((m) => m.value === value);
}

export function pickDefaultCloudModel(models: ModelOption[]): string {
  return models[0]?.value ?? "deepseek-chat";
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

/** Resolve which providers/models are usable from env and keychain (never reads key values in UI). */
export async function resolveLlmAvailability(options?: {
  ollamaUrl?: string;
}): Promise<LlmAvailability> {
  const env = await getEnvLlmConfig();
  const cloudConfig = await getCloudApiConfig();
  const keychainKeySet = await hasCredential("llm_api_key");
  const cloudConfigured = env.deepseek_configured || keychainKeySet;

  const keychainOllama = await loadCredential("ollama_url");
  const ollamaUrl = (options?.ollamaUrl ?? keychainOllama).trim();
  const ollamaConfigured = ollamaUrl.length > 0;

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

  const localConfigured = ollamaConfigured || ggufConfigured;
  const localModels: ModelOption[] = [];
  if (ggufConfigured) {
    localModels.push(EMBEDDED_GGUF_MODEL);
  }
  if (ollamaConfigured) {
    localModels.push(...OLLAMA_MODELS);
  }

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
  };
}
