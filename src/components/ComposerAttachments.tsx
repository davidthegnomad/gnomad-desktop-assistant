import { convertFileSrc } from "@tauri-apps/api/core";
import { FileText, Paperclip, X } from "lucide-react";
import type { ChatAttachment } from "../lib/attachments";
import { formatBytes } from "../lib/attachments";

interface AttachmentChipListProps {
  attachments: ChatAttachment[];
  onRemove: (id: string) => void;
  disabled?: boolean;
}

export function AttachmentChipList({
  attachments,
  onRemove,
  disabled = false,
}: AttachmentChipListProps) {
  if (attachments.length === 0) return null;

  return (
    <div className="composer-attachments" role="list" aria-label="Attachments">
      {attachments.map((a) => (
        <div key={a.id} className="attachment-chip" role="listitem">
          {a.kind === "image" ? (
            <img
              src={convertFileSrc(a.path)}
              alt=""
              className="attachment-thumb"
            />
          ) : (
            <span className="attachment-icon" aria-hidden>
              {a.kind === "text" ? (
                <FileText size={14} />
              ) : (
                <Paperclip size={14} />
              )}
            </span>
          )}
          <span className="attachment-meta">
            <span className="attachment-name" title={a.name}>
              {a.name}
            </span>
            <span className="attachment-size">{formatBytes(a.sizeBytes)}</span>
          </span>
          <button
            type="button"
            className="attachment-remove"
            onClick={() => onRemove(a.id)}
            disabled={disabled}
            aria-label={`Remove ${a.name}`}
          >
            <X size={12} />
          </button>
        </div>
      ))}
    </div>
  );
}

interface AttachFilesButtonProps {
  onClick: () => void;
  disabled?: boolean;
  showLabel?: boolean;
}

export function AttachFilesButton({
  onClick,
  disabled = false,
  showLabel = false,
}: AttachFilesButtonProps) {
  return (
    <button
      type="button"
      className="composer-tool-btn"
      onClick={onClick}
      disabled={disabled}
      title="Attach pictures or files"
      aria-label="Attach pictures or files"
    >
      <Paperclip size={16} />
      {showLabel && <span className="composer-tool-label">Attach</span>}
    </button>
  );
}
