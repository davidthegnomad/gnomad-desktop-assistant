import { invoke } from "@tauri-apps/api/core";
import { saveCredential } from "./preferences";

export interface CloudApiConfig {
  baseUrl: string;
  isDefaultDeepseek: boolean;
  apiKeyConfigured: boolean;
  baseUrlFromEnv: boolean;
}

export interface CloudApiPreset {
  id: string;
  label: string;
  baseUrl: string;
}

export const CLOUD_API_PRESETS: CloudApiPreset[] = [
  { id: "deepseek", label: "DeepSeek", baseUrl: "https://api.deepseek.com" },
  { id: "openai", label: "OpenAI", baseUrl: "https://api.openai.com/v1" },
  { id: "groq", label: "Groq", baseUrl: "https://api.groq.com/openai/v1" },
  { id: "together", label: "Together", baseUrl: "https://api.together.xyz/v1" },
  { id: "lmstudio", label: "LM Studio (local)", baseUrl: "http://localhost:1234/v1" },
];

export async function getCloudApiConfig(): Promise<CloudApiConfig> {
  return invoke<CloudApiConfig>("get_cloud_api_config");
}

export async function saveCloudApiBaseUrl(baseUrl: string): Promise<void> {
  const trimmed = baseUrl.trim();
  if (!trimmed) {
    await saveCredential("cloud_api_base_url", "");
    return;
  }
  await saveCredential("cloud_api_base_url", trimmed.replace(/\/+$/, ""));
}

export function presetForBaseUrl(baseUrl: string): CloudApiPreset | null {
  const normalized = baseUrl.trim().replace(/\/+$/, "");
  return CLOUD_API_PRESETS.find((p) => p.baseUrl === normalized) ?? null;
}
