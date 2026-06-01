# Test Notes — Omni Taskbar AI

**Run date:** 2026-05-31  
**Environment:** macOS (darwin), project path with spaces

## Automated runs

| Command | Result | Notes |
|---------|--------|-------|
| `npm run build` | **PASS** | `tsc` + Vite 7; output ~210 KB JS bundle |
| `cargo test` (src-tauri) | **PASS (0 tests)** | No `#[test]` functions in crate |
| `cargo build` (src-tauri) | **PASS** | Debug profile |
| `cargo clippy -D warnings` | **FAIL** | Build script error: stale permissions path under `Desktop AI Project Folder/src-tauri/target/...` (wrong parent folder). Not a source defect; try `cargo clean` if it recurs |

## Not run in this session (GUI / OS)

These require an interactive Mac session and are **manual**:

| Test | How |
|------|-----|
| App launch | `npm run tauri dev` |
| Overlay toggle | ⌘⇧Space — window should show/hide |
| Tray menu | Show Overlay / Quit |
| Context strip | Active app, window title, clipboard update every ~2.5s |
| Cloud chat | Submit prompt → demo reply + `git --version` in thread |
| Local mode | Switch to Local → Run CLI with safe command (e.g. `echo hello`) |
| Sudo Gate | Local: `sudo ls` or `rm -rf test` → modal → approve/deny |
| Settings keychain | Save API key, restart app, confirm reload |
| Elevation test | Settings → Trigger Touch ID prompt (`echo 'Success'`) |
| Accessibility | Deny/revoke permission → banner → Open Preferences |
| Screenshot / automation | Invoke via devtools or future UI; needs Accessibility (+ possibly Screen Recording) |

## Manual checklist (copy for QA)

```
[ ] tauri dev starts without panic
[ ] Overlay hidden on launch; shortcut shows it
[ ] Tray icon visible (Accessory — no dock icon)
[ ] Context: app name matches frontmost app
[ ] Context: clipboard snippet updates after copy
[ ] Cloud mode: assistant responds (demo, not real LLM)
[ ] Local mode: echo hello → stdout in chat
[ ] Dangerous cmd: Sudo Gate appears before run
[ ] Deny on Sudo Gate: error message in chat
[ ] Approve admin cmd: verify stdout (watch for shape bug — see CODE_REVIEW.md)
[ ] API key persists across restart
```

## Known test gaps

- No CI workflow in repo.
- No headless E2E (Tauri WebDriver not configured).
- Safety logic untested — add Rust unit tests before relying on HITL in production.
