import { ShieldAlert } from "lucide-react";

interface PathGateModalProps {
  open: boolean;
  target: string;
  reason: string;
  onDeny: () => void;
  onApprove: () => void;
}

export function PathGateModal({
  open,
  target,
  reason,
  onDeny,
  onApprove,
}: PathGateModalProps) {
  if (!open) return null;
  return (
    <div className="modal-overlay">
      <div className="danger-modal">
        <h3 className="danger-title">
          <ShieldAlert size={20} />
          Access path outside workspace
        </h3>
        <p className="danger-body">
          The agent wants to access a path outside your workspace. Approve only if you trust this action.
        </p>
        <div className="settings-row">
          <span className="settings-label">Reason</span>
          <p className="knowledge-muted">{reason}</p>
        </div>
        {target && (
          <div className="settings-row">
            <span className="settings-label">Path</span>
            <pre className="cmd-preview">{target}</pre>
          </div>
        )}
        <div className="modal-actions">
          <button type="button" className="btn secondary" onClick={onDeny}>
            Deny
          </button>
          <button type="button" className="btn primary" onClick={onApprove}>
            Allow once
          </button>
        </div>
      </div>
    </div>
  );
}
