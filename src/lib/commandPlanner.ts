import { invoke } from "@tauri-apps/api/core";
import { open } from "@tauri-apps/plugin-dialog";
import type { AgentSettings } from "./agentSettings";

export async function planShellCommand(params: {
  intent: string;
  model: string;
  ollamaUrl?: string;
  ggufPath?: string;
}): Promise<string> {
  return invoke<string>("plan_shell_command", {
    intent: params.intent,
    model: params.model,
    ollamaUrl: params.ollamaUrl ?? null,
    ggufPath: params.ggufPath ?? null,
  });
}

/** Resolve which Ollama model tag the planner should use. */
export function resolvePlannerModel(
  settings: AgentSettings,
  chatLocalModel?: string
): string {
  if (settings.commandPlannerUseChatLocalModel && chatLocalModel?.trim()) {
    return chatLocalModel.trim();
  }
  return settings.commandPlannerModel.trim() || "llama3.2:1b";
}

export async function pickGgufFile(): Promise<string | null> {
  const selected = await open({
    multiple: false,
    title: "Choose GGUF model file",
    filters: [{ name: "GGUF", extensions: ["gguf"] }],
  });
  if (!selected || Array.isArray(selected)) return null;
  return selected;
}

export async function tryPlanInvalidCommand(
  invalidText: string,
  settings: AgentSettings,
  ollamaUrl: string,
  chatLocalModel?: string
): Promise<string | null> {
  if (!settings.commandPlannerEnabled) return null;
  try {
    const command = await planShellCommand({
      intent: invalidText,
      model: resolvePlannerModel(settings, chatLocalModel),
      ollamaUrl,
      ggufPath: settings.commandPlannerGgufPath || undefined,
    });
    return command;
  } catch {
    return null;
  }
}
