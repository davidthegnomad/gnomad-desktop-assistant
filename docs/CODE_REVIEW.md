# Code Review — Gnomad Desktop Assistant

**Reviewed:** 2026-05-31 (updated after hiring-feedback implementation)  
**Scope:** `src/`, `src-tauri/src/`, workflows, agent/security paths

## Summary

The app is a **Tauri v2 + React 19** desktop assistant: tray/panel, global shortcut, OS context (active window, clipboard), **cloud + local LLM** chat with an **agent tool loop**, persistent **PTY shell session**, filesystem agent tools, **Sudo Gate** / **Path Gate** HITL, and structured error payloads end-to-end.

| Area | Status |
|------|--------|
| Frontend build (`npm run build`) | Pass |
| Rust tests (`cargo test`) | Pass (error, privilege, shell_session) |
| LLM orchestration | Cloud `chat_completion_turn` + tools; local Ollama + `<gnomad-run>` fallback |
| Structured errors | `GnomadError` → JSON in invoke `Err(String)`; frontend `parseInvokeError` + `AgentErrorBanner` |
| App shell | `App.tsx` ~400 lines; hooks + `ChatView` / `SettingsPanel` / gate modals |
| Elevation hardening | Pre-flight injection blocks; Linux `pkexec` argv-only; macOS per-arg escaping |

---

## Architecture (high level)

```
Tray + Global Shortcut (lib.rs)
        │
        ▼
React App shell (App.tsx) ──hooks──► useAgentExecution, useChatSubmit, …
        │                              │
        ▼                              ▼
ChatView / SettingsPanel ──invoke──► Tauri commands
                                     ├── agent_runtime / agent_fs
                                     ├── shell_session (PTY)
                                     ├── privilege.rs (safety + elevation)
                                     ├── error.rs (GnomadError payloads)
                                     └── context, keychain, llm, …
```

---

## Agent error payload contract

Tauri commands still return `Result<T, String>`. During migration, error strings are **JSON** matching:

```json
{
  "code": "safety_blocked",
  "message": "Human-readable summary",
  "detail": "Optional technical detail",
  "hint": "Optional remediation",
  "retryable": false
}
```

Frontend: [`src/lib/errors.ts`](../src/lib/errors.ts) — `parseInvokeError`, `executionFailedLabel`, `formatErrorForUser`.  
UI: [`src/components/AgentErrorBanner.tsx`](../src/components/AgentErrorBanner.tsx) on messages with `errorPayload`.

Stable `code` values are covered by `error::tests::payload_codes_are_stable`.

---

## Strengths

1. **Module split** — Rust agent, shell, privilege, FS; React hooks mirror execution concerns.
2. **HITL** — Sudo Gate and Path Gate with explicit approve/deny.
3. **Defense in depth** — Server-side safety before shell; elevation rejects injection patterns.
4. **Cross-platform awareness** — `platformInfo` drives labels and capability flags.

---

## Remaining gaps (priority)

### P1 — Security / product

1. **Wave B error migration** — `llm.rs`, `command_planner.rs`, `chat_history.rs` still return plain strings in some paths.
2. **Windows elevation** — Structured `elevation_unsupported`; user must use elevated terminal for admin ops.
3. **Path Gate tokens** — ✓ Shipped — [`path_token.rs`](../src-tauri/src/path_token.rs); boolean IPC bypass rejected.

### P2 — Engineering

4. **Typed invoke errors** — Optional future: `Result<T, AgentErrorPayload>` at Tauri boundary once JSON-in-string is stable everywhere.
5. **Vitest** — ✓ `parseInvokeError` tests in CI (`npm run test`).
6. **GGUF planner + local chat** — In-process inference via optional `embedded-llm` feature; Ollama remains fallback.

---

## Suggested next steps

1. Generate updater signing keys per [UPDATER.md](UPDATER.md) and replace the placeholder `pubkey` in `tauri.conf.json`.
2. Prefer `agent_fs` + Path Gate over elevated shell for file writes.
3. Add integration test for cloud agent turn (mocked) if CI budget allows.

---

## File reference

| File | Role |
|------|------|
| `src/App.tsx` | Thin shell, providers wiring |
| `src/hooks/useAgentExecution.ts` | Shell cwd, gates, `executeCommandSafely` |
| `src/hooks/useChatSubmit.ts` | Submit orchestration, agent loop |
| `src/components/ChatView.tsx` | Messages, composer, context footer |
| `src-tauri/src/error.rs` | `GnomadError`, `AgentErrorPayload` |
| `src-tauri/src/privilege.rs` | Safety + elevation |
| `src-tauri/src/shell_session.rs` | PTY session + validation |
