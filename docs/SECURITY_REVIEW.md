# Security Review Checklist — Gnomad Desktop Assistant

**Version:** 0.2.0-beta.1  
**Type:** Structured internal review (substitute for external pen test until v1.0 GA)  
**Last updated:** June 2026

Use this checklist before beta/GA releases. External penetration testing is still recommended for v1.0 enterprise deployments.

---

## 1. Shell and privilege

| # | Check | Pass |
|---|--------|------|
| 1.1 | No IPC path executes shell without `shell_session_run` + validation | ☐ |
| 1.2 | Legacy `execute_shell_command` removed from invoke handler | ☐ |
| 1.3 | Risky commands require HITL token (not boolean bypass) | ☐ |
| 1.4 | HITL tokens are single-use and command-bound | ☐ |
| 1.5 | YOLO sandbox active when enabled (platform-appropriate level) | ☐ |
| 1.6 | Injection patterns blocked in `privilege.rs` heuristics | ☐ |

---

## 2. Filesystem

| # | Check | Pass |
|---|--------|------|
| 2.1 | Out-of-workspace access requires Path Gate token | ☐ |
| 2.2 | Path tokens reject boolean bypass | ☐ |
| 2.3 | `fs_write` preferred over shell redirects (UI hint) | ☐ |
| 2.4 | Audit log records shell `sandboxed` and `success` | ☐ |

---

## 3. Secrets and network

| # | Check | Pass |
|---|--------|------|
| 3.1 | API keys in OS keychain / credential store only | ☐ |
| 3.2 | No secrets in repo, logs, or error-log.jsonl by default | ☐ |
| 3.3 | Cloud traffic uses HTTPS; base URL configurable | ☐ |
| 3.4 | Updater verifies artifact signatures (real pubkey, not placeholder) | ☐ |

---

## 4. Supply chain

| # | Check | Pass |
|---|--------|------|
| 4.1 | `npm ci` / lockfile in CI | ☐ |
| 4.2 | Release artifacts built on GitHub Actions with pinned actions | ☐ |
| 4.3 | macOS builds signed; notarized before enterprise distribution | ☐ |
| 4.4 | CHANGELOG documents security-relevant changes | ☐ |

---

## 5. Privacy

| # | Check | Pass |
|---|--------|------|
| 5.1 | Error log opt-in only | ☐ |
| 5.2 | Voice input opt-in; user informed of browser STT | ☐ |
| 5.3 | No telemetry to Gnomad servers by default | ☐ |

---

## 6. Regression tests

```bash
npm run test
cd src-tauri && cargo test
npm run verify:updater   # before release with real keys
```

---

## Sign-off

| Role | Name | Date |
|------|------|------|
| Maintainer | | |
| Reviewer | | |

---

## Related

- [SECURITY_MODEL.md](SECURITY_MODEL.md)
- [QA_CHECKLIST.md](QA_CHECKLIST.md)
- [RELEASE_RUNBOOK.md](RELEASE_RUNBOOK.md)

---

Built with ❤️ by [Gnomad Studio](https://gnomadstudio.org) 🦙
