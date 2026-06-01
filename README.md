# Gnomad Desktop Assistant

[![Beta](https://img.shields.io/badge/status-beta-7c6cf0)](https://davidthegnomad.github.io/gnomad-desktop-assistant/)
[![Version](https://img.shields.io/badge/version-0.2.0--beta.1-7c6cf0)](https://github.com/davidthegnomad/gnomad-desktop-assistant/releases)
[![Platforms](https://img.shields.io/badge/platforms-macOS%20%7C%20Windows%20%7C%20Linux-333)](docs/BUILD_PLATFORMS.md)

**Gnomad** is a cross-platform desktop AI assistant that integrates with the operating system—system tray, global shortcut, live window and clipboard context—and executes **real** shell and filesystem work under explicit user approval, not simulated chat output.

Built with ❤️ by [Gnomad Studio](https://gnomadstudio.org) 🦙

**Live project site:** [davidthegnomad.github.io/gnomad-desktop-assistant](https://davidthegnomad.github.io/gnomad-desktop-assistant/)

---

## Why this exists

Desktop knowledge work is fragmented: chat in a browser tab, terminal in another, files elsewhere. Gnomad unifies **conversation**, **OS awareness**, and **action** in a native shell that stays out of the way until invoked—then anchors next to the menu bar or tray with a focused, Gemini-inspired interface.

The alpha demonstrates end-to-end delivery: multi-platform installers, CI/CD, credential hygiene, agent tooling with human-in-the-loop gates, and documentation suitable for technical review.

---

## Capabilities (v0.2.0-beta.1)

| Area | What you get |
|------|----------------|
| **Access** | Menu bar / system tray, global shortcut, four window modes (panel, pop-out, windowed, fullscreen) |
| **Intelligence** | DeepSeek (cloud default), **OpenAI-compatible** endpoints, Ollama (local), optional in-process **GGUF** |
| **Agent** | Multi-step tool loop (shell + filesystem), command planner, persistent PTY shell |
| **Safety** | **Cryptographic** Sudo Gate (HITL) and Path Gate tokens; Standard vs YOLO trust; optional YOLO shell sandbox |
| **Terminal** | xterm.js live stream + replay on command cards |
| **Context** | Active application, window title, clipboard snippet in the footer |
| **Memory** | Chat history on disk; knowledge library (skills, agents, uploads); **starter skill packs** |
| **Voice** | Opt-in push-to-talk dictation (Web Speech API) |
| **Updates** | In-app check (stable/beta) via Tauri updater — see [`docs/UPDATER.md`](docs/UPDATER.md) |
| **Platform** | macOS (primary), Windows, Linux (`.deb`, `.rpm`, AppImage; **ARM64** in CI); optional Flatpak/Snap |
| **Accessibility** | Keyboard shortcuts, focus traps, skip link — [`docs/ACCESSIBILITY.md`](docs/ACCESSIBILITY.md) |

---

## Platforms

| Platform | Status | Install | Notes |
|----------|--------|---------|-------|
| **macOS** | Primary | [Releases](https://github.com/davidthegnomad/gnomad-desktop-assistant/releases) | Menu-bar accessory; notarization guide for enterprise |
| **Windows** | Supported | [Releases](https://github.com/davidthegnomad/gnomad-desktop-assistant/releases) | System tray; `Ctrl+Shift+Space` |
| **Linux** | Supported | [Releases](https://github.com/davidthegnomad/gnomad-desktop-assistant/releases) (`.deb`, `.rpm`, AppImage, ARM64) | See [Linux packages](docs/LINUX_PACKAGES.md) |

Per-OS build instructions: [`docs/BUILD_PLATFORMS.md`](docs/BUILD_PLATFORMS.md)  
**Adding features or UI?** Use the [`docs/CROSS_PLATFORM_CHECKLIST.md`](docs/CROSS_PLATFORM_CHECKLIST.md) so changes are verified on macOS, Windows, and Linux.

---

## Quick start

### End users

1. Download the installer for your OS from [Releases](https://github.com/davidthegnomad/gnomad-desktop-assistant/releases) or the [project site](https://davidthegnomad.github.io/gnomad-desktop-assistant/).
2. Launch Gnomad; complete the setup wizard (cloud API key **or** local Ollama URL).
3. Press **⌘⇧Space** (macOS) or **Ctrl+Shift+Space** (Windows/Linux) to show the assistant.
4. Read the full manual: [`docs/USER_GUIDE.html`](docs/USER_GUIDE.html) (recommended) or [`docs/USER_GUIDE.txt`](docs/USER_GUIDE.txt).

### Developers

**Prerequisites:** Node.js LTS, Rust stable, [Tauri v2 platform deps](https://v2.tauri.app/start/prerequisites/)

```bash
git clone https://github.com/davidthegnomad/gnomad-desktop-assistant.git
cd gnomad-desktop-assistant
npm ci
cp .env.example .env   # optional: DeepSeek_API_KEY for local dev
npm run tauri dev
npm run test              # Vitest (error parsing)
cd src-tauri && cargo test
```

**Embedded GGUF (optional, in-process local LLM):**

```bash
npm run tauri:dev:embedded
```

**Production build:**

```bash
npm run tauri:build:mac     # macOS
npm run tauri:build:linux   # Linux (on Linux host)
npm run tauri:build:win     # Windows (on Windows host)
```

Never commit `.env` or API keys.

---

## Architecture (summary)

```
┌─────────────────────────────────────────────────────────┐
│  React 19 UI (chat, settings, knowledge, gates)         │
└───────────────────────────┬─────────────────────────────┘
                            │ Tauri IPC (invoke / events)
┌───────────────────────────▼─────────────────────────────┐
│  Rust: tray, window modes, shell PTY, agent FS,         │
│        keychain, context, safety, audit, chat store      │
└───────────────────────────┬─────────────────────────────┘
                            │
         macOS / Windows / Linux native APIs
```

Deep dive: [`docs/TECH_STACK.md`](docs/TECH_STACK.md) · Delivery narrative: [`docs/BUILD.md`](docs/BUILD.md)

---

## Documentation index

All docs ship as **Markdown** (source), **HTML** (browser), and **TXT** (Notepad/Word). Full index: [`docs/DOCS_INDEX.md`](docs/DOCS_INDEX.md). Regenerate: `npm run docs:export`.

| Document | MD | HTML | TXT |
|----------|----|------|-----|
| **All docs (index)** | [docs/DOCS_INDEX.md](docs/DOCS_INDEX.md) | [docs/DOCS_INDEX.html](docs/DOCS_INDEX.html) | [docs/DOCS_INDEX.txt](docs/DOCS_INDEX.txt) |
| **Project site** | — | [docs/index.html](docs/index.html) | — |
| User Guide | [docs/USER_GUIDE.md](docs/USER_GUIDE.md) | [docs/USER_GUIDE.html](docs/USER_GUIDE.html) | [docs/USER_GUIDE.txt](docs/USER_GUIDE.txt) |
| Tech Stack | [docs/TECH_STACK.md](docs/TECH_STACK.md) | [docs/TECH_STACK.html](docs/TECH_STACK.html) | [docs/TECH_STACK.txt](docs/TECH_STACK.txt) |
| Build Narrative | [docs/BUILD.md](docs/BUILD.md) | [docs/BUILD.html](docs/BUILD.html) | [docs/BUILD.txt](docs/BUILD.txt) |
| Architecture | [docs/ARCHITECTURE.md](docs/ARCHITECTURE.md) | [docs/ARCHITECTURE.html](docs/ARCHITECTURE.html) | [docs/ARCHITECTURE.txt](docs/ARCHITECTURE.txt) |
| Security Model | [docs/SECURITY_MODEL.md](docs/SECURITY_MODEL.md) | [docs/SECURITY_MODEL.html](docs/SECURITY_MODEL.html) | [docs/SECURITY_MODEL.txt](docs/SECURITY_MODEL.txt) |
| Wave B Roadmap | [docs/WAVE_B_ROADMAP.md](docs/WAVE_B_ROADMAP.md) | [docs/WAVE_B_ROADMAP.html](docs/WAVE_B_ROADMAP.html) | [docs/WAVE_B_ROADMAP.txt](docs/WAVE_B_ROADMAP.txt) |
| Auto-updater | [docs/UPDATER.md](docs/UPDATER.md) | [docs/UPDATER.html](docs/UPDATER.html) | [docs/UPDATER.txt](docs/UPDATER.txt) |
| Release runbook | [docs/RELEASE_RUNBOOK.md](docs/RELEASE_RUNBOOK.md) | [docs/RELEASE_RUNBOOK.html](docs/RELEASE_RUNBOOK.html) | [docs/RELEASE_RUNBOOK.txt](docs/RELEASE_RUNBOOK.txt) |
| Troubleshooting | [docs/TROUBLESHOOTING.md](docs/TROUBLESHOOTING.md) | [docs/TROUBLESHOOTING.html](docs/TROUBLESHOOTING.html) | [docs/TROUBLESHOOTING.txt](docs/TROUBLESHOOTING.txt) |
| Privacy | [docs/PRIVACY.md](docs/PRIVACY.md) | [docs/PRIVACY.html](docs/PRIVACY.html) | [docs/PRIVACY.txt](docs/PRIVACY.txt) |
| Roadmap | [docs/ROADMAP.md](docs/ROADMAP.md) | [docs/ROADMAP.html](docs/ROADMAP.html) | [docs/ROADMAP.txt](docs/ROADMAP.txt) |
| Demo Script | [docs/DEMO_SCRIPT.md](docs/DEMO_SCRIPT.md) | [docs/DEMO_SCRIPT.html](docs/DEMO_SCRIPT.html) | [docs/DEMO_SCRIPT.txt](docs/DEMO_SCRIPT.txt) |
| QA Checklists | [docs/QA_CHECKLIST.md](docs/QA_CHECKLIST.md) | [docs/QA_CHECKLIST.html](docs/QA_CHECKLIST.html) | [docs/QA_CHECKLIST.txt](docs/QA_CHECKLIST.txt) |
| Build Platforms | [docs/BUILD_PLATFORMS.md](docs/BUILD_PLATFORMS.md) | [docs/BUILD_PLATFORMS.html](docs/BUILD_PLATFORMS.html) | [docs/BUILD_PLATFORMS.txt](docs/BUILD_PLATFORMS.txt) |
| Changelog | [CHANGELOG.md](CHANGELOG.md) | [CHANGELOG.html](CHANGELOG.html) | [CHANGELOG.txt](CHANGELOG.txt) |

---

## Security & privacy (alpha)

- API keys are stored in the **OS keychain**, not in chat logs or `localStorage`.
- Destructive or privileged shell operations require **Sudo Gate** approval with **signed HITL tokens** (unsigned IPC bypass rejected).
- Filesystem access outside the workspace requires **Path Gate** approval with **signed path tokens** (Standard mode).
- Optional **YOLO shell sandbox** (macOS/Linux experimental) limits network and writes when enabled.
- Agent actions are appended to a local **audit log** under application data.

Alpha software: review [`CHANGELOG.md`](CHANGELOG.md) for known limitations before production use.

---

## CI / releases

GitHub Actions builds **macOS**, **Linux x86_64**, **Linux ARM64**, and **Windows** on every push to `main` / `master`. Tagged `v*` releases attach installers to [GitHub Releases](https://github.com/davidthegnomad/gnomad-desktop-assistant/releases).

Optional **Flatpak** / **Snap** builds: `npm run pack:flatpak`, `npm run pack:snap`, or the **Packaging** GitHub workflow. See [`docs/FLATPAK.md`](docs/FLATPAK.md) and [`docs/SNAP.md`](docs/SNAP.md).

Updater signing: `npm run setup:updater-keys` · verify with `npm run verify:updater` — see [`docs/UPDATER.md`](docs/UPDATER.md) and [`docs/RELEASE_RUNBOOK.md`](docs/RELEASE_RUNBOOK.md).

Optional embedded GGUF: `npm run download:gguf` — see [`docs/GGUF_SETUP.md`](docs/GGUF_SETUP.md).

---

## License

Private project — see repository settings. Contact [Gnomad Studio](https://gnomadstudio.org) for licensing inquiries.

---

## Acknowledgments

UI patterns informed by Google Gemini’s 2026 “Neural Expressive” redesign; implementation is original to Gnomad Studio. Built with [Tauri](https://v2.tauri.app/), [React](https://react.dev/), and [Rust](https://www.rust-lang.org/).
