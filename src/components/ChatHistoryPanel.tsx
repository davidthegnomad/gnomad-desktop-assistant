import { Trash2 } from "lucide-react";
import type { ChatSessionSummary } from "../lib/chatHistory";
import { formatSessionTime } from "../lib/chatHistory";

interface ChatHistoryPanelProps {
  sessions: ChatSessionSummary[];
  activeId: string | null;
  onSelect: (id: string) => void;
  onDelete: (id: string) => void;
  /** When true, header (title + new chat) is rendered by ChatSidebar. */
  hideHeader?: boolean;
}

export function ChatHistoryPanel({
  sessions,
  activeId,
  onSelect,
  onDelete,
  hideHeader = false,
}: ChatHistoryPanelProps) {
  return (
    <div className="chat-history-panel">
      {!hideHeader && (
        <div className="side-rail-head">
          <span className="side-rail-title">Chats</span>
        </div>
      )}
      <ul className="chat-history-list">
        {sessions.length === 0 && (
          <li className="chat-history-empty">No chats yet</li>
        )}
        {sessions.map((s) => (
          <li key={s.id}>
            <button
              type="button"
              className={`chat-history-item ${activeId === s.id ? "active" : ""}`}
              onClick={() => onSelect(s.id)}
            >
              <span className="chat-history-item-title">{s.title}</span>
              {s.preview && (
                <span className="chat-history-item-preview">{s.preview}</span>
              )}
              <span className="chat-history-item-time">
                {formatSessionTime(s.updated_at)}
              </span>
            </button>
            <button
              type="button"
              className="icon-btn chat-history-delete"
              title="Delete chat"
              aria-label={`Delete ${s.title}`}
              onClick={(e) => {
                e.stopPropagation();
                onDelete(s.id);
              }}
            >
              <Trash2 size={13} />
            </button>
          </li>
        ))}
      </ul>
    </div>
  );
}
