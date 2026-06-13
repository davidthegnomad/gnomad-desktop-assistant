# Status: Gnomad Desktop Assistant

**Last Updated:** 2026-06-09
**Overall:** 🟢 Active

> **Canonical docs:** [docs/ROADMAP.md](docs/ROADMAP.md) · [docs/USER_GUIDE.md](docs/USER_GUIDE.md)

## Current State

Cross-platform desktop AI assistant — tray, global shortcut, OS context, shell/filesystem agent with human-in-the-loop gates. **v0.2.0-beta.1** on `main`.

## Recent Accomplishments

- v0.2.0-beta.1 shipped on main
- macOS/Windows/Linux CI matrix + GitHub Pages docs
- DeepSeek cloud + Ollama local; Wave B HITL + agent tooling

## Blockers / Dependencies

- TAURI_SIGNING_* / APPLE_* secrets needed for signed installers

## Next Actions

- [ ] Run `npm run verify:updater`
- [ ] Review `docs/RELEASE_RUNBOOK.md` for GA checklist
- [ ] macOS notarization + signed release tag

---
*See also [PROJECT_SUMMARY.md](./PROJECT_SUMMARY.md) and [ROADMAP.md](./ROADMAP.md).*
