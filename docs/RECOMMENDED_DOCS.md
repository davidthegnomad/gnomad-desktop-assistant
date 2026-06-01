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
| [RELEASE_RUNBOOK.md](RELEASE_RUNBOOK.md) | Tag → CI → release checklist |
| [TROUBLESHOOTING.md](TROUBLESHOOTING.md) | Support guide |
| [FLATPAK.md](FLATPAK.md) | Optional Flatpak packaging |
| [SNAP.md](SNAP.md) | Optional Snap packaging |
| [ACCESSIBILITY.md](ACCESSIBILITY.md) | Keyboard shortcuts and a11y status |
| [ENTERPRISE.md](ENTERPRISE.md) | MDM, proxy, enterprise deploy |
| [GGUF_SETUP.md](GGUF_SETUP.md) | Download and configure embedded GGUF |
| [MACOS_NOTARIZATION.md](MACOS_NOTARIZATION.md) | Apple notarization for enterprise macOS |
| [TEST_STRATEGY.md](TEST_STRATEGY.md) | Unit, integration, and QA layers |
| [SECURITY_REVIEW.md](SECURITY_REVIEW.md) | Pre-release security checklist |
| [ACCESSIBILITY_STATEMENT.md](ACCESSIBILITY_STATEMENT.md) | Formal WCAG statement (beta) |
| [CROSS_PLATFORM_CHECKLIST.md](CROSS_PLATFORM_CHECKLIST.md) | Per-OS dev verification |
| [MACOS_PERMISSIONS.md](MACOS_PERMISSIONS.md) | macOS privacy matrix |
| [KNOWLEDGE.md](KNOWLEDGE.md) | Knowledge base layout |

---

## Still recommended (not yet written)

| Document | Priority | Notes |
|----------|----------|-------|
| **CONTRIBUTING.md** | Open source | ✓ Root [CONTRIBUTING.md](../CONTRIBUTING.md) |
| **TEST_STRATEGY.md** | Engineering | ✓ [TEST_STRATEGY.md](TEST_STRATEGY.md) |
| **ADR folder** (`docs/adr/`) | Engineering | Decision records |
| **ACCESSIBILITY_STATEMENT.md** | GA | ✓ [ACCESSIBILITY_STATEMENT.md](ACCESSIBILITY_STATEMENT.md) |

### Recently added

| Document | Purpose |
|----------|---------|
| [RELEASE_RUNBOOK.md](RELEASE_RUNBOOK.md) | Tag → CI → GitHub Release checklist |
| [TROUBLESHOOTING.md](TROUBLESHOOTING.md) | Consolidated support guide |

---

## Regenerating HTML & TXT

Edit the `.md` source, then:

```bash
npm run docs:export
```

Script: `scripts/export-docs.mjs` · Theme: `docs/_assets/doc-theme.css`

---

Built with ❤️ by [Gnomad Studio](https://gnomadstudio.org) 🦙
