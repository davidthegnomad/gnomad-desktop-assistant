import { invoke } from "@tauri-apps/api/core";

export interface EmbeddedLlmStatus {
  available: boolean;
  loaded: boolean;
  modelPath: string | null;
  message: string;
}

export async function getEmbeddedLlmStatus(): Promise<EmbeddedLlmStatus> {
  return invoke<EmbeddedLlmStatus>("embedded_llm_status");
}

export async function embeddedLlmComplete(
  ggufPath: string,
  prompt: string,
  maxTokens?: number
): Promise<string> {
  return invoke<string>("embedded_llm_complete", {
    ggufPath,
    prompt,
    maxTokens: maxTokens ?? null,
  });
}

export async function embeddedLlmUnload(): Promise<void> {
  await invoke("embedded_llm_unload");
}
