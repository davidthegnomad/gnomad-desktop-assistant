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

## Linux

Build on a Linux host (Ubuntu/Debian recommended):

```bash
npm run tauri:build:linux
```

Install [Tauri Linux dependencies](https://v2.tauri.app/start/prerequisites/#linux) first (webkit2gtk, etc.).

Output: `.deb` and/or `.AppImage` under `src-tauri/target/release/bundle/`

- Same UI as macOS (Gemini layout, sidebar, composer)
- Full **File / Edit / View / Window / Help** menus in the window menu bar
- **View → Windowed** hides the in-app toolbar (controls live in menus)
- System tray supported; close hides to tray

## Windows

Build on a Windows PC (recommended):

```bash
npm run tauri:build:win
```

**CI:** Pushes to `main`/`master` run [`.github/workflows/build.yml`](../.github/workflows/build.yml) on `windows-latest`, `ubuntu-22.04`, and `macos-latest` and upload installers as artifacts.

**Cross-compile from macOS** (optional): install MinGW (`brew install mingw-w64`), add `rustup target add x86_64-pc-windows-gnu`, set `CARGO_TARGET_X86_64_PC_WINDOWS_GNU_LINKER=x86_64-w64-mingw32-gcc`, then `npm run tauri:build:win`.

Output: `.msi` / `.exe` under `src-tauri/target/release/bundle/`

- Same menus in the window title area
- **Ctrl+Shift+Space** toggles visibility
- Close hides to system tray; exit via tray **Quit Gnomad**

## Development

```bash
npm run tauri dev
```
