import { useRef } from "react";
import { AlertCircle, ShieldAlert } from "lucide-react";
import { useFocusTrap } from "../hooks/useFocusTrap";

interface SudoGateModalProps {
  open: boolean;
  command: string;
  reason: string;
  hint?: string | null;
  onDeny: () => void;
  onApprove: () => void;
}

export function SudoGateModal({
  open,
  command,
  reason,
  hint,
  onDeny,
  onApprove,
}: SudoGateModalProps) {
  const dialogRef = useRef<HTMLDivElement>(null);
  useFocusTrap(dialogRef, open);

  if (!open) return null;
  return (
    <div className="modal-overlay" role="presentation">
      <div
        ref={dialogRef}
        className="danger-modal"
        role="dialog"
        aria-modal="true"
        aria-labelledby="sudo-gate-title"
      >
        <h3 id="sudo-gate-title" className="danger-title">
          <ShieldAlert size={20} />
          Review command
        </h3>
        <p className="danger-body">
          This command may be risky or need elevated privileges. Approve only if you trust it.
        </p>
        <div className="settings-row">
          <span className="settings-label">Reason</span>
          <div className="sudo-gate-reason">
            <AlertCircle size={14} />
            {reason}
          </div>
        </div>
        {hint && (
          <p className="knowledge-muted sudo-gate-hint">{hint}</p>
        )}
        <div className="settings-row">
          <span className="settings-label">Command</span>
          <pre className="cmd-preview">{command}</pre>
        </div>
        <div className="modal-actions">
          <button type="button" className="btn secondary" onClick={onDeny}>
            Deny
          </button>
          <button type="button" className="btn danger" onClick={onApprove}>
            Approve
          </button>
        </div>
      </div>
    </div>
  );
}
