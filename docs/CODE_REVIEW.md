# Code Review — Omni Taskbar AI

**Reviewed:** 2026-05-31  
**Scope:** `src/`, `src-tauri/src/`, `tauri.conf.json`, `package.json`

## Summary

The app is a **Tauri v2 + React 19** macOS-oriented desktop overlay: tray icon, global shortcut (⌘⇧Space), OS context scraping (active window, clipboard), shell execution with a **Sudo Gate** HITL modal, keychain credential storage, and automation hooks (screenshot, click, type). The UI is polished; the **assistant chat path is still a demo** (timed `setTimeout` flow, no LLM API calls).

| Area | Status |
|------|--------|
| Frontend build (`npm run build`) | Pass |
| Rust build (`cargo build`) | Pass |
| Unit / integration tests | None defined |
| Real LLM orchestration | Not implemented |
| Security model (HITL) | Partial — see gaps below |

---

## Architecture (high level)

```
Tray + Global Shortcut (lib.rs)
        │
        ▼
React Overlay (App.tsx) ──invoke──► Tauri commands
        │                              ├── context.rs      (window, clipboard)
        │                              ├── shell_executor  (zsh/sh)
        │                              ├── privilege.rs    (safety + elevation)
        │                              ├── keychain.rs     (API key)
        │                              └── automation.rs   (enigo + OS fallbacks)
```

**Window behavior:** `main` webview starts hidden, frameless, transparent, always-on-top; macOS `ActivationPolicy::Accessory` (no dock icon).

---

## Strengths

1. **Clear module split** in Rust: context, privilege, shell, keychain, automation.
2. **Sudo Gate UX** — modal for dangerous commands with reason text and deny/approve.
3. **Command safety heuristics** — splits on `;`, `&`, `|`; flags `rm -rf`, `dd`, `chmod 777`, `sudo`, etc.
4. **macOS fallbacks** — AppleScript for elevation, window title, click/type when Enigo fails.
5. **Credentials** — `keyring` crate with service `com.omni.agent`; empty string when missing (no throw on first launch).
6. **Context polling** — 2.5s interval for app/title/clipboard/accessibility.

---

## Issues & gaps (priority order)

### P0 — Correctness

1. **No LLM integration**  
   `handleSubmit` ignores `apiType`, `selectedModel`, `apiKey`, and `ollamaUrl`. Every cloud prompt follows the same scripted path ending in `git --version`. Settings are UI-only.

2. **`execute_elevated_command` return shape mismatch**  
   Rust returns `Result<String, String>`. After Sudo Gate approval with `requires_admin`, the frontend still treats the result like `execute_shell_command` (`res.success`, `res.stdout`). Admin-approved commands will break in chat/CLI views.

3. **Sudo Gate resolver pattern**  
   `setSudoGateResolve(() => async (approved) => { ... })` stores a function that *returns* an async function; buttons call `sudoGateResolve(true)` expecting a direct async handler. Works only because the outer arrow returns the inner function — fragile and easy to break. Prefer `useRef` for the pending resolver.

4. **`is_safe` always `true`** (`privilege.rs`)  
   The field is unused for gating; only `requires_hitl_approval` drives the modal. Either enforce `is_safe` or remove it from the API.

### P1 — Security

5. **Shell injection surface**  
   Commands run via `zsh -c` / AppleScript string interpolation. Escaping in `execute_elevated_command` only handles `\` and `"`; complex payloads need structured argv or allowlists for production.

6. **`check_command_safety` bypass**  
   `execute_shell_command` does not call safety checks server-side. A modified or future client could invoke it directly. **Re-check in Rust before every execution.**

7. **`csp: null`** in `tauri.conf.json` — acceptable for local dev; tighten for release.

8. **HITL not enforced in `execute_elevated_command`**  
   Comment says frontend must confirm; backend does not verify approval token/timestamp.

### P2 — UX / polish

9. **`index.html` title** still "Tauri + React + Typescript".

10. **`accessibilityGranted` defaults to `true`** until first poll — brief false negative for permission banner.

11. **Automation commands unused in UI** — `capture_screen`, `simulate_click`, `simulate_typing` are registered but not invoked from React.

12. **README** was still the generic Tauri template (see root `README.md` update).

### P3 — Engineering

13. **No tests** — safety parser and shell JSON responses are good first candidates.

14. **`any` types** in `App.tsx` (`windowContext`, `cmdRes`) — add shared TS types matching Rust `Serialize` structs.

15. **Clippy** failed in this environment with a stale permissions path under a different folder (`Desktop AI Project Folder/src-tauri/...`); `cargo build` succeeded. Clean `target/` if build scripts point at wrong caches.

---

## Suggested next steps

1. Wire cloud/local LLM calls (or a single Rust `orchestrate_prompt` command) using stored credentials.
2. Normalize command results: one `CommandResult` struct for shell and elevated paths.
3. Call `check_command_safety` inside `execute_shell_command` (and optionally require HITL token for elevated).
4. Add unit tests for `check_command_safety` edge cases (`rm -rf /`, chained commands).
5. Fix Sudo Gate with `useRef<((approved: boolean) => void) | null>`.
6. Document macOS permissions (Accessibility, Screen Recording for capture) in `docs/MACOS_PERMISSIONS.md`.

---

## File reference

| File | Role |
|------|------|
| `src/App.tsx` | UI, chat demo, Sudo Gate, settings |
| `src-tauri/src/lib.rs` | App setup, tray, shortcuts, command registry |
| `src-tauri/src/privilege.rs` | Safety + elevation |
| `src-tauri/src/shell_executor.rs` | Non-privileged shell |
| `src-tauri/src/context.rs` | Window + clipboard |
| `src-tauri/src/keychain.rs` | Secrets |
| `src-tauri/src/automation.rs` | Screenshot + input simulation |
