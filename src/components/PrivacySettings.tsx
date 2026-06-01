import { useState } from "react";
import { FileWarning, Mic } from "lucide-react";
import {
  getErrorLogEnabled,
  getVoiceInputEnabled,
  setErrorLogEnabled,
  setVoiceInputEnabled,
} from "../lib/preferences";
import { isVoiceInputSupported } from "../lib/voiceInput";

export function PrivacySettings() {
  const [errorLog, setErrorLog] = useState(getErrorLogEnabled());
  const [voiceInput, setVoiceInput] = useState(getVoiceInputEnabled());
  const voiceSupported = isVoiceInputSupported();

  const onToggle = (enabled: boolean) => {
    setErrorLog(enabled);
    setErrorLogEnabled(enabled);
  };

  const onVoiceToggle = (enabled: boolean) => {
    setVoiceInput(enabled);
    setVoiceInputEnabled(enabled);
  };

  return (
    <div className="settings-section">
      <h4 className="section-title">
        <FileWarning size={16} />
        Diagnostics
      </h4>
      <p className="knowledge-muted">
        Optional local-only logs stored in app data. Nothing is sent to Gnomad or third parties.
      </p>
      <label className="agent-trust-option">
        <input
          type="checkbox"
          checked={errorLog}
          onChange={(e) => onToggle(e.target.checked)}
        />
        <span>
          <strong>Save agent errors locally</strong> — append structured errors to{" "}
          <code>error-log.jsonl</code> for debugging
        </span>
      </label>

      <h4 className="section-title" style={{ marginTop: 20 }}>
        <Mic size={16} />
        Voice input
      </h4>
      <p className="knowledge-muted">
        Push-to-talk dictation uses your browser/WebView speech engine (e.g. Apple or Google STT).
        Audio may be processed by the OS or browser vendor — not by Gnomad servers.
      </p>
      <label className="agent-trust-option">
        <input
          type="checkbox"
          checked={voiceInput}
          disabled={!voiceSupported}
          onChange={(e) => onVoiceToggle(e.target.checked)}
        />
        <span>
          <strong>Enable microphone dictation</strong> — mic button in the composer
          {!voiceSupported && " (not supported in this WebView)"}
        </span>
      </label>
    </div>
  );
}
