import { describe, expect, it } from "vitest";
import {
  executionFailedLabel,
  formatErrorForUser,
  parseInvokeError,
} from "./errors";

describe("parseInvokeError", () => {
  it("parses JSON AgentErrorPayload from Error.message", () => {
    const payload = {
      code: "path_policy",
      message: "Path is outside workspace.",
      detail: "/etc/hosts",
      hint: "Approve access once.",
      retryable: false,
    };
    const err = new Error(JSON.stringify(payload));
    expect(parseInvokeError(err)).toEqual(payload);
  });

  it("parses JSON string directly", () => {
    const raw = JSON.stringify({
      code: "llm",
      message: "Ollama request failed.",
      retryable: true,
    });
    const parsed = parseInvokeError(raw);
    expect(parsed?.code).toBe("llm");
    expect(parsed?.retryable).toBe(true);
  });

  it("returns null for plain string errors", () => {
    expect(parseInvokeError(new Error("something broke"))).toBeNull();
    expect(parseInvokeError("not json")).toBeNull();
    expect(parseInvokeError(null)).toBeNull();
  });

  it("returns null when JSON lacks required fields", () => {
    expect(parseInvokeError(JSON.stringify({ foo: "bar" }))).toBeNull();
  });
});

describe("formatErrorForUser", () => {
  it("includes hint when present", () => {
    const err = new Error(
      JSON.stringify({
        code: "safety_blocked",
        message: "Blocked.",
        hint: "Use Sudo Gate.",
        retryable: false,
      })
    );
    expect(formatErrorForUser(err)).toBe("Blocked. Use Sudo Gate.");
  });

  it("falls back for non-JSON errors", () => {
    expect(formatErrorForUser(new Error("boom"), "Failed")).toBe("Failed: boom");
  });
});

describe("executionFailedLabel", () => {
  it("uses payload message when available", () => {
    const err = new Error(
      JSON.stringify({
        code: "shell_validation",
        message: "Not a valid shell command.",
        retryable: false,
      })
    );
    expect(executionFailedLabel(err)).toBe("Not a valid shell command.");
  });
});
