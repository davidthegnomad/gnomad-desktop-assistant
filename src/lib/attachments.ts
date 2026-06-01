import { invoke } from "@tauri-apps/api/core";
import { open } from "@tauri-apps/plugin-dialog";

export interface ChatAttachment {
  id: string;
  name: string;
  path: string;
  mime: string;
  kind: "text" | "image" | "file";
  sizeBytes: number;
}

export async function pickChatAttachments(): Promise<ChatAttachment[]> {
  const selected = await open({
    multiple: true,
    directory: false,
    filters: [
      {
        name: "Images",
        extensions: ["png", "jpg", "jpeg", "gif", "webp", "heic"],
      },
      {
        name: "Documents",
        extensions: [
          "txt",
          "md",
          "markdown",
          "json",
          "csv",
          "pdf",
          "xml",
          "html",
          "htm",
        ],
      },
    ],
  });
  if (!selected) return [];
  const paths = Array.isArray(selected) ? selected : [selected];
  const staged = await invoke<ChatAttachment[]>("stage_chat_attachments", {
    sourcePaths: paths,
  });
  return staged.map((a) => ({
    ...a,
    kind: a.kind as ChatAttachment["kind"],
  }));
}

export async function formatAttachmentsForPrompt(
  paths: string[]
): Promise<string> {
  if (paths.length === 0) return "";
  return invoke<string>("format_attachments_for_prompt", { paths });
}

export async function removeStagedAttachments(paths: string[]): Promise<void> {
  if (paths.length === 0) return;
  await invoke("remove_staged_attachments", { paths });
}

export function formatBytes(n: number): string {
  if (n < 1024) return `${n} B`;
  if (n < 1024 * 1024) return `${(n / 1024).toFixed(1)} KB`;
  return `${(n / (1024 * 1024)).toFixed(1)} MB`;
}
