import { invoke } from "@tauri-apps/api/core";
import { open } from "@tauri-apps/plugin-dialog";

export type KnowledgeCategory = "skills" | "agents" | "preferences" | "uploads";

export interface KnowledgeFileEntry {
  id: string;
  name: string;
  category: string;
  path: string;
  size_bytes: number;
  updated_at: string;
  extension: string;
}

export async function listKnowledgeFiles(
  category?: KnowledgeCategory
): Promise<KnowledgeFileEntry[]> {
  return invoke("list_knowledge_files", { category: category ?? null });
}

export async function pickAndImportKnowledge(
  category: KnowledgeCategory
): Promise<KnowledgeFileEntry[]> {
  const selected = await open({
    multiple: true,
    directory: false,
    filters: [
      {
        name: "Knowledge files",
        extensions: ["md", "txt", "json", "markdown"],
      },
    ],
  });
  if (!selected) return [];
  const paths = Array.isArray(selected) ? selected : [selected];
  return invoke("import_knowledge_files", {
    paths,
    category,
  });
}

export async function deleteKnowledgeFile(id: string): Promise<void> {
  await invoke("delete_knowledge_file", { id });
}

export async function appendUserPreference(
  note: string,
  source?: string
): Promise<void> {
  await invoke("append_user_preference", { note, source });
}

export async function getKnowledgeRoot(): Promise<string> {
  return invoke("get_knowledge_root");
}

export async function getAgentContextBundle(): Promise<string> {
  return invoke<string>("get_agent_context_bundle");
}

/** Trim knowledge bundle so it fits in the model context window. */
export function trimContextBundle(bundle: string, maxChars = 12_000): string {
  const trimmed = bundle.trim();
  if (trimmed.length <= maxChars) return trimmed;
  return `${trimmed.slice(0, maxChars)}\n\n…(knowledge truncated)`;
}
