# Contributing to Gnomad Desktop Assistant

Thanks for your interest in Gnomad. This project is maintained by [Gnomad Studio](https://gnomadstudio.org).

---

## Development setup

**Prerequisites:** Node.js LTS, Rust stable, [Tauri v2 platform deps](https://v2.tauri.app/start/prerequisites/)

```bash
git clone https://github.com/davidthegnomad/gnomad-desktop-assistant.git
cd gnomad-desktop-assistant
npm ci
cp .env.example .env   # optional API keys for local dev
npm run tauri dev
```

**Embedded GGUF (optional):**

```bash
npm run tauri:dev:embedded
npm run download:gguf    # optional small model for planner dev
```

---

## Before you open a PR

```bash
npm run test              # Vitest (frontend)
cd src-tauri && cargo test
npm run build
```

Edit docs in `.md` form, then:

```bash
npm run docs:export
```

Cross-platform UI or shell changes: walk through [CROSS_PLATFORM_CHECKLIST.md](docs/CROSS_PLATFORM_CHECKLIST.md).

---

## Project layout

| Path | Purpose |
|------|---------|
| `src/` | React UI, hooks, agent loop |
| `src-tauri/src/` | Rust commands, agent, safety, LLM |
| `docs/` | User and engineering documentation |
| `packaging/` | Flatpak and Snap manifests |
| `.github/workflows/` | CI, release, packaging |

Start from [docs/DOCS_INDEX.md](docs/DOCS_INDEX.md) and [docs/ARCHITECTURE.md](docs/ARCHITECTURE.md).

---

## Code conventions

- **Minimal diffs** — match existing naming and patterns  
- **Safety first** — shell and FS paths must use HITL tokens; no new unsigned bypasses  
- **Structured errors** — use `GnomadError` JSON on Rust invoke errors  
- **No secrets in repo** — keys in keychain or `.env` (gitignored)

---

## Reporting issues

Use [GitHub Issues](https://github.com/davidthegnomad/gnomad-desktop-assistant/issues) with OS version, Gnomad version, and steps to reproduce. Do not paste API keys.

---

## License

Private / portfolio project — see repository settings for licensing inquiries.

---

Built with ❤️ by [Gnomad Studio](https://gnomadstudio.org) 🦙
