import { useEffect, useRef } from "react";
import {
  AlertCircle,
  ChevronRight,
  Clipboard,
  Cpu,
  RefreshCw,
  Sparkles,
  Terminal,
} from "lucide-react";
import { AgentErrorBanner } from "./AgentErrorBanner";
import {
  AttachFilesButton,
  AttachmentChipList,
} from "./ComposerAttachments";
import { GnomadLogo } from "./GnomadLogo";
import { LiveTerminal } from "./LiveTerminal";
import { ShellCommandBlock } from "./ShellCommandBlock";
import { StudioLink } from "./StudioLink";
import { APP_NAME } from "../lib/brand";
import type { ChatAttachment } from "../lib/attachments";
import type { LlmAvailability } from "../lib/models";
import type { ProviderMode } from "../lib/preferences";
import type { Message } from "../types/chat";

interface ChatViewProps {
  messages: Message[];
  activeSessionId: string | null;
  showWelcome: boolean;
  isTrayCompact: boolean;
  isThinking: boolean;
  thinkingText: string;
  onStopThinking: () => void;
  suggestionChips: string[];
  onSuggestionClick: (text: string) => void;
  input: string;
  onInputChange: (v: string) => void;
  onSubmit: (e: React.FormEvent) => void;
  onDirectCommand: () => void;
  pendingAttachments: ChatAttachment[];
  onAddAttachments: () => void;
  onRemoveAttachment: (id: string) => void;
  apiType: ProviderMode;
  llmAvailability: LlmAvailability;
  headerModelValue: string;
  onProviderChange: (mode: ProviderMode) => void;
  onModelChange: (model: string) => void;
  activeApp: string;
  activeTitle: string;
  clipboardSnippet: string;
  accessibilityGranted: boolean;
  onRequestPermissions: () => void;
}

export function ChatView({
  messages,
  activeSessionId,
  showWelcome,
  isTrayCompact,
  isThinking,
  thinkingText,
  onStopThinking,
  suggestionChips,
  onSuggestionClick,
  input,
  onInputChange,
  onSubmit,
  onDirectCommand,
  pendingAttachments,
  onAddAttachments,
  onRemoveAttachment,
  apiType,
  llmAvailability,
  headerModelValue,
  onProviderChange,
  onModelChange,
  activeApp,
  activeTitle,
  clipboardSnippet,
  accessibilityGranted,
  onRequestPermissions,
}: ChatViewProps) {
  const scrollRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    scrollRef.current?.scrollIntoView({ behavior: "smooth" });
  }, [messages, isThinking, thinkingText]);

  const cloudModelOptions = llmAvailability.cloudModels;
  const localModelOptions = llmAvailability.localModels;

  return (
    <>
      <div className="chat-scroll">
        {showWelcome && (
          <div className="chat-empty-state">
            <h1 className="welcome-greeting">Welcome!</h1>
            <p className="welcome-sub">What adventure should we start?</p>
          </div>
        )}

        {messages.map((msg, idx) =>
          !showWelcome || msg.role === "user" || idx > 0 ? (
            <div key={`${activeSessionId ?? "chat"}-${idx}`} className={`msg ${msg.role}`}>
              <span className="msg-header">
                {msg.role === "user" ? "You" : `${APP_NAME} 🍄`}
              </span>
              {msg.attachments && msg.attachments.length > 0 && (
                <div className="msg-attachments">
                  {msg.attachments.map((a) => (
                    <span key={a.id} className="msg-attachment-pill" title={a.name}>
                      {a.kind === "image" ? "🖼" : "📎"} {a.name}
                    </span>
                  ))}
                </div>
              )}
              {msg.errorPayload ? (
                <AgentErrorBanner payload={msg.errorPayload} />
              ) : msg.commandExecuted && msg.commandResult ? (
                <ShellCommandBlock
                  command={msg.commandExecuted}
                  success={msg.commandResult.success}
                  stdout={msg.commandResult.stdout}
                  stderr={msg.commandResult.stderr}
                  statusCode={msg.commandResult.status_code}
                  cwd={msg.commandResult.cwd}
                  state={msg.commandResult.state}
                  message={msg.commandResult.message}
                  durationMs={msg.commandResult.duration_ms}
                />
              ) : (
                <span className="msg-text">{msg.text}</span>
              )}
            </div>
          ) : null
        )}

        {isThinking && (
          <div className="msg assistant">
            <span className="msg-header thinking-header">
              <RefreshCw size={12} className="spinning" />
              Thinking
              <button
                type="button"
                className="btn-secondary btn-sm agent-stop-btn"
                onClick={onStopThinking}
              >
                Stop
              </button>
            </span>
                <div className="thinking-block">{thinkingText}</div>
                <LiveTerminal active stream />
              </div>
            )}
        <div ref={scrollRef} />
      </div>

      <form className="prompt-footer" onSubmit={onSubmit}>
        {showWelcome && !isTrayCompact && (
          <div className="suggestion-chips" role="group" aria-label="Suggestions">
            {suggestionChips.map((text) => (
              <button
                key={text}
                type="button"
                className="suggestion-chip"
                disabled={isThinking}
                onClick={() => onSuggestionClick(text)}
              >
                {text}
              </button>
            ))}
          </div>
        )}

        <div className="gemini-composer">
          <AttachmentChipList
            attachments={pendingAttachments}
            onRemove={onRemoveAttachment}
            disabled={isThinking}
          />
          <input
            className="composer-input"
            value={input}
            onChange={(e) => onInputChange(e.target.value)}
            placeholder={
              apiType === "local"
                ? "Ask Gnomad or type a shell command…"
                : "Ask Gnomad anything…"
            }
            disabled={isThinking}
          />
          <div className="composer-bar">
            <div className="composer-bar-left">
              <AttachFilesButton
                onClick={onAddAttachments}
                disabled={isThinking}
                showLabel={!isTrayCompact && pendingAttachments.length === 0}
              />
              {apiType === "local" && !isTrayCompact && (
                <button
                  type="button"
                  className="composer-tool-btn"
                  onClick={onDirectCommand}
                  disabled={isThinking || !input.trim()}
                  title="Run as shell command"
                >
                  <Terminal size={16} />
                </button>
              )}
            </div>
            <div className="composer-bar-right">
              {!isTrayCompact && (
                <>
                  <select
                    className="composer-select"
                    value={apiType}
                    onChange={(e) => onProviderChange(e.target.value as ProviderMode)}
                    aria-label="Provider"
                  >
                    <option value="cloud" disabled={!llmAvailability.cloudConfigured}>
                      Cloud
                    </option>
                    <option value="local" disabled={!llmAvailability.localConfigured}>
                      Local
                    </option>
                  </select>
                  <select
                    className="composer-select composer-select-model"
                    value={headerModelValue}
                    onChange={(e) => onModelChange(e.target.value)}
                    aria-label="Model"
                    disabled={
                      apiType === "cloud"
                        ? cloudModelOptions.length === 0
                        : localModelOptions.length === 0
                    }
                  >
                    {apiType === "cloud" ? (
                      cloudModelOptions.length > 0 ? (
                        cloudModelOptions.map((m) => (
                          <option key={m.value} value={m.value}>
                            {m.label}
                          </option>
                        ))
                      ) : (
                        <option value="">Add API key</option>
                      )
                    ) : localModelOptions.length > 0 ? (
                      localModelOptions.map((m) => (
                        <option key={m.value} value={m.value}>
                          {m.label}
                        </option>
                      ))
                    ) : (
                      <option value="">Set Ollama URL</option>
                    )}
                  </select>
                </>
              )}
              <button
                type="submit"
                className="composer-send"
                disabled={isThinking || (!input.trim() && pendingAttachments.length === 0)}
                aria-label="Send"
              >
                <Sparkles size={18} />
              </button>
            </div>
          </div>
        </div>
      </form>

      <div
        className={`context-footer ${isTrayCompact ? "context-footer-compact" : ""}`}
        aria-label="Active context"
      >
        {!isTrayCompact && (
          <div className="context-inline">
            <div className="context-tag" title="Active app">
              <Cpu size={12} />
              <span>{activeApp}</span>
            </div>
            <div className="context-tag" title="Window title">
              <ChevronRight size={12} style={{ color: "var(--color-muted)" }} />
              <span>{activeTitle}</span>
            </div>
            <div className="context-tag" title="Clipboard">
              <Clipboard size={12} />
              <span className="context-tag-mono">{clipboardSnippet}</span>
            </div>
            {!accessibilityGranted && (
              <button type="button" className="permission-banner" onClick={onRequestPermissions}>
                <AlertCircle size={12} />
                <span>Accessibility needed</span>
              </button>
            )}
          </div>
        )}
        <div className="context-footer-brand">
          <GnomadLogo size="sm" />
          <StudioLink />
        </div>
      </div>
    </>
  );
}
