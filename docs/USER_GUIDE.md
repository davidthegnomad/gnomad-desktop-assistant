# User Guide — Gnomad Desktop Assistant

**Version:** 0.1.0-alpha

Project site: [davidthegnomad.github.io/gnomad-desktop-assistant](https://davidthegnomad.github.io/gnomad-desktop-assistant/)  
Studio: [gnomadstudio.org](https://gnomadstudio.org)

---

## Introduction

Gnomad is a desktop AI assistant that lives in your menu bar (macOS) or system tray (Windows/Linux). It can:

- Chat using cloud (DeepSeek) or local (Ollama) models
- See your active app, window title, and clipboard (footer context strip)
- Run real shell commands and file operations in a controlled workspace
- Store skills and reference documents in a knowledge library

Closing the window hides Gnomad to the tray—it does not quit the app unless you choose **Quit** from the tray menu.

---

## Installation

### macOS

- Download the `.dmg` from GitHub Releases or the project site
- Drag Gnomad to Applications
- First open: right-click → Open if Gatekeeper blocks unsigned alpha builds

### Windows

- Run the `.msi` or `setup.exe` installer
- Launch from Start Menu; allow tray icon if prompted

### Linux

- Debian/Ubuntu: `sudo apt install ./gnomad_*_amd64.deb`
- Fedora/RHEL: `sudo dnf install ./gnomad-*-1.x86_64.rpm`
- Other: `chmod +x gnomad_*.AppImage` && run it

See [LINUX_PACKAGES.md](LINUX_PACKAGES.md) for distro-specific details.

---

## First launch & onboarding

On first launch (when no API key and no Ollama URL are configured), a welcome wizard appears:

1. **Choose provider** — Cloud API (DeepSeek) or Local model (Ollama)
2. **Configure** — API key + model, or Ollama URL + model tag

Keys are saved to your system keychain. Change setup anytime in **Settings → Re-run setup wizard**.

Developers may use `.env` with `DeepSeek_API_KEY`; the UI shows **Set from .env** and locks that key slot.

---

## Opening and closing Gnomad

| Action | macOS | Windows / Linux |
|--------|-------|-----------------|
| Show / hide | Menu bar icon or **⌘⇧Space** | Tray icon or **Ctrl+Shift+Space** |
| Hide window | Close button or shortcut again | Same |
| Quit | Tray → Quit Gnomad | Same |

**Tray menu:** Show Gnomad, Pop Out Window, Settings, Visit gnomadstudio.org, About, Quit.

---

## Window modes

Use the four chips in the title bar (or **View** menu in windowed mode):

| Mode | Description |
|------|-------------|
| **Panel** | Drops from menu bar / near tray; compact layout |
| **Pop out** | Floating, movable window |
| **Window** | Standard window with native menus |
| **Full** | Fullscreen; click again to return to pop out |

---

## Main interface overview

**Title bar:** Window modes, theme toggle, Knowledge (book), Settings (gear).

**Sidebar:** New chat, history, knowledge shortcut; resize, collapse, flip side.

**Center:** Welcome, suggestion chips, messages, shell output blocks.

**Composer:** Input, attachments, terminal (local), provider/model, send.

**Context footer:** Active app, window title, clipboard, permission banner (macOS).

---

## Chat & composer

1. Type or click a suggestion chip
2. Optionally attach files
3. Choose Cloud/Local and model
4. Press **Send** or Enter

**While thinking:** Click **Stop** to interrupt shell activity.

**Direct shell (local):** Prefix with `$` or use the terminal icon.

---

## Cloud vs local AI

| | Cloud (DeepSeek) | Local (Ollama) |
|---|------------------|----------------|
| Requires | API key | Ollama URL + running server |
| Models | Chat, Reasoner | llama3.2, mistral, custom tags |
| Agent | Tool calling | `<gnomad-run>` tags + optional planner |
| Network | Internet | Usually localhost |

Switch via composer or **Settings → Model & API**.

---

## Changing models and API keys

**Composer:** Provider and model dropdowns.

**Settings → Model & API:** Cloud model or Ollama URL + model name.

**Settings → API keys:** Add/change/remove DeepSeek key (keychain). `.env` keys are locked in UI.

---

## Agent: shell & filesystem

- **Cloud:** Tools for shell, files, workspace (up to 10 steps per message)
- **Local:** `gnomad-run` tags, `$` commands, optional command planner
- **Shell session:** Persistent cwd; reset in Settings → Agent shell
- **Workspace:** Settings → Agent access → Change folder
- **Trust:** Standard (workspace + gates) or YOLO (broader FS)
- **Audit:** `{app_data}/gnomad/agent-audit.jsonl`

---

## Safety: Sudo Gate & Path Gate

**Sudo Gate** — Risky shell (sudo, rm -rf, etc.): read reason, Approve or Deny. Approval mints a **signed HITL token** bound to the exact command (single-use, ~2 min TTL). Passing approval without a token is rejected.

**Path Gate** — Standard mode: files outside workspace need approval. **Allow once** mints a **signed path token** bound to the canonical path and scope (read/write).

---

## Embedded GGUF (optional build)

When built with `embedded-llm` (`npm run tauri:dev:embedded`):

1. **Settings → Agent access → GGUF path** — point to a small `.gguf` (e.g. Qwen2.5-Coder 1.5B Q4)
2. **Command planner** uses GGUF in-process (no Ollama required for planner)
3. **Experimental → Use GGUF for local chat** — full local chat without Ollama
4. **Experimental → Sandbox shell in YOLO** — wrap shell in OS sandbox (macOS/Linux)

---

## Terminal view (xterm.js)

While the agent runs commands, live PTY output streams in the thinking panel. On command cards, click **Terminal view** to replay ANSI-colored output in an embedded terminal.

---

## Updates

**Settings → Updates** — choose **Stable** or **Beta** channel, then **Check for updates**. Requires signed release artifacts; see [`UPDATER.md`](UPDATER.md).

---

## Knowledge & skills library

Open via book icon or sidebar. Folders under `gnomad/knowledge/`:

- `skills/` — instructions (`.md`, `.txt`, `.json`)
- `agents/` — personas
- `preferences/` — user preferences
- `uploads/` — reference files

Add files in the panel; `INDEX.md` updates automatically.

---

## Chat history & sidebar

- **New chat:** Sidebar + or **⌘/Ctrl+N**
- **Select / delete** sessions in the list
- **Layout:** Resize, collapse, drag to flip side

Sessions stored locally on disk.

---

## Settings reference

| Section | Purpose |
|---------|---------|
| Model & API | Provider, models, wizard |
| API keys | Keychain secrets |
| Agent access | Workspace, trust, planner, GGUF, experimental flags |
| Updates | Stable/beta channel, check/install |
| Agent shell | CWD, reset session |
| Knowledge | Open library |
| Permissions | Accessibility, elevation test; Wayland tray hint |

**Theme:** Sun/moon/monitor icon cycles Light → Dark → System.

---

## Keyboard shortcuts

| Shortcut | Action |
|----------|--------|
| ⌘⇧Space / Ctrl+Shift+Space | Show / hide |
| ⌘N / Ctrl+N | New chat |
| ⌘, / Ctrl+, | Settings |
| ⌘Q / Ctrl+Q | Quit (windowed) |

---

## Platform-specific notes

**macOS:** Accessibility for automation; admin auth for elevation.

**Windows:** Tray near notification area; elevated cmds in admin terminal.

**Linux:** WebKitGTK + AppIndicator; optional `xdotool`, `wl-paste`, `xclip`. On **Wayland**, left-click the tray icon to open the menu (Settings shows a hint when Wayland is detected).

---

## Troubleshooting

| Issue | Fix |
|-------|-----|
| Cloud disabled | Add DeepSeek API key or `.env` |
| Local disabled | Set Ollama URL; run `ollama serve` |
| Chat errors | Check network or `curl` Ollama URL |
| No tray (Linux) | Install appindicator packages |
| Empty context (Linux) | Install clipboard/window tools |
| macOS permissions | Re-grant Accessibility after updates |

---

## Privacy & data

- Chat and knowledge stay on your device
- Cloud prompts go to DeepSeek per their terms
- Local prompts go to your Ollama server
- API keys in OS keychain
- Clipboard included only when you send a message

See [PRIVACY.md](PRIVACY.md) for the full policy.

---

Built with ❤️ by [Gnomad Studio](https://gnomadstudio.org) 🦙
