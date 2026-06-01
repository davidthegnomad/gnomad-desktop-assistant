# Gnomad Desktop Assistant 🦙🍄

Cross-platform desktop assistant built with **Tauri v2**, **React 19**, and **TypeScript**. Gemini-inspired UI, system tray, chat history, knowledge base, and DeepSeek/Ollama chat.

Built with ❤️ by [Gnomad Studio](https://gnomadstudio.org)

## Platforms

| Platform | Status | Notes |
|----------|--------|--------|
| **macOS** | Primary | Menu-bar accessory; full menu bar in Windowed mode |
| **Linux** | Supported | `.deb` / AppImage; in-window menus, system tray |
| **Windows** | Supported | `.msi` installer; system tray, `Ctrl+Shift+Space` |

See [`docs/BUILD_PLATFORMS.md`](docs/BUILD_PLATFORMS.md) for per-OS build steps.

## Features

- Gemini-style welcome, suggestion chips, and composer
- Collapsible chat sidebar + knowledge/skills library
- Cloud (DeepSeek) and local (Ollama) chat
- OS context (active app, window title, clipboard)
- Safe shell execution with Sudo Gate for risky commands
- Chat history persisted locally
- Close window → hide to tray; quit from tray menu

## Prerequisites

- [Node.js](https://nodejs.org/) (LTS)
- [Rust](https://www.rust-lang.org/tools/install)
- Platform prerequisites: [Tauri v2 docs](https://v2.tauri.app/start/prerequisites/)

## API keys (testing)

```bash
cp .env.example .env
# DeepSeek_API_KEY=your-key-here
```

Loaded at startup from project root `.env`. **Never commit `.env`.**

## Development

```bash
npm install
npm run tauri dev
```

## Build

```bash
npm run tauri:build:mac    # macOS
npm run tauri:build:linux  # Linux (on Linux host)
npm run tauri:build:win    # Windows (on Windows host)
```

## Documentation

| Doc | Contents |
|-----|----------|
| [`docs/BUILD_PLATFORMS.md`](docs/BUILD_PLATFORMS.md) | macOS / Linux / Windows builds |
| [`docs/CODE_REVIEW.md`](docs/CODE_REVIEW.md) | Architecture and findings |
| [`docs/KNOWLEDGE.md`](docs/KNOWLEDGE.md) | Knowledge base and skills |
| [`docs/MACOS_PERMISSIONS.md`](docs/MACOS_PERMISSIONS.md) | macOS privacy and elevation |

## License

Private project — see repository settings.
