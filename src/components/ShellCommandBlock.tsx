import { useState } from "react";
import { AlertTriangle, CheckCircle2, Clock, Monitor, Terminal, XCircle } from "lucide-react";
import { LiveTerminal } from "./LiveTerminal";
import type { ShellRunState } from "../lib/shellSession";

interface ShellCommandBlockProps {
  command: string;
  success: boolean;
  stdout: string;
  stderr?: string;
  statusCode?: number;
  cwd?: string;
  state?: ShellRunState;
  message?: string;
  durationMs?: number;
}

function stateLabel(state?: ShellRunState): string {
  switch (state) {
    case "completed":
      return "Completed";
    case "failed":
      return "Failed";
    case "timeout":
      return "Timed out";
    case "stalled":
      return "Stalled";
    case "incomplete":
      return "Incomplete";
    default:
      return "";
  }
}

export function ShellCommandBlock({
  command,
  success,
  stdout,
  stderr,
  statusCode,
  cwd,
  state,
  message,
  durationMs,
}: ShellCommandBlockProps) {
  const [showLive, setShowLive] = useState(false);
  const output = (stderr?.trim() ? stderr : stdout).trim() || "(no output)";
  const blockClass =
    state === "completed" || (success && !state)
      ? "success"
      : state === "stalled" || state === "timeout"
        ? "warning"
        : "failed";

  const StatusIcon =
    state === "stalled" || state === "timeout"
      ? state === "timeout"
        ? Clock
        : AlertTriangle
      : success
        ? CheckCircle2
        : XCircle;

  return (
    <div className={`shell-command-block ${blockClass}`}>
      <div className="shell-command-header">
        <StatusIcon
          size={14}
          className={`shell-command-icon ${blockClass}`}
        />
        <Terminal size={14} aria-hidden />
        <code className="shell-command-text">{command}</code>
        {state && (
          <span className={`shell-command-state state-${state}`}>
            {stateLabel(state)}
          </span>
        )}
        {statusCode !== undefined && statusCode >= 0 && (
          <span className="shell-command-exit">exit {statusCode}</span>
        )}
        {durationMs !== undefined && durationMs > 0 && (
          <span className="shell-command-duration">
            {(durationMs / 1000).toFixed(1)}s
          </span>
        )}
      </div>
      {message && <p className="shell-command-message">{message}</p>}
      {cwd && (
        <div className="shell-command-cwd" title="Working directory">
          cwd: {cwd}
        </div>
      )}
      <div className="shell-command-output-row">
        <button
          type="button"
          className="btn-secondary btn-sm shell-live-toggle"
          onClick={() => setShowLive((v) => !v)}
          title="Toggle ANSI terminal view (replay uses summary text)"
        >
          <Monitor size={12} />
          {showLive ? "Hide terminal" : "Terminal view"}
        </button>
      </div>
      {showLive ? (
        <LiveTerminal
          active
          stream={false}
          initialText={output}
          className="shell-command-live"
        />
      ) : (
        <pre className="shell-command-output">{output}</pre>
      )}
    </div>
  );
}
