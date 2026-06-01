export interface AgentErrorPayload {
  code: string;
  message: string;
  detail?: string;
  hint?: string;
  retryable: boolean;
}

export function parseInvokeError(err: unknown): AgentErrorPayload | null {
  const raw =
    err instanceof Error
      ? err.message
      : typeof err === "string"
        ? err
        : null;
  if (!raw) return null;
  const trimmed = raw.trim();
  if (!trimmed.startsWith("{")) return null;
  try {
    const parsed = JSON.parse(trimmed) as AgentErrorPayload;
    if (parsed && typeof parsed.code === "string" && typeof parsed.message === "string") {
      return parsed;
    }
  } catch {
    /* plain string error */
  }
  return null;
}

export function formatErrorForUser(
  err: unknown,
  fallbackPrefix = "Something went wrong"
): string {
  const payload = parseInvokeError(err);
  if (!payload) {
    const msg = err instanceof Error ? err.message : String(err);
    return `${fallbackPrefix}: ${msg}`;
  }
  if (payload.hint) {
    return `${payload.message} ${payload.hint}`;
  }
  return payload.message;
}

export function executionFailedLabel(err: unknown): string {
  const payload = parseInvokeError(err);
  if (payload) {
    return payload.message;
  }
  return `Execution failed: ${err instanceof Error ? err.message : String(err)}`;
}
