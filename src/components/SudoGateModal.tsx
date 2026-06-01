import { AlertCircle, ShieldAlert } from "lucide-react";

interface SudoGateModalProps {
  open: boolean;
  command: string;
  reason: string;
  onDeny: () => void;
  onApprove: () => void;
}

export function SudoGateModal({
  open,
  command,
  reason,
  onDeny,
  onApprove,
}: SudoGateModalProps) {
  if (!open) return null;
  return (
    <div className="modal-overlay">
      <div className="danger-modal">
        <h3 className="danger-title">
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
