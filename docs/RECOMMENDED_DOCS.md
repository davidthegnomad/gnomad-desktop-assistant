# Recommended Documentation — Portfolio & Production Readiness

**Status:** Core portfolio set **complete** (see [DOCS_INDEX.md](DOCS_INDEX.md)). All docs available as `.md`, `.html`, and `.txt` via `npm run docs:export`.

---

## Shipped in this repository

| Document | Purpose |
|----------|---------|
| [DOCS_INDEX.md](DOCS_INDEX.md) | Master index — all formats |
| [USER_GUIDE.md](USER_GUIDE.md) | End-user manual |
| [TECH_STACK.md](TECH_STACK.md) | Stack rationale |
| [BUILD.md](BUILD.md) | Phased delivery narrative |
| [ARCHITECTURE.md](ARCHITECTURE.md) | System diagram + IPC |
| [SECURITY_MODEL.md](SECURITY_MODEL.md) | Trust, gates, threats |
| [PRIVACY.md](PRIVACY.md) | Data handling policy (alpha) |
| [ROADMAP.md](ROADMAP.md) | v0.2 → v1.0 plan |
| [DEMO_SCRIPT.md](DEMO_SCRIPT.md) | 5–7 min live demo |
| [QA_CHECKLIST.md](QA_CHECKLIST.md) | Per-OS release QA |
| [BUILD_PLATFORMS.md](BUILD_PLATFORMS.md) | Build commands |
| [WAVE_B_ROADMAP.md](WAVE_B_ROADMAP.md) | HITL, GGUF, xterm, sandbox (shipped on main) |
| [UPDATER.md](UPDATER.md) | Auto-update keys and channels |
| [CROSS_PLATFORM_CHECKLIST.md](CROSS_PLATFORM_CHECKLIST.md) | Per-OS dev verification |
| [MACOS_PERMISSIONS.md](MACOS_PERMISSIONS.md) | macOS privacy matrix |
| [KNOWLEDGE.md](KNOWLEDGE.md) | Knowledge base layout |

---

## Still recommended (not yet written)

| Document | Priority | Notes |
|----------|----------|-------|
| **RELEASE_RUNBOOK.md** | Production | Tag → CI → GitHub Release checklist |
| **TROUBLESHOOTING.md** | Support | Top issues consolidated from USER_GUIDE |
| **CONTRIBUTING.md** | Open source | If repo goes public |
| **TEST_STRATEGY.md** | Engineering | Unit/E2E plan |
| **ADR folder** (`docs/adr/`) | Engineering | Decision records |
| **ACCESSIBILITY_STATEMENT.md** | GA | WCAG goals |

---

## Regenerating HTML & TXT

Edit the `.md` source, then:

```bash
npm run docs:export
```

Script: `scripts/export-docs.mjs` · Theme: `docs/_assets/doc-theme.css`

---

Built with ❤️ by [Gnomad Studio](https://gnomadstudio.org) 🦙
