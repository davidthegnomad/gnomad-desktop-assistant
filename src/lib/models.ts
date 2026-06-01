import { getEnvLlmConfig } from "./envConfig";
import { loadCredential } from "./preferences";

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

/** Resolve which providers/models are usable from env, keychain, and in-app state. */
export async function resolveLlmAvailability(options?: {
  apiKey?: string;
  ollamaUrl?: string;
}): Promise<LlmAvailability> {
  const env = await getEnvLlmConfig();
  const keychainKey = await loadCredential("llm_api_key");
  const keychainOllama = await loadCredential("ollama_url");

  const inlineKey = options?.apiKey?.trim() ?? "";
  const cloudConfigured = !!(
    env.deepseek_api_key?.trim() ||
    keychainKey.trim() ||
    inlineKey
  );

  const ollamaUrl = (options?.ollamaUrl ?? keychainOllama).trim();
  const localConfigured = ollamaUrl.length > 0;

  return {
    cloudConfigured,
    localConfigured,
    cloudModels: cloudConfigured ? [...DEEPSEEK_MODELS] : [],
    localModels: localConfigured ? [...OLLAMA_MODELS] : [],
  };
}
