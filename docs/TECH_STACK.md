# Technology Stack — Gnomad Desktop Assistant

**Document owner:** Engineering  
**Version:** 0.1.0-alpha  
**Last updated:** June 2026

---

## Executive summary

Gnomad is a **native desktop application** built on **Tauri v2**, pairing a **React 19 + TypeScript** presentation layer with a **Rust** systems backend. The architecture deliberately keeps privileged operations (shell, filesystem, secrets, OS integration) on the Rust side while the webview handles UX, chat orchestration, and LLM client logic. This split yields a small install footprint relative to Electron, cross-platform parity from a single UI codebase, and a security boundary that is easier to reason about than a pure browser-based assistant.

---

## Stack overview

| Layer | Technology | Role |
|-------|------------|------|
| **Shell / runtime** | Tauri 2.x | Native windowing, tray, menus, IPC, bundling |
| **UI** | React 19, TypeScript 5.8, Vite 7 | Component model, build pipeline, HMR in dev |
| **Systems backend** | Rust 2021 edition | Commands, PTY shell, FS agent, keychain, platform APIs |
| **Styling** | CSS custom properties (design tokens) | Gemini-inspired theming; no CSS-in-JS framework |
| **Icons** | Lucide React | Consistent stroke icons |
| **HTTP (Rust)** | reqwest + rustls | Server-side LLM proxy paths where applicable |
| **Secrets** | keyring crate | OS-native credential storage |
| **PTY** | portable-pty | Persistent shell sessions with cwd tracking |
| **CI/CD** | GitHub Actions + tauri-action | Matrix builds: macOS, Ubuntu 22.04, Windows |
| **Frontend tests** | Vitest 3 | `parseInvokeError` and error formatting |
| **Terminal UI** | @xterm/xterm | Live PTY stream + command card replay |

---

## Frontend (presentation layer)

### React 19 + TypeScript

The UI is a single-page application compiled by Vite and embedded in Tauri’s WebView. TypeScript enforces contracts between the frontend and Tauri `invoke` payloads (messages, safety checks, platform info, agent settings).

**Key frontend modules:**

| Module | Responsibility |
|--------|----------------|
| `App.tsx` | Thin shell: providers, overlays, gate modals wiring |
| `hooks/useAgentExecution.ts` | Shell cwd, Sudo/Path gates, command execution |
| `hooks/useChatSubmit.ts` | Submit orchestration, agent loop entry |
| `components/ChatView.tsx` | Messages, composer, context footer |
| `components/LiveTerminal.tsx` | xterm.js PTY panel |
| `lib/hitlToken.ts` / `lib/pathToken.ts` | Mint signed approval tokens after gates |
| `lib/embeddedLlm.ts` | Embedded GGUF status helpers |
| `lib/updater.ts` | In-app update check/install |
| `lib/errors.ts` | Parse structured `GnomadError` JSON from invoke |

### Design system

Visual language follows a **Gemini-inspired “Neural Expressive”** pattern: pill composer, centered welcome state, gradient mesh backgrounds, dual provider/model selectors, and near-black dark theme (`#0a0a0a`). Theme tokens live in `src/index.css` as CSS variables under `[data-theme="light"]` and `[data-theme="dark"]`, enabling consistent surfaces without a runtime CSS framework.

### State and persistence

- **Ephemeral UI state:** React `useState` / `useRef` (chat messages, modals, thinking state).
- **Preferences:** `localStorage` for theme, sidebar layout, provider/model selection, onboarding flag.
- **Secrets:** Never stored in `localStorage`; routed to OS keychain via Rust (`keyring`).
- **Chat history & knowledge:** App data directory (`gnomad/`) managed from Rust for path safety.

---

## Backend (systems layer)

### Rust module map

```
src-tauri/src/
├── lib.rs              # App bootstrap, tray, shortcuts, command registry
├── platform.rs         # OS capability matrix exposed to UI
├── window_manager.rs   # Panel / floating / windowed / fullscreen placement
├── context.rs          # Active window + clipboard (per-OS implementations)
├── keychain.rs         # Credential slots (API keys, Ollama URL)
├── env_config.rs       # .env loading for dev/CI
├── llm.rs              # Rust-side LLM helpers where needed
├── shell_executor.rs   # One-shot shell execution
├── shell_session.rs    # PTY-backed persistent shell + interrupt
├── error.rs            # GnomadError JSON payloads
├── hitl_token.rs       # Cryptographic shell approval tokens
├── path_token.rs       # Cryptographic path approval tokens
├── local_inference.rs  # In-process GGUF (optional embedded-llm)
├── shell_sandbox.rs    # YOLO micro-sandbox (macOS/Linux)
├── updater.rs          # Tauri updater commands
├── privilege.rs        # Safety heuristics, Sudo Gate signals
├── agent_runtime.rs    # Tool execution loop (shell_run, fs_*)
├── agent_fs.rs         # Workspace-scoped filesystem operations
├── agent_settings.rs   # Trust mode, workspace root, planner config
├── agent_audit.rs      # JSONL audit log
├── command_planner.rs  # Local GGUF or Ollama command planner
├── knowledge.rs        # Knowledge base indexing on disk
├── chat_history.rs     # Session store
├── attachments.rs      # Staged file handling
├── automation.rs       # Screenshot / input simulation (Enigo + OS fallbacks)
└── menu_shell.rs       # Native File/Edit/View/Window/Help menus
```

### Why Rust for the backend

1. **Process and PTY control** — Shell sessions need reliable spawn, signal handling (`Ctrl+C` interrupt), and cwd propagation; Rust’s ownership model fits long-lived session state guarded by `Mutex`.
2. **Cross-platform OS APIs** — Window titles, clipboard, elevation, and tray placement differ per OS; `cfg(target_os = ...)` blocks isolate platform code without duplicating the React UI.
3. **Security-sensitive paths** — Command safety checks and filesystem sandboxing run server-side so a modified client cannot bypass gates by skipping UI modals.
4. **Distribution** — Tauri bundles produce `.app`, `.dmg`, `.deb`, `.rpm`, AppImage, `.msi`, and NSIS installers from one codebase.

### Tauri plugins

| Plugin | Use |
|--------|-----|
| `tauri-plugin-global-shortcut` | Toggle visibility (⌘⇧Space / Ctrl+Shift+Space) |
| `tauri-plugin-opener` | Open URLs (e.g. Gnomad Studio site) |
| `tauri-plugin-dialog` | File/folder pickers for knowledge and workspace |
| `tauri-plugin-updater` | Signed in-app updates (stable/beta channels) |

Capabilities are declared in `src-tauri/capabilities/` (`default.json`, `desktop.json`) following Tauri v2’s permission model.

---

## AI / LLM integration

| Mode | Provider | Configuration |
|------|----------|----------------|
| **Cloud** | DeepSeek API | API key in keychain or `DeepSeek_API_KEY` in `.env` |
| **Local** | Ollama HTTP **or** embedded GGUF (`embedded-llm` build) | Base URL in keychain; or GGUF path in Agent access |

**Cloud agent path:** `chatCompletion` + `runAgentLoop` issue structured tool calls (`shell_run`, filesystem tools, `workspace_info`) with up to **10 steps** per user message.

**Local path:** Models may emit `<gnomad-run>` command tags; optional **command planner** (small Ollama model) recovers when the main model returns prose instead of a valid command.

**Context injection:** Active app, window title, clipboard snippet, and trimmed knowledge bundle are appended to prompts so responses are grounded in the user’s current work.

---

## Security model (high level)

| Control | Mechanism |
|---------|-----------|
| **Sudo Gate** | Modal HITL + **HMAC approval tokens** for shell (`hitl_token.rs`) |
| **Path Gate** | Modal + **HMAC path tokens** for out-of-workspace FS (`path_token.rs`) |
| **Trust modes** | Standard (workspace-scoped) vs YOLO (broader FS; optional shell sandbox) |
| **Structured errors** | JSON `AgentErrorPayload` on invoke failures (`error.rs`) |
| **Audit log** | `{app_data}/gnomad/agent-audit.jsonl` |
| **Secrets** | OS keychain; keys never re-displayed after save |

Shell safety heuristics flag patterns such as `rm -rf`, `sudo`, `chmod 777`, and chained operators before execution. Unsigned `hitl_approved` / `path_approved` bypass attempts are rejected server-side.

---

## Build & release engineering

| Output | Platform |
|--------|----------|
| `.app` / `.dmg` | macOS (primary); menu-bar accessory mode |
| `.deb`, `.rpm`, AppImage | Linux x86_64 (CI); optional `xdotool` / `wl-paste` for context |
| `.msi`, NSIS | Windows |

**CI:** `.github/workflows/build.yml` runs a three-OS matrix on push to `main`/`master` (`npm run test`, `npm run build`, `cargo test`). Release artifacts attach to version tags via `release.yml` with updater JSON.

**Dev loop:** `npm run tauri dev` — Vite on port 1420, Rust debug binary, hot reload for frontend. Optional: `npm run tauri:dev:embedded` for GGUF.

---

## Dependencies worth noting

**Intentionally omitted:** Electron, Next.js, heavy state libraries (Redux), CSS frameworks (Tailwind). The stack stays lean for a portfolio-grade native assistant: one UI bundle, one Rust binary, OS-native chrome where possible.

**Alpha caveats:** CSP is relaxed (`null`) for local asset loading; Snap/Flatpak not shipped; ARM Linux requires local build. See `CHANGELOG.md` for release-scoped limitations.

---

## Related documents

| Document | Contents |
|----------|----------|
| [`BUILD.md`](BUILD.md) | Product build narrative and feature delivery phases |
| [`BUILD_PLATFORMS.md`](BUILD_PLATFORMS.md) | Per-OS build commands |
| [`WAVE_B_ROADMAP.md`](WAVE_B_ROADMAP.md) | Advanced systems (HITL, GGUF, xterm, sandbox) |
| [`UPDATER.md`](UPDATER.md) | Auto-update signing keys and channels |
| [`CROSS_PLATFORM_CHECKLIST.md`](CROSS_PLATFORM_CHECKLIST.md) | Dev checklist when adding UI/features (mac → Win/Linux) |
| [`QA_CHECKLIST.md`](QA_CHECKLIST.md) | Release QA matrices |

Built with ❤️ by [Gnomad Studio](https://gnomadstudio.org) 🦙
