import { invoke } from "@tauri-apps/api/core";
import type { ApiKeySource } from "./apiKeys";

export interface EnvLlmConfig {
  deepseek_configured: boolean;
  env_path: string | null;
}

export async function getEnvLlmConfig(): Promise<EnvLlmConfig> {
  return invoke<EnvLlmConfig>("get_env_llm_config");
}

export async function hasLlmConfigured(): Promise<boolean> {
  return invoke<boolean>("has_llm_configured");
}

export async function getCloudApiKeySource(): Promise<ApiKeySource> {
  return invoke<ApiKeySource>("get_cloud_api_key_source");
}
