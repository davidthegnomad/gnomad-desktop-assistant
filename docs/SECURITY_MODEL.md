# Security Model — Gnomad Desktop Assistant

**Version:** 0.1.0-alpha  
**Classification:** Internal / portfolio  
**Last updated:** June 2026

---

## Scope

This document describes how Gnomad **reduces risk** when an LLM can run shell commands and access files. It is not a formal penetration test or certification.

**In scope:** Desktop app on user-controlled machine; cloud (DeepSeek) and local (Ollama) inference; agent tools; keychain storage.

**Out of scope (alpha):** Multi-tenant SaaS, supply-chain signing beyond platform defaults, enterprise MDM deployment guides.

---

## Threat summary

| Threat | Mitigation (alpha) | Residual risk |
|--------|-------------------|---------------|
| Model suggests destructive shell | `privilege` heuristics + **Sudo Gate** | User may approve; heuristics incomplete |
| Model reads sensitive files | **Workspace** + **Path Gate** (Standard mode) | YOLO mode expands access by user choice |
| Stolen API key | OS keychain; not logged in UI | Key exfiltration via other malware on host |
| Malicious attachment in chat | User explicitly attaches; sent to model | User social-engineered to attach secrets |
| IPC bypass (modified client) | Commands should re-validate in Rust | Not all paths audited to production bar |
| Prompt injection via clipboard/context | Context labeled; user visibility | Model may over-trust injected text |

---

## Trust modes

| Mode | Filesystem | Shell |
|------|------------|-------|
| **Standard** (default) | Read/write/search inside **workspace**; paths outside require **Path Gate** | Sudo Gate for flagged commands |
| **YOLO!** | Broader machine access (user opt-in) | Sudo Gate may still apply for destructive patterns |

Workspace defaults to `$HOME` until changed in **Settings → Agent access**.

---

## Human-in-the-loop gates

### Sudo Gate

Triggered when `check_command_safety` marks a command as requiring HITL (e.g. `sudo`, `rm -rf`, `chmod 777`, chained operators).

- UI shows **reason** text  
- **Deny** → command not executed  
- **Approve** → execution proceeds; macOS/Linux may prompt for administrator credentials  

### Path Gate

In **Standard** mode, filesystem tool calls targeting paths outside the workspace require explicit approval. After **Allow once**, the app issues a signed **path gate token** (same HMAC pattern as HITL) bound to the canonical path hash, scope (`read` / `write`), nonce, and expiry (~300s). Passing `path_approved: true` without a token is rejected.

Implementation: [`src-tauri/src/path_token.rs`](../src-tauri/src/path_token.rs), frontend [`src/lib/pathToken.ts`](../src/lib/pathToken.ts).

### Cryptographic approval tokens (Wave B1 — shipped)

When `check_command_safety` requires HITL:

1. After Sudo Gate **Approve**, the app calls `issue_hitl_approval_token` (Rust). The backend returns a short-lived **HMAC-SHA256** token bound to `SHA-256(normalized command)`, nonce, expiry (~120s), and scope (`shell_run` or `elevated`).
2. Signing secret is stored in the OS keychain (`hitl-hmac-secret`); not exposed to the webview.
3. `shell_session_run`, `agent_execute_tool` (`shell_run`), and `execute_elevated_command` require a valid token. Passing `hitl_approved: true` without a token returns `safety_blocked` (“Unsigned HITL approval is not accepted”).
4. Tokens are **single-use** (nonce burned in memory after verification).

Implementation: [`src-tauri/src/hitl_token.rs`](../src-tauri/src/hitl_token.rs), frontend [`src/lib/hitlToken.ts`](../src/lib/hitlToken.ts).

---

## Command safety (server-side)

`privilege.rs` analyzes command strings before execution:

- Splits on `;`, `&`, `|`
- Flags known-dangerous patterns
- Returns `requires_hitl_approval`, `requires_admin`, `danger_reason`

**Alpha gap:** All invoke paths must call safety checks; treat direct `execute_shell_command` without checks as a finding in review.

---

## Secrets handling

| Secret | Storage | UI behavior |
|--------|---------|-------------|
| DeepSeek API key | Keychain (+ optional `.env` in dev) | Never re-displayed after save |
| Ollama URL | Keychain | Editable in settings |
| Chat content | Local app data | Not encrypted at rest (alpha) |

`.env` keys show as **locked** in Settings; user edits file to rotate.

---

## Audit & observability

Agent actions append to:

`{app_data}/gnomad/agent-audit.jsonl`

Each line should include timestamp, action type, and outcome (exact schema in `agent_audit.rs`). Users can inspect this file for accountability.

---

## Network exposure

| Path | Endpoint | Data sent |
|------|----------|-----------|
| Cloud chat | DeepSeek API | Prompts, tool results, attachments user includes |
| Local chat | User-configured Ollama URL | Same, stays on network path to Ollama |
| Studio link | gnomadstudio.org | Normal HTTPS (user-initiated) |

No Gnomad Studio backend receives chat content in the alpha architecture.

---

## Platform permissions

| OS | Permission | Feature |
|----|------------|---------|
| macOS | Accessibility | Input automation fallbacks |
| macOS | Automation (System Events) | Window titles |
| macOS | Admin auth | Elevated shell |
| All | Tray / shortcuts | No extra privacy prompt |

See [`MACOS_PERMISSIONS.md`](MACOS_PERMISSIONS.md).

---

## Hardening roadmap (post-alpha)

1. Enforce `check_command_safety` on every shell invoke path  
2. HITL approval token validated in Rust (not UI-only)  
3. Tighten CSP for release builds  
4. Optional at-rest encryption for chat store  
5. Rust unit tests for safety parser edge cases  
6. Signed releases + notarization (macOS)  

---

## Related documents

- [`ARCHITECTURE.md`](ARCHITECTURE.md)  
- [`PRIVACY.md`](PRIVACY.md)  
- [`QA_CHECKLIST.md`](QA_CHECKLIST.md) — security spot checks  

Built with ❤️ by [Gnomad Studio](https://gnomadstudio.org) 🦙
