# Roadmap: Gnomad Desktop Assistant

**Last Updated:** 2026-06-05

## Linux GTK rewrite (`gnomad-gtk`)

| Capability | Status |
|------------|--------|
| Tray + window modes (Panel / Pop out / Window / Full) | Shipped |
| Cross-DE tray UX + help + tooltips | Shipped |
| `gnomad-gtk --doctor` + `gtk:preflight` | Shipped |
| Chat sessions, Ollama/cloud LLM | Shipped |
| Agent loop (`shell_run`, `fs_*`, gates, audit) | Shipped |
| Settings, knowledge library, agent secrets | Shipped |
| Context pills (KDE/GNOME/COSMIC/Hyprland probes) | Shipped |
| VTE embedded terminal | Shipped (optional `vte` feature) |
| KDE Wayland doctor gate | 12 pass / 1 warn / 0 fail |
| First-run onboarding wizard | Shipped (`ui-prefs.json`) |
| VTE live agent PTY stream | Shipped |
| KDE panel anchor (KWin + wmctrl) | Shipped |
| Theme prefs (`omni_theme` migration) | Shipped |
| GNOME/COSMIC manual QA | Deferred |

## Near Term (0–30 days)

- [x] Phase 1 GTK shell — tray, modes, resize
- [x] Phase 2 chat + agent + gates
- [x] Tray/help UX cross-DE safe
- [x] Desktop doctor + `gtk:preflight`
- [x] Phase 3 GTK — onboarding wizard (2-step cloud/Ollama)
- [ ] KDE interactive smoke (tray, Pop Out, composer) — `npm run gtk:smoke:kde` + manual matrix
- [ ] Tag `v0.2.0-gtk` release after `npm run gtk:release:smoke`
- [x] Phase 3 GTK — composer attachments (`gnomad-core` staging + GTK chip tray)
- [x] Phase 4 (partial) — `--migrate`, `gtk:package:rpm`, KDE smoke script
- [x] Phase 4 — deb/AppImage CI (`gtk-build.yml`), binary renamed to `gnomad`

## Medium Term (30–90 days)

- [ ] GNOME + COSMIC QA matrix
- [ ] Update checker (GTK)
- [x] Rename shipping binary to `gnomad` (from `gnomad-gtk`)

---
*See [STATUS.md](./STATUS.md), [docs/LINUX_NATIVE_REWRITE_PLAN.md](./docs/LINUX_NATIVE_REWRITE_PLAN.md).*
