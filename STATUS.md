# Status: Gnomad Desktop Assistant

**Last Updated:** 2026-06-05
**Overall:** 🟢 Active — Linux GTK rewrite in progress

## Current State

- **Tauri/React:** v0.2.0-beta.1 — macOS + legacy Linux packaging
- **gnomad-gtk:** Phase 2 — tray UX, help, doctor, agent/chat/knowledge/settings shipped on KDE Wayland

## Recent Accomplishments (GTK)

- Cross-DE tray menu (Windowed left-click, flavor labels, Settings, Studio link)
- Help `?` dialog + `GNOMAD_HELP.md` injected into agent context
- `gnomad-gtk --doctor` desktop diagnostics (session, SNI, tools, live context)
- `npm run gtk:preflight` — test + build + doctor gate
- First-run onboarding wizard (cloud API key or Ollama URL)
- GNOME/COSMIC context probes in `linux_context.rs`

## KDE verification

`gnomad-gtk --doctor`: **12 passed, 1 warning, 0 failed** (Nobara 43, KDE Wayland, NVIDIA).

GNOME/COSMIC manual QA deferred — doctor matrix documented in `crates/gnomad-gtk/README.md`.

## Next Actions

- [ ] KDE interactive smoke — run `npm run gtk:smoke:kde` then manual checklist in README
- [ ] GNOME/COSMIC doctor + manual matrix when machines available
- [x] Phase 3: composer attachments (GTK — file picker, chips, prompt inlining)
- [x] Phase 4 (partial): `gnomad --migrate`, `gtk:package:rpm`, KDE smoke script
- [x] Phase 4: deb/AppImage CI (`gtk-build.yml`), shipping binary `gnomad`
- [x] VTE live PTY stream, KDE panel anchor, theme migration, `gtk:release:smoke`
- [ ] Tag `v0.2.0-gtk` and verify install on clean machine

---
*See [ROADMAP.md](./ROADMAP.md), [crates/gnomad-gtk/README.md](./crates/gnomad-gtk/README.md).*
