import { useCallback, useEffect, useRef, useState } from "react";
import {
  createChatSession,
  deleteChatSession,
  getChatStore,
  loadChatSession,
  saveChatSession,
  setActiveChatSession,
  type ChatSessionSummary,
  type StoredChatMessage,
} from "../lib/chatHistory";
import { removeStagedAttachments, type ChatAttachment } from "../lib/attachments";
import type { ShellRunState } from "../lib/shellSession";
import type { Message } from "../types/chat";
import { WELCOME_ONLY } from "../types/chat";

export function useChatSessions(prefsLoaded: boolean, isThinking: boolean) {
  const [messages, setMessages] = useState<Message[]>([
    { role: "assistant", text: WELCOME_ONLY },
  ]);
  const [activeSessionId, setActiveSessionId] = useState<string | null>(null);
  const [chatSessions, setChatSessions] = useState<ChatSessionSummary[]>([]);
  const saveChatDebounceRef = useRef<ReturnType<typeof setTimeout> | null>(null);

  const toStoredMessages = useCallback(
    (msgs: Message[]) =>
      msgs.map((m) => ({
        role: m.role,
        text: m.text,
        command_executed: m.commandExecuted,
        command_result: m.commandResult
          ? {
              success: m.commandResult.success,
              stdout: m.commandResult.stdout,
              stderr: m.commandResult.stderr,
              status_code: m.commandResult.status_code,
              cwd: m.commandResult.cwd,
              state: m.commandResult.state,
              message: m.commandResult.message,
              duration_ms: m.commandResult.duration_ms,
            }
          : undefined,
      })),
    []
  );

  const fromStoredMessages = useCallback((msgs: StoredChatMessage[]): Message[] => {
    return msgs.map((m) => ({
      role: m.role === "user" ? "user" : "assistant",
      text: m.text,
      commandExecuted: m.command_executed ?? m.commandExecuted,
      commandResult: m.command_result
        ? {
            success: m.command_result.success,
            stdout: m.command_result.stdout,
            stderr: m.command_result.stderr,
            status_code: m.command_result.status_code,
            cwd: m.command_result.cwd,
            state: m.command_result.state as ShellRunState | undefined,
            message: m.command_result.message,
            duration_ms: m.command_result.duration_ms,
          }
        : undefined,
    }));
  }, []);

  const refreshChatSessionList = useCallback(async () => {
    const store = await getChatStore();
    setChatSessions(store.sessions);
    return store;
  }, []);

  const persistCurrentChat = useCallback(
    async (msgs: Message[]) => {
      if (!activeSessionId) return;
      const summary = await saveChatSession(activeSessionId, toStoredMessages(msgs));
      setChatSessions((prev) => {
        const rest = prev.filter((s) => s.id !== summary.id);
        return [summary, ...rest].sort((a, b) => b.updated_at - a.updated_at);
      });
    },
    [activeSessionId, toStoredMessages]
  );

  useEffect(() => {
    if (!prefsLoaded || !activeSessionId || isThinking) return;
    if (saveChatDebounceRef.current) clearTimeout(saveChatDebounceRef.current);
    saveChatDebounceRef.current = setTimeout(() => {
      persistCurrentChat(messages).catch(console.error);
    }, 600);
    return () => {
      if (saveChatDebounceRef.current) clearTimeout(saveChatDebounceRef.current);
    };
  }, [messages, prefsLoaded, activeSessionId, isThinking, persistCurrentChat]);

  const clearPendingAttachments = (items: ChatAttachment[]) => {
    if (items.length === 0) return;
    void removeStagedAttachments(items.map((a) => a.path));
  };

  const handleNewChat = async (
    pendingAttachments: ChatAttachment[],
    clearPending: () => void
  ) => {
    clearPendingAttachments(pendingAttachments);
    clearPending();
    const session = await createChatSession();
    setActiveSessionId(session.id);
    setMessages(fromStoredMessages(session.messages));
    await refreshChatSessionList();
    return session;
  };

  const handleSelectChat = async (id: string) => {
    if (id === activeSessionId) return;
    await persistCurrentChat(messages);
    await setActiveChatSession(id);
    const session = await loadChatSession(id);
    setActiveSessionId(session.id);
    setMessages(fromStoredMessages(session.messages));
  };

  const handleDeleteChat = async (id: string) => {
    const wasActive = id === activeSessionId;
    if (wasActive) await persistCurrentChat(messages);
    const next = await deleteChatSession(id);
    await refreshChatSessionList();
    if (wasActive) {
      setActiveSessionId(next.id);
      setMessages(fromStoredMessages(next.messages));
    }
  };

  const initChatStore = async () => {
    const chatStore = await getChatStore();
    setChatSessions(chatStore.sessions);
    if (chatStore.active_id) {
      const session = await loadChatSession(chatStore.active_id);
      setActiveSessionId(session.id);
      setMessages(fromStoredMessages(session.messages));
    }
  };

  return {
    messages,
    setMessages,
    activeSessionId,
    chatSessions,
    persistCurrentChat,
    refreshChatSessionList,
    handleNewChat,
    handleSelectChat,
    handleDeleteChat,
    initChatStore,
    hasUserMessages: messages.some((m) => m.role === "user"),
  };
}
