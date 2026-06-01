import { getCloudApiKeySource, getEnvLlmConfig } from "./envConfig";
import { deleteCredential, hasCredential, saveCredential } from "./preferences";

export type ApiKeySource = "env" | "keychain" | "none";

export interface ApiKeySlot {
  id: string;
  credentialKey: string;
  name: string;
  description: string;
  envHint: string;
}

export interface ApiKeyStatus {
  slot: ApiKeySlot;
  configured: boolean;
  source: ApiKeySource;
  envPath: string | null;
  /** When true, key is only in .env — cannot edit or remove from the app. */
  lockedByEnv: boolean;
}

export const DEEPSEEK_API_KEY_SLOT: ApiKeySlot = {
  id: "deepseek",
  credentialKey: "llm_api_key",
  name: "Cloud LLM API",
  description: "OpenAI-compatible API key (DeepSeek default; also OpenAI, Groq, Together, etc.).",
  envHint: "DeepSeek_API_KEY or OPENAI_API_KEY",
};

export const MANAGED_API_KEY_SLOTS: ApiKeySlot[] = [DEEPSEEK_API_KEY_SLOT];

export async function fetchApiKeyStatuses(): Promise<ApiKeyStatus[]> {
  const env = await getEnvLlmConfig();
  const cloudSource = await getCloudApiKeySource();
  const keychainSet = await hasCredential(DEEPSEEK_API_KEY_SLOT.credentialKey);

  const deepseekConfigured =
    env.deepseek_configured || keychainSet || cloudSource !== "none";

  let source: ApiKeySource = "none";
  if (env.deepseek_configured || cloudSource === "env") {
    source = "env";
  } else if (keychainSet || cloudSource === "keychain") {
    source = "keychain";
  }

  return [
    {
      slot: DEEPSEEK_API_KEY_SLOT,
      configured: deepseekConfigured,
      source,
      envPath: env.env_path,
      lockedByEnv: env.deepseek_configured,
    },
  ];
}

export async function storeApiKey(slot: ApiKeySlot, value: string): Promise<void> {
  const trimmed = value.trim();
  if (!trimmed) {
    throw new Error("API key cannot be empty.");
  }
  await saveCredential(slot.credentialKey, trimmed);
}

export async function removeApiKey(slot: ApiKeySlot): Promise<void> {
  await deleteCredential(slot.credentialKey);
}
