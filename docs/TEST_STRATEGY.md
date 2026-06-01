# Test Strategy — Gnomad Desktop Assistant

**Status:** Alpha — automated unit/integration tests + manual QA matrices  
**Last updated:** June 2026

---

## Goals

1. **Safety regressions never ship silently** — HITL tokens, Path Gate, shell validation
2. **Cross-platform UI stays buildable** — TypeScript + Vite on every push
3. **Release confidence** — manual QA checklist per OS before tags

---

## Test layers

| Layer | Tooling | Scope | CI |
|-------|---------|-------|-----|
| **Rust unit** | `cargo test` | Tokens, privilege heuristics, parsers, env config, sandbox helpers | ✓ `build.yml` |
| **Frontend unit** | Vitest | Error parsing, model normalization, mocked agent loop | ✓ `build.yml` |
| **Build smoke** | `npm run build`, `cargo build` | TS compile + Vite bundle | ✓ |
| **Embedded LLM** | `cargo test --features embedded-llm` | GGUF path (optional) | Manual / local only |
| **E2E GUI** | Manual + [QA_CHECKLIST.md](QA_CHECKLIST.md) | Tray, gates, updater, xterm | Pre-release |
| **Security** | Manual + external review (v1.0) | Pen test, token bypass attempts | v1.0 |

---

## Rust tests (`src-tauri`)

Run: `cd src-tauri && cargo test`

| Module | What it guards |
|--------|----------------|
| `hitl_token` | Issue/verify, replay rejection, boolean bypass blocked |
| `path_token` | Same for filesystem scope |
| `privilege` | Injection patterns, file-write detection |
| `shell_session` | Marker parsing, command validation |
| `shell_sandbox` | Escape helpers, sandbox level constants |
| `error` | Stable JSON error codes |
| `env_config` | Cloud base URL normalization |

Optional feature:

```bash
cargo test --features embedded-llm
```

---

## Frontend tests (`src/`)

Run: `npm run test`

| File | What it guards |
|------|----------------|
| `errors.test.ts` | `parseInvokeError`, user-facing formatting |
| `models.test.ts` | Cloud model pickers, custom endpoint models |
| `agentLoop.test.ts` | Mock cloud turn → `shell_run` → completion; HITL deny |

### Gaps (future)

- Component tests (React Testing Library) for gate modals
- Playwright/Tauri WebDriver for one happy-path agent run
- Updater E2E against a signed test release

---

## Manual QA (required before release)

Use [QA_CHECKLIST.md](QA_CHECKLIST.md) and [CROSS_PLATFORM_CHECKLIST.md](CROSS_PLATFORM_CHECKLIST.md).

Minimum smoke:

```
npm run test && cd src-tauri && cargo test && npm run build
npm run tauri dev   # interactive
```

Verify on **each target OS** you ship: macOS, Windows, Linux (X11 + Wayland if possible).

---

## CI matrix

[`.github/workflows/build.yml`](../.github/workflows/build.yml):

- macOS universal
- Linux x86_64
- Linux ARM64 (`ubuntu-24.04-arm`)
- Windows

Release workflow adds signed updater JSON when keys are configured — see [UPDATER.md](UPDATER.md).

---

## Adding tests

1. **Safety path changed?** Add or extend Rust tests in the same module.
2. **Agent loop / IPC contract changed?** Extend `agentLoop.test.ts` mocks.
3. **New user-facing error code?** Add case to `errors.test.ts`.
4. **Doc-only change?** `npm run docs:export`; no test required.

See [CONTRIBUTING.md](../CONTRIBUTING.md) for PR checklist.

---

## Related

- [TEST_NOTES.md](TEST_NOTES.md) — last local run log
- [QA_CHECKLIST.md](QA_CHECKLIST.md) — release sign-off
- [CODE_REVIEW.md](CODE_REVIEW.md) — architecture for reviewers

---

Built with ❤️ by [Gnomad Studio](https://gnomadstudio.org) 🦙
