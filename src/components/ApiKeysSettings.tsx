import { useCallback, useEffect, useState } from "react";
import { KeyRound, Lock, Pencil, Plus, Trash2 } from "lucide-react";
import {
  fetchApiKeyStatuses,
  removeApiKey,
  storeApiKey,
  type ApiKeyStatus,
} from "../lib/apiKeys";
import { SecretKeyModal } from "./SecretKeyModal";

interface ApiKeysSettingsProps {
  onKeysChanged?: () => void;
}

function statusLabel(status: ApiKeyStatus): string {
  if (!status.configured) return "Not set";
  if (status.source === "env") return "Set (from .env)";
  if (status.source === "keychain") return "Set (keychain)";
  return "Set";
}

export function ApiKeysSettings({ onKeysChanged }: ApiKeysSettingsProps) {
  const [statuses, setStatuses] = useState<ApiKeyStatus[]>([]);
  const [loading, setLoading] = useState(true);
  const [editorOpen, setEditorOpen] = useState(false);
  const [editing, setEditing] = useState<ApiKeyStatus | null>(null);
  const [removingId, setRemovingId] = useState<string | null>(null);

  const refresh = useCallback(async () => {
    setLoading(true);
    try {
      setStatuses(await fetchApiKeyStatuses());
    } finally {
      setLoading(false);
    }
  }, []);

  useEffect(() => {
    void refresh();
  }, [refresh]);

  const openEditor = (status: ApiKeyStatus) => {
    if (status.lockedByEnv) return;
    setEditing(status);
    setEditorOpen(true);
  };

  const handleSaved = async (value: string) => {
    if (!editing) return;
    await storeApiKey(editing.slot, value);
    await refresh();
    onKeysChanged?.();
  };

  const handleRemove = async (status: ApiKeyStatus) => {
    if (status.lockedByEnv || !status.configured) return;
    if (status.source !== "keychain") return;
    setRemovingId(status.slot.id);
    try {
      await removeApiKey(status.slot);
      await refresh();
      onKeysChanged?.();
    } catch (err) {
      console.error(err);
    } finally {
      setRemovingId(null);
    }
  };

  return (
    <>
      <div className="settings-section api-keys-section">
        <h4 className="section-title">
          <KeyRound size={16} />
          API keys
        </h4>
        <p className="knowledge-muted api-keys-intro">
          Keys are stored in your system keychain. Values are never shown in the
          app after saving.
        </p>

        {loading ? (
          <p className="knowledge-muted">Checking keys…</p>
        ) : (
          <ul className="api-key-list">
            {statuses.map((status) => (
              <li key={status.slot.id} className="api-key-row">
                <div className="api-key-row-main">
                  <span className="api-key-name">{status.slot.name}</span>
                  <span
                    className={`api-key-badge ${
                      status.configured ? "configured" : "missing"
                    }`}
                  >
                    {statusLabel(status)}
                  </span>
                </div>
                <p className="api-key-desc">{status.slot.description}</p>
                {status.lockedByEnv && (
                  <p className="api-key-env-note">
                    <Lock size={12} aria-hidden />
                    Loaded from{" "}
                    <code>{status.slot.envHint}</code>
                    {status.envPath ? (
                      <>
                        {" "}
                        in <code className="api-key-env-path">{status.envPath}</code>
                      </>
                    ) : (
                      " in your project .env"
                    )}
                    . Remove or change it in that file to update.
                  </p>
                )}
                <div className="api-key-actions">
                  {!status.lockedByEnv && (
                    <>
                      <button
                        type="button"
                        className="btn secondary api-key-btn"
                        onClick={() => openEditor(status)}
                      >
                        {status.configured ? (
                          <>
                            <Pencil size={14} />
                            Replace
                          </>
                        ) : (
                          <>
                            <Plus size={14} />
                            Add key
                          </>
                        )}
                      </button>
                      {status.configured && status.source === "keychain" && (
                        <button
                          type="button"
                          className="btn secondary api-key-btn danger"
                          onClick={() => void handleRemove(status)}
                          disabled={removingId === status.slot.id}
                        >
                          <Trash2 size={14} />
                          Remove
                        </button>
                      )}
                    </>
                  )}
                </div>
              </li>
            ))}
          </ul>
        )}
      </div>

      <SecretKeyModal
        open={editorOpen}
        title={
          editing?.configured
            ? `Replace ${editing.slot.name}`
            : `Add ${editing?.slot.name ?? "API key"}`
        }
        description={
          editing?.configured
            ? "Enter a new key. The previous key will be overwritten and is not shown."
            : "Paste your API key below. It will be saved to the system keychain."
        }
        confirmLabel={editing?.configured ? "Replace key" : "Save key"}
        onClose={() => {
          setEditorOpen(false);
          setEditing(null);
        }}
        onSave={handleSaved}
      />
    </>
  );
}
