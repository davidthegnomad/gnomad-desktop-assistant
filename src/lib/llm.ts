import { invoke } from "@tauri-apps/api/core";
import type { ProviderMode } from "./preferences";

export interface LlmChatMessage {
  role: "user" | "assistant";
  content: string;
}

export async function chatCompletion(params: {
  provider: ProviderMode;
  model: string;
  messages: LlmChatMessage[];
  ollamaUrl?: string;
  systemContext?: string;
}): Promise<string> {
  const res = await invoke<{ content: string }>("chat_completion", {
    provider: params.provider,
    model: params.model,
    messages: params.messages,
    ollamaUrl: params.ollamaUrl ?? null,
    systemContext: params.systemContext ?? null,
  });
  return res.content;
}
