import { getEnvLlmConfig } from "./envConfig";
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

export interface LlmAvailability {
  cloudConfigured: boolean;
  localConfigured: boolean;
  cloudModels: ModelOption[];
  localModels: ModelOption[];
}

export function isCloudModelValue(value: string): boolean {
  return DEEPSEEK_MODELS.some((m) => m.value === value);
}

export function pickDefaultCloudModel(models: ModelOption[]): string {
  return models[0]?.value ?? "deepseek-chat";
}

export function normalizeCloudModel(
  stored: string,
  available: ModelOption[]
): string {
  if (available.some((m) => m.value === stored)) return stored;
  return pickDefaultCloudModel(available);
}

/** Resolve which providers/models are usable from env and keychain (never reads key values in UI). */
export async function resolveLlmAvailability(options?: {
  ollamaUrl?: string;
}): Promise<LlmAvailability> {
  const env = await getEnvLlmConfig();
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

  return {
    cloudConfigured,
    localConfigured,
    cloudModels: cloudConfigured ? [...DEEPSEEK_MODELS] : [],
    localModels,
  };
}
