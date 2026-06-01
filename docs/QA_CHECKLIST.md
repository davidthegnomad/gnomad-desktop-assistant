# QA Checklist — Gnomad Desktop Assistant

**Version under test:** _______________  
**Build / artifact:** _______________  
**Tester:** _______________  
**Date:** _______________  
**Environment:** Production-like (release build preferred; note if `tauri dev`)

Use one checklist section per OS. Mark **PASS**, **FAIL**, or **N/A** and add notes for failures. Block release on any **P0** failure.

**During development** (before a formal QA pass), use [`CROSS_PLATFORM_CHECKLIST.md`](CROSS_PLATFORM_CHECKLIST.md) when adding features or UI so macOS work is verified—or explicitly scoped—on Windows and Linux.

**Severity key**

| Level | Definition |
|-------|------------|
| **P0** | Crash, data loss, security bypass, or core feature unusable |
| **P1** | Major feature broken; workaround difficult |
| **P2** | Cosmetic or edge case; workaround exists |

---

## Pre-flight (all platforms)

| ID | Test | P | Result | Notes |
|----|------|---|--------|-------|
| PF-01 | Install from release artifact (not dev folder) | P0 | | |
| PF-02 | App launches without panic or blank window | P0 | | |
| PF-03 | Version in About matches expected tag | P1 | | |
| PF-04 | No `.env` required for basic UI (keys may be empty) | P1 | | |
| PF-05 | Tray/status icon visible after launch | P0 | | |
| PF-06 | Window hidden on first launch until shortcut/tray open | P1 | | |

---

## macOS QA

**Target OS versions (test matrix):**

| macOS | Arch | Build | Tester | Date |
|-------|------|-------|--------|------|
| 14 Sonoma | arm64 | | | |
| 15 Sequoia | arm64 | | | |
| 13 Ventura | x86_64 or arm64 | | | |

**Install artifact:** `.dmg` (Universal or arch-specific) → `Gnomad.app`

### Installation & lifecycle

| ID | Test | P | Result | Notes |
|----|------|---|--------|-------|
| MAC-01 | Drag to Applications; first open (Gatekeeper) | P0 | | |
| MAC-02 | Quit from tray → process exits | P0 | | |
| MAC-03 | Close window (red button) → hides to menu bar, not quit | P0 | | |
| MAC-04 | No unwanted Dock icon in panel/accessory mode | P2 | | |
| MAC-05 | Reopen after reboot (tray still works) | P1 | | |

### Window modes & chrome

| ID | Test | P | Result | Notes |
|----|------|---|--------|-------|
| MAC-10 | **⌘⇧Space** toggles show/hide | P0 | | |
| MAC-11 | Tray click opens panel anchored near menu bar | P0 | | |
| MAC-12 | Panel mode: compact UI, tray positioning | P1 | | |
| MAC-13 | Pop out: movable floating window | P1 | | |
| MAC-14 | Windowed: native menus (File/Edit/View/Window/Help) | P1 | | |
| MAC-15 | Fullscreen enters/exits cleanly | P2 | | |
| MAC-16 | Title bar overlay; traffic lights usable | P2 | | |
| MAC-17 | Resize min/max within configured bounds | P2 | | |

### Tray menu

| ID | Test | P | Result | Notes |
|----|------|---|--------|-------|
| MAC-20 | Show Gnomad / menu bar item | P0 | | |
| MAC-21 | Pop Out Window | P1 | | |
| MAC-22 | Settings opens app + settings pane | P1 | | |
| MAC-23 | Visit gnomadstudio.org opens browser | P2 | | |
| MAC-24 | About modal | P2 | | |
| MAC-25 | Quit Gnomad terminates app | P0 | | |

### Onboarding & settings

| ID | Test | P | Result | Notes |
|----|------|---|--------|-------|
| MAC-30 | First launch onboarding (no key, no Ollama) | P1 | | |
| MAC-31 | Cloud path: save DeepSeek key → chat enabled | P0 | | |
| MAC-32 | Local path: Ollama URL + model → chat enabled | P0 | | |
| MAC-33 | Key persists after app restart (keychain) | P0 | | |
| MAC-34 | Re-run setup wizard from settings | P2 | | |
| MAC-35 | Theme cycle: light → dark → system | P2 | | |
| MAC-36 | Agent access: change workspace folder | P1 | | |
| MAC-37 | Trust mode Standard vs YOLO | P1 | | |
| MAC-38 | Reset shell session | P2 | | |

### Chat & LLM

| ID | Test | P | Result | Notes |
|----|------|---|--------|-------|
| MAC-40 | Cloud: send message → real model response | P0 | | |
| MAC-41 | Local: send message → Ollama response | P0 | | |
| MAC-42 | Provider/model dropdowns match configuration | P1 | | |
| MAC-43 | Suggestion chips populate composer | P2 | | |
| MAC-44 | New chat (sidebar + menu **⌘N**) | P1 | | |
| MAC-45 | Select/delete prior session | P1 | | |
| MAC-46 | Attach file → appears in message | P1 | | |
| MAC-47 | Stop button interrupts thinking/shell | P1 | | |

### Agent & shell

| ID | Test | P | Result | Notes |
|----|------|---|--------|-------|
| MAC-50 | Safe command (`echo hello`) → stdout in thread | P0 | | |
| MAC-51 | `pwd` / `cd` → cwd updates in session | P1 | | |
| MAC-52 | Risky command (`sudo ls`) → Sudo Gate modal | P0 | | |
| MAC-53 | Deny Sudo Gate → no execution, clear error | P0 | | |
| MAC-54 | Approve Sudo Gate → command runs | P0 | | |
| MAC-55 | Path outside workspace → Path Gate (Standard mode) | P1 | | |
| MAC-56 | Local: `$ command` or terminal button runs shell | P1 | | |
| MAC-57 | Audit log appended (`agent-audit.jsonl`) | P2 | | |

### OS context & permissions

| ID | Test | P | Result | Notes |
|----|------|---|--------|-------|
| MAC-60 | Active app name updates when switching apps | P1 | | |
| MAC-61 | Window title updates | P1 | | |
| MAC-62 | Clipboard snippet updates after copy | P1 | | |
| MAC-63 | Accessibility banner when denied | P1 | | |
| MAC-64 | Open System Settings from banner/settings | P2 | | |
| MAC-65 | Test elevation triggers admin prompt | P2 | | |

### Knowledge base

| ID | Test | P | Result | Notes |
|----|------|---|--------|-------|
| MAC-70 | Open Knowledge panel (book icon / sidebar) | P1 | | |
| MAC-71 | Add files → listed in library | P1 | | |
| MAC-72 | INDEX.md updates | P2 | | |
| MAC-73 | Skill referenced in chat context | P2 | | |

---

## Windows QA

**Target versions:**

| Windows | Edition | Arch | Build | Tester | Date |
|---------|---------|------|-------|--------|------|
| 11 23H2+ | Pro/Home | x64 | | | |
| 10 22H2 | Pro | x64 | | | |

**Install artifact:** `.msi` or NSIS `setup.exe`

### Installation & lifecycle

| ID | Test | P | Result | Notes |
|----|------|---|--------|-------|
| WIN-01 | Installer completes; Start Menu entry | P0 | | |
| WIN-02 | Uninstall removes app cleanly | P1 | | |
| WIN-03 | Close window → hides to tray | P0 | | |
| WIN-04 | Quit from tray exits process | P0 | | |

### Window modes & shortcuts

| ID | Test | P | Result | Notes |
|----|------|---|--------|-------|
| WIN-10 | **Ctrl+Shift+Space** toggles visibility | P0 | | |
| WIN-11 | Tray click opens panel near system tray | P0 | | |
| WIN-12 | Pop out / Windowed / Fullscreen modes | P1 | | |
| WIN-13 | Windowed: in-app toolbar hidden; OS menu bar used | P1 | | |
| WIN-14 | Native caption buttons work | P1 | | |

### Tray & menus

| ID | Test | P | Result | Notes |
|----|------|---|--------|-------|
| WIN-20 | Tray icon visible | P0 | | |
| WIN-21 | Show / Settings / Quit items | P0 | | |
| WIN-22 | Application menus in windowed mode | P1 | | |

### Credentials & chat

| ID | Test | P | Result | Notes |
|----|------|---|--------|-------|
| WIN-30 | API key → Credential Manager persistence | P0 | | |
| WIN-31 | Cloud + local chat functional | P0 | | |
| WIN-32 | Chat history survives restart | P1 | | |

### Agent & context

| ID | Test | P | Result | Notes |
|----|------|---|--------|-------|
| WIN-40 | `echo test` shell output in chat | P0 | | |
| WIN-41 | Sudo Gate on risky commands | P0 | | |
| WIN-42 | Active window via PowerShell | P1 | | |
| WIN-43 | Clipboard context updates | P1 | | |
| WIN-44 | Elevated command messaging (admin terminal) | P2 | | |

### Knowledge & UI

| ID | Test | P | Result | Notes |
|----|------|---|--------|-------|
| WIN-50 | Knowledge panel add/browse files | P1 | | |
| WIN-51 | Theme toggle | P2 | | |
| WIN-52 | Sidebar collapse/resize | P2 | | |

---

## Linux QA

**Target environments (run at least one .deb, one .rpm, one AppImage):**

| Distro | Version | Format | DE (GNOME/KDE) | Tester | Date |
|--------|---------|--------|----------------|--------|------|
| Ubuntu | 22.04/24.04 | .deb | | | |
| Fedora | 39+ | .rpm | | | |
| Arch / other | rolling | AppImage | | | |

**Optional packages for full context:** `xdotool` (X11), `wl-paste` (Wayland) or `xclip`

### Installation

| ID | Test | P | Result | Notes |
|----|------|---|--------|-------|
| LNX-01 | `.deb`: `apt install` + dependencies resolve | P0 | | |
| LNX-02 | `.rpm`: `dnf install` succeeds | P0 | | |
| LNX-03 | AppImage executes (`chmod +x`) | P0 | | |
| LNX-04 | Desktop entry / launcher appears | P1 | | |
| LNX-05 | Uninstall/remove clean | P1 | | |

### Tray & windowing (Wayland + X11 if possible)

| ID | Test | P | Result | Notes |
|----|------|---|--------|-------|
| LNX-10 | System tray icon visible (AppIndicator) | P0 | | |
| LNX-11 | **Ctrl+Shift+Space** toggle | P0 | | |
| LNX-12 | Panel / Pop out / Windowed / Fullscreen | P1 | | |
| LNX-13 | Close → hide to tray | P0 | | |

### Secrets & chat

| ID | Test | P | Result | Notes |
|----|------|---|--------|-------|
| LNX-20 | Secret Service stores API key | P0 | | |
| LNX-21 | Cloud + Ollama chat | P0 | | |
| LNX-22 | Chat sessions persist | P1 | | |

### Agent & context

| ID | Test | P | Result | Notes |
|----|------|---|--------|-------|
| LNX-30 | Shell command output in chat | P0 | | |
| LNX-31 | Sudo Gate / `pkexec` elevation path | P1 | | |
| LNX-32 | Active window (with `xdotool` if X11) | P2 | | |
| LNX-33 | Clipboard (`wl-paste` or `xclip`) | P2 | | |
| LNX-34 | Without optional tools: app still usable, context N/A | P2 | | |

### Packaging-specific

| ID | Test | P | Result | Notes |
|----|------|---|--------|-------|
| LNX-40 | WebKitGTK renders UI correctly | P0 | | |
| LNX-41 | README in `/usr/share/doc/gnomad/` (.deb) | P2 | | |
| LNX-42 | HiDPI / fractional scaling | P2 | | |

---

## Cross-platform regression (run on each OS)

| ID | Test | P | Result | Notes |
|----|------|---|--------|-------|
| XP-01 | No secrets in chat logs or screenshots | P0 | | |
| XP-02 | Large attachment does not crash webview | P1 | | |
| XP-03 | Long chat thread scroll performance | P2 | | |
| XP-04 | Offline: graceful error when cloud unreachable | P1 | | |
| XP-05 | Ollama stopped: clear local error message | P1 | | |
| XP-06 | Invalid API key: user-visible error | P1 | | |
| XP-07 | Studio link opens https://gnomadstudio.org | P2 | | |

---

## Security spot checks (all platforms)

| ID | Test | P | Result | Notes |
|----|------|---|--------|-------|
| SEC-01 | Denied Sudo Gate does not execute command | P0 | | |
| SEC-02 | Standard mode blocks FS outside workspace without approval | P0 | | |
| SEC-03 | API key not visible after save | P0 | | |
| SEC-04 | `.env` keys show “locked by env” in UI | P2 | | |

---

## Sign-off

| Role | Name | Date | Approved |
|------|------|------|----------|
| QA | | | ☐ |
| Engineering | | | ☐ |
| Product | | | ☐ |

**Open defects blocking release:**

1. _______________________________________________
2. _______________________________________________

---

Built with ❤️ by Gnomad Studio 🦙
