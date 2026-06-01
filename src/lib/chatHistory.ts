import { invoke } from "@tauri-apps/api/core";

export interface StoredChatMessage {
  role: "user" | "assistant";
  text: string;
  commandExecuted?: string;
}

export interface ChatSessionSummary {
  id: string;
  title: string;
  updated_at: number;
  preview: string;
}

export interface ChatSession {
  id: string;
  title: string;
  created_at: number;
  updated_at: number;
  messages: StoredChatMessage[];
}

export interface ChatStoreState {
  active_id: string | null;
  sessions: ChatSessionSummary[];
}

export async function getChatStore(): Promise<ChatStoreState> {
  return invoke<ChatStoreState>("get_chat_store");
}

export async function loadChatSession(id: string): Promise<ChatSession> {
  const session = await invoke<{
    id: string;
    title: string;
    created_at: number;
    updated_at: number;
    messages: {
      role: string;
      text: string;
      command_executed?: string;
    }[];
  }>("load_chat_session", { id });

  return {
    id: session.id,
    title: session.title,
    created_at: session.created_at,
    updated_at: session.updated_at,
    messages: session.messages.map((m) => ({
      role: m.role === "user" ? "user" : "assistant",
      text: m.text,
      commandExecuted: m.command_executed,
    })),
  };
}

export async function createChatSession(): Promise<ChatSession> {
  const session = await invoke<{
    id: string;
    title: string;
    created_at: number;
    updated_at: number;
    messages: {
      role: string;
      text: string;
      command_executed?: string;
    }[];
  }>("create_chat_session");

  return {
    id: session.id,
    title: session.title,
    created_at: session.created_at,
    updated_at: session.updated_at,
    messages: session.messages.map((m) => ({
      role: m.role === "user" ? "user" : "assistant",
      text: m.text,
      commandExecuted: m.command_executed,
    })),
  };
}

export async function saveChatSession(
  id: string,
  messages: StoredChatMessage[]
): Promise<ChatSessionSummary> {
  return invoke<ChatSessionSummary>("save_chat_session", {
    id,
    messages: messages.map((m) => ({
      role: m.role,
      text: m.text,
      command_executed: m.commandExecuted ?? null,
    })),
  });
}

export async function deleteChatSession(id: string): Promise<ChatSession> {
  const session = await invoke<{
    id: string;
    title: string;
    created_at: number;
    updated_at: number;
    messages: {
      role: string;
      text: string;
      command_executed?: string;
    }[];
  }>("delete_chat_session", { id });

  return {
    id: session.id,
    title: session.title,
    created_at: session.created_at,
    updated_at: session.updated_at,
    messages: session.messages.map((m) => ({
      role: m.role === "user" ? "user" : "assistant",
      text: m.text,
      commandExecuted: m.command_executed,
    })),
  };
}

export async function setActiveChatSession(id: string): Promise<void> {
  await invoke("set_active_chat_session", { id });
}

export function formatSessionTime(updatedAt: number): string {
  const d = new Date(updatedAt * 1000);
  const now = new Date();
  const sameDay =
    d.getFullYear() === now.getFullYear() &&
    d.getMonth() === now.getMonth() &&
    d.getDate() === now.getDate();
  if (sameDay) {
    return d.toLocaleTimeString([], { hour: "numeric", minute: "2-digit" });
  }
  return d.toLocaleDateString([], { month: "short", day: "numeric" });
}
