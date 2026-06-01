# Product Roadmap — Gnomad Desktop Assistant

**Version:** 0.2.0-beta.1  
**Last updated:** June 2026  
**Horizon:** v1.0 GA (indicative)

---

## Vision

Gnomad becomes the **default desktop agent surface**: always available, OS-aware, trustworthy by default, and capable of real work through shell and files—without surrendering control to opaque automation.

---

## Released — v0.1.0-alpha ✓

| Theme | Delivered |
|-------|-----------|
| **Platform** | macOS, Windows, Linux installers; CI matrix |
| **UX** | Gemini-inspired UI, tray, 4 window modes, themes |
| **Intelligence** | DeepSeek cloud + Ollama local |
| **Agent** | Tool loop, PTY shell, FS tools, audit log |
| **Safety** | Sudo Gate, Path Gate, Standard/YOLO trust |
| **Memory** | Chat history, knowledge library |
| **Docs** | README, tech stack, build narrative, QA, user guide |

---

## v0.2 — Beta readiness ✓

**Status:** Shipped on `main` as **0.2.0-beta.1**

| Item | Priority | Notes |
|------|----------|-------|
| Server-side safety on all shell paths | P0 | ✓ Legacy `execute_shell_command` removed |
| Cryptographic HITL tokens (Wave B1) | P0 | ✓ [hitl_token.rs](../src-tauri/src/hitl_token.rs) |
| Wave B error migration | P1 | ✓ JSON `GnomadError` on LLM + chat paths |
| Rust unit tests | P0 | ✓ CI gate (`cargo test`) |
| CODE_REVIEW / TEST_NOTES refresh | P1 | ✓ |
| Auto-update channel | P1 | ✓ Settings → Updates; [UPDATER.md](UPDATER.md) |
| Linux Wayland tray | P1 | ✓ |
| Error telemetry (opt-in) | P2 | ✓ Local `error-log.jsonl` |

---

## Wave B — State-of-the-art ✓

→ **[WAVE_B_ROADMAP.md](WAVE_B_ROADMAP.md)** — HITL tokens, GGUF, xterm PTY, YOLO sandbox (shipped)

---

## v0.3 — Broader reach ✓

**Status:** Shipped on `main` as **0.2.0-beta.1**

| Item | Priority | Notes |
|------|----------|-------|
| Linux ARM64 builds | P1 | ✓ CI `ubuntu-24.04-arm` |
| Snap / Flatpak | P2 | ✓ [FLATPAK.md](FLATPAK.md), [SNAP.md](SNAP.md) |
| Additional cloud providers | P2 | ✓ OpenAI-compatible endpoint |
| Plugin/skills marketplace (local) | P2 | ✓ Starter skill packs |
| Voice input (push-to-talk) | P3 | ✓ Beta — Web Speech API; Settings → Privacy |

---

## v1.0 — General availability (in progress)

| Item | Priority | Notes |
|------|----------|-------|
| Signed + notarized macOS | P0 | Script + optional CI — [MACOS_NOTARIZATION.md](MACOS_NOTARIZATION.md); needs Apple secrets |
| Security review / pen test | P0 | ✓ Internal checklist — [SECURITY_REVIEW.md](SECURITY_REVIEW.md); external pen test before GA |
| WCAG accessibility pass | P1 | ✓ Partial + [ACCESSIBILITY_STATEMENT.md](ACCESSIBILITY_STATEMENT.md); full audit pending |
| Enterprise deployment guide | P2 | ✓ [ENTERPRISE.md](ENTERPRISE.md) |
| Team workspace sync (optional) | P3 | Deferred post–v1.0 |

### Operational before GA tag

1. Replace updater placeholder pubkey — `npm run setup:updater-keys` → `verify:updater`
2. Configure Apple Developer ID + notarization secrets
3. Run [SECURITY_REVIEW.md](SECURITY_REVIEW.md) sign-off
4. Full WCAG contrast + screen reader pass

---

## Non-goals (current)

- Mobile iOS/Android clients  
- Hosted multi-user SaaS backend for chat  
- Autonomous background tasks without user message  
- Cryptocurrency / unrelated integrations  

---

## How priorities are set

1. **Safety and trust** before new capabilities  
2. **Cross-platform parity** before niche features  
3. **Local-first privacy** preserved for default paths  
4. **Portfolio/demo quality** aligned with production bar  

Feedback: [GitHub Issues](https://github.com/davidthegnomad/gnomad-desktop-assistant/issues)

---

Built with ❤️ by [Gnomad Studio](https://gnomadstudio.org) 🦙
