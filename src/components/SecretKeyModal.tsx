import { useEffect, useState } from "react";
import { KeyRound, X } from "lucide-react";

interface SecretKeyModalProps {
  open: boolean;
  title: string;
  description: string;
  confirmLabel?: string;
  onClose: () => void;
  onSave: (value: string) => Promise<void>;
}

export function SecretKeyModal({
  open,
  title,
  description,
  confirmLabel = "Save",
  onClose,
  onSave,
}: SecretKeyModalProps) {
  const [value, setValue] = useState("");
  const [error, setError] = useState("");
  const [saving, setSaving] = useState(false);

  useEffect(() => {
    if (open) {
      setValue("");
      setError("");
      setSaving(false);
    }
  }, [open]);

  if (!open) return null;

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    setError("");
    if (!value.trim()) {
      setError("Enter a key before saving.");
      return;
    }
    setSaving(true);
    try {
      await onSave(value.trim());
      onClose();
    } catch (err) {
      setError(String(err));
    } finally {
      setSaving(false);
    }
  };

  return (
    <div className="modal-overlay secret-key-overlay" role="presentation">
      <div
        className="secret-key-modal"
        role="dialog"
        aria-labelledby="secret-key-title"
        aria-modal="true"
      >
        <header className="secret-key-header">
          <div className="secret-key-title-row">
            <KeyRound size={18} style={{ color: "var(--color-accent)" }} />
            <h3 id="secret-key-title">{title}</h3>
          </div>
          <button
            type="button"
            className="icon-btn"
            onClick={onClose}
            aria-label="Close"
          >
            <X size={16} />
          </button>
        </header>
        <p className="secret-key-desc">{description}</p>
        <form onSubmit={(e) => void handleSubmit(e)}>
          <label className="settings-label" htmlFor="secret-key-input">
            New key
          </label>
          <input
            id="secret-key-input"
            type="password"
            className="settings-input"
            value={value}
            onChange={(e) => setValue(e.target.value)}
            placeholder="Paste your API key"
            autoComplete="off"
            autoFocus
            spellCheck={false}
          />
          <p className="secret-key-hint">
            Stored in your system keychain. The key is never shown after saving.
          </p>
          {error && <p className="onboarding-error">{error}</p>}
          <div className="secret-key-actions">
            <button type="button" className="btn secondary" onClick={onClose}>
              Cancel
            </button>
            <button type="submit" className="btn primary" disabled={saving}>
              {saving ? "Saving…" : confirmLabel}
            </button>
          </div>
        </form>
      </div>
    </div>
  );
}
