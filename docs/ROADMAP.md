# Product Roadmap — Gnomad Desktop Assistant

**Version:** 0.1.0-alpha  
**Last updated:** June 2026  
**Horizon:** 6–12 months (indicative)

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

## v0.2 — Beta readiness

**Target:** Hardening and supportability

| Item | Priority | Notes |
|------|----------|-------|
| Server-side safety on all shell paths | P0 | Close IPC bypass class |
| **Cryptographic HITL tokens (Wave B1)** | P0 | ✓ Shipped — [hitl_token.rs](../src-tauri/src/hitl_token.rs) |
| Wave B error migration (`llm`, planner, chat history) | P1 | ✓ Shipped — JSON `GnomadError` on LLM + chat store paths |
| Rust unit tests (`privilege`, parsers, `error`) | P0 | CI gate ✓ (`cargo test` in build workflow) |
| Refresh CODE_REVIEW / TEST_NOTES | P1 | Match shipped agent ✓ |
| Auto-update channel (Tauri updater) | P1 | ✓ Shipped — Settings → Updates; see [UPDATER.md](UPDATER.md) |
| Improved Linux Wayland tray | P1 | ✓ Left-click menu on Wayland; session hint in Settings |
| Error telemetry (opt-in, local-first) | P2 | Crash logs only |

---

## Wave B — State-of-the-art (v0.3+)

Detailed design for four evaluation follow-ups: **HITL tokens**, **in-process LLM**, **Xterm.js PTY**, **YOLO micro-sandboxing**.

→ **[WAVE_B_ROADMAP.md](WAVE_B_ROADMAP.md)** (phasing, acceptance criteria, risks)

---

## v0.3 — Broader reach

| Item | Priority | Notes |
|------|----------|-------|
| Linux ARM64 builds | P1 | Apple Silicon Linux devices |
| Snap / Flatpak | P2 | Community demand |
| Additional cloud providers | P2 | OpenAI-compatible router |
| Plugin/skills marketplace (local) | P2 | Curated skill packs |
| Voice input (push-to-talk) | P3 | Platform STT APIs |

---

## v1.0 — General availability

| Item | Priority | Notes |
|------|----------|-------|
| Signed + notarized macOS | P0 | Enterprise trust |
| Security review / pen test | P0 | External or structured internal |
| WCAG accessibility pass | P1 | Keyboard nav, contrast |
| Enterprise deployment guide | P2 | MDM, proxy, key escrow |
| Team workspace sync (optional) | P3 | E2E encrypted |

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
