# Building Gnomad for macOS, Linux, and Windows

## Prerequisites (all platforms)

- Node.js LTS
- Rust stable (`rustup`)
- Platform SDKs for the OS you are building **on** (cross-compiling is possible but not covered here)

```bash
npm install
npm run build
```

## macOS

```bash
npm run tauri:build:mac
```

Output: `src-tauri/target/release/bundle/macos/Gnomad.app`

- **Panel / Pop out:** menu-bar accessory, in-app toolbar
- **Windowed:** full menu bar (Gnomad, File, Edit, View, Window, Help), Gemini-style body without duplicate toolbar
- Close button hides to menu bar; **Quit** from tray menu

## Cross-platform parity (macOS, Windows, Linux)

The **same React UI** ships in every build: chat, attachments, API keys (keychain / Credential Manager / Secret Service), cursor bloom, knowledge base, settings, and window modes (tray panel, pop-out, windowed, fullscreen).

| Area | macOS | Windows | Linux |
|------|-------|---------|-------|
| Tray / status area | Menu bar icon | System tray | System tray (AppIndicator) |
| Global shortcut | ⌘⇧Space | Ctrl+Shift+Space | Ctrl+Shift+Space |
| Panel placement | Under menu bar / tray click | Top-right near tray (or under tray click) | Same as Windows |
| Title bar | Overlay + in-app toolbar | Native caption; toolbar hidden in **Windowed** (menus in OS bar) | Same as Windows |
| Active window context | AppleScript | PowerShell + Win32 | `xdotool` (optional) |
| Clipboard context | `pbpaste` | PowerShell `Get-Clipboard` | `wl-paste` / `xclip` |
| API keys | Keychain | Credential Manager | Secret Service |
| Admin commands | `osascript` sudo | User runs elevated terminal | `pkexec` |

**Optional Linux packages** for full context pills: `xdotool` (X11), `wl-paste` (Wayland) or `xclip`.

CI (`.github/workflows/build.yml`) builds all three platforms on every push to `main` / `master`.

### Cross-platform verification (features & UI tweaks)

Whenever you add or change UI, settings, or OS-facing behavior, use **[`CROSS_PLATFORM_CHECKLIST.md`](CROSS_PLATFORM_CHECKLIST.md)** so work on macOS translates—or is explicitly handled—for Windows and Linux.

Quick rules:

1. **Shared React** (`src/`) — same bundle everywhere; smoke **panel** and **windowed** on Win/Linux (toolbar may be hidden in windowed).
2. **Platform shell** (title bar, tray, shortcuts) — update [`platform.rs`](../src-tauri/src/platform.rs), [`window_manager.rs`](../src-tauri/src/window_manager.rs), and `.platform-*` CSS in [`index.css`](../src/index.css).
3. **Before merge** — `npm run build`, `cargo test`, CI green on all three OS jobs.

## Linux

Build on a Linux host (Ubuntu/Debian recommended):

```bash
npm run tauri:build:linux
```

Install [Tauri Linux dependencies](https://v2.tauri.app/start/prerequisites/#linux) first (webkit2gtk, etc.).

Output under `src-tauri/target/release/bundle/`:

- **`.deb`** — Debian, Ubuntu, Mint, Pop!\_OS
- **`.rpm`** — Fedora, RHEL, Rocky, openSUSE
- **AppImage** — distro-agnostic (Arch, NixOS, etc.)

See [`LINUX_PACKAGES.md`](LINUX_PACKAGES.md) for install commands per distribution.

```bash
npm run tauri:build:linux          # all three
npm run tauri:build:linux:deb      # deb only
npm run tauri:build:linux:rpm      # rpm only
npm run tauri:build:linux:appimage # AppImage only
```

- Same UI and features as macOS (Gemini layout, sidebar, composer, attachments, API keys)
- Full **File / Edit / View / Window / Help** menus in the window menu bar
- **View → System Tray Panel** / **Windowed** — labels match Linux tray UX; windowed mode uses the OS menu bar (in-app toolbar hidden)
- System tray supported; close hides to tray

## Windows

Build on a Windows PC (recommended):

```bash
npm run tauri:build:win
```

**CI:** Pushes to `main`/`master` run [`.github/workflows/build.yml`](../.github/workflows/build.yml) on `windows-latest`, `ubuntu-22.04`, and `macos-latest` and upload installers as artifacts.

**Cross-compile from macOS** (optional): install MinGW (`brew install mingw-w64`), add `rustup target add x86_64-pc-windows-gnu`, set `CARGO_TARGET_X86_64_PC_WINDOWS_GNU_LINKER=x86_64-w64-mingw32-gcc`, then `npm run tauri:build:win`.

Output: `.msi` / `.exe` under `src-tauri/target/release/bundle/`

- Same UI and features as macOS; menus in the window menu bar
- **Ctrl+Shift+Space** toggles visibility
- Tray panel opens near the **system tray**; **Windowed** hides the duplicate in-app toolbar
- Active window + clipboard context via PowerShell
- Close hides to system tray; exit via tray **Quit Gnomad**

## Development

```bash
npm run tauri dev
```
