import { invoke } from "@tauri-apps/api/core";

export interface EnvLlmConfig {
  deepseek_api_key: string | null;
  loaded_from_env: boolean;
  env_path: string | null;
}

export async function getEnvLlmConfig(): Promise<EnvLlmConfig> {
  return invoke<EnvLlmConfig>("get_env_llm_config");
}

export async function hasLlmConfigured(): Promise<boolean> {
  return invoke<boolean>("has_llm_configured");
}

/** Prefer `.env` DeepSeek key, then keychain. */
export async function getEffectiveCloudApiKey(): Promise<string> {
  return invoke<string>("get_effective_cloud_api_key");
}

export async function resolveCloudApiKey(): Promise<{
  key: string;
  source: "env" | "keychain" | "none";
}> {
  const env = await getEnvLlmConfig();
  if (env.deepseek_api_key?.trim()) {
    return { key: env.deepseek_api_key.trim(), source: "env" };
  }
  const key = await invoke<string>("get_credential", { key: "llm_api_key" });
  if (key.trim()) {
    return { key: key.trim(), source: "keychain" };
  }
  return { key: "", source: "none" };
}
