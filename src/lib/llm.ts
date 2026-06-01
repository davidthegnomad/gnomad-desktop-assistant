import { invoke } from "@tauri-apps/api/core";
import type { ProviderMode } from "./preferences";
import type { AgentToolCall } from "./agentRuntime";

export interface LlmChatMessage {
  role: "user" | "assistant" | "tool" | "system";
  content: string;
  toolCallId?: string;
  toolCalls?: AgentToolCall[];
}

export interface ChatTurnResult {
  content?: string;
  toolCalls: AgentToolCall[];
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

export async function chatCompletionTurn(params: {
  provider: ProviderMode;
  model: string;
  messages: LlmChatMessage[];
  ollamaUrl?: string;
  systemContext?: string;
  enableTools?: boolean;
}): Promise<ChatTurnResult> {
  return invoke<ChatTurnResult>("chat_completion_turn", {
    provider: params.provider,
    model: params.model,
    messages: params.messages,
    ollamaUrl: params.ollamaUrl ?? null,
    systemContext: params.systemContext ?? null,
    enableTools: params.enableTools ?? true,
  });
}
