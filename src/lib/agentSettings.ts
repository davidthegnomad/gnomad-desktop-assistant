import { invoke } from "@tauri-apps/api/core";
import { open } from "@tauri-apps/plugin-dialog";

export type TrustMode = "standard" | "yolo";

export interface AgentSettings {
  workspaceRoot: string;
  trustMode: TrustMode;
  homeDir: string;
  commandPlannerEnabled: boolean;
  commandPlannerModel: string;
  commandPlannerUseChatLocalModel: boolean;
  commandPlannerGgufPath: string;
  useGgufForLocalChat: boolean;
  sandboxShellInYolo: boolean;
  /** Platform sandbox: full | workspace | none */
  sandboxLevel?: string;
}

export interface CommandPlannerPatch {
  enabled: boolean;
  model: string;
  useChatLocalModel: boolean;
  ggufPath: string;
}

export async function getAgentSettings(): Promise<AgentSettings> {
  return invoke<AgentSettings>("get_agent_settings");
}

export async function setWorkspaceRoot(path: string): Promise<AgentSettings> {
  return invoke<AgentSettings>("set_workspace_root", { path });
}

export async function setTrustMode(mode: TrustMode): Promise<AgentSettings> {
  return invoke<AgentSettings>("set_trust_mode", { mode });
}

export async function setCommandPlanner(
  patch: CommandPlannerPatch
): Promise<AgentSettings> {
  return invoke<AgentSettings>("set_command_planner", {
    enabled: patch.enabled,
    model: patch.model,
    useChatLocalModel: patch.useChatLocalModel,
    ggufPath: patch.ggufPath,
  });
}

export async function setAgentExperimentalFlags(flags: {
  useGgufForLocalChat: boolean;
  sandboxShellInYolo: boolean;
}): Promise<AgentSettings> {
  return invoke<AgentSettings>("set_agent_experimental_flags", {
    useGgufForLocalChat: flags.useGgufForLocalChat,
    sandboxShellInYolo: flags.sandboxShellInYolo,
  });
}

export async function pickWorkspaceFolder(): Promise<AgentSettings | null> {
  const selected = await open({
    directory: true,
    multiple: false,
    title: "Choose workspace folder",
  });
  if (!selected || Array.isArray(selected)) return null;
  return setWorkspaceRoot(selected);
}
