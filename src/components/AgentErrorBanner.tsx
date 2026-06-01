import { useState } from "react";
import { AlertCircle, ChevronDown, ChevronUp } from "lucide-react";
import type { AgentErrorPayload } from "../lib/errors";

interface AgentErrorBannerProps {
  payload: AgentErrorPayload;
  className?: string;
}

export function AgentErrorBanner({ payload, className = "" }: AgentErrorBannerProps) {
  const [expanded, setExpanded] = useState(false);
  const hasDetail = Boolean(payload.detail?.trim());

  return (
    <div className={`agent-error-banner ${className}`.trim()} role="alert">
      <div className="agent-error-banner-head">
        <AlertCircle size={14} aria-hidden />
        <span className="agent-error-banner-message">{payload.message}</span>
        {hasDetail && (
          <button
            type="button"
            className="agent-error-expand"
            onClick={() => setExpanded((e) => !e)}
            aria-expanded={expanded}
          >
            {expanded ? <ChevronUp size={14} /> : <ChevronDown size={14} />}
            Details
          </button>
        )}
      </div>
      {payload.hint && <p className="agent-error-hint">{payload.hint}</p>}
      {expanded && payload.detail && (
        <pre className="agent-error-detail">{payload.detail}</pre>
      )}
      <span className="agent-error-code">{payload.code}</span>
    </div>
  );
}
