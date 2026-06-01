# Test Notes — Gnomad Desktop Assistant

**Run date:** 2026-05-31  
**Environment:** macOS (darwin), project path with spaces

## Automated runs

| Command | Result | Notes |
|---------|--------|-------|
| `npm run build` | **PASS** | `tsc` + Vite 7 |
| `npm run test` | **PASS** | Vitest — 7 tests (`parseInvokeError`, error formatting) |
| `cargo test` (src-tauri) | **PASS (13 tests)** | error, hitl_token, path_token, privilege, shell_session, shell_sandbox |
| `cargo build` (src-tauri) | **PASS** | Debug profile; default features (no `embedded-llm`) |
| CI (`build.yml`) | **PASS** | macOS, Ubuntu, Windows matrix on push to main |

### Embedded LLM (optional)

```bash
npm run tauri:dev:embedded
# or: cd src-tauri && cargo test --features embedded-llm
```

Not run in default CI to keep matrix fast.

## Not run in this session (GUI / OS)

These require an interactive session and are **manual**:

| Test | How |
|------|-----|
| App launch | `npm run tauri dev` |
| Overlay toggle | ⌘⇧Space / Ctrl+Shift+Space |
| Tray menu | Show Gnomad / Settings / Quit |
| Wayland tray (Linux) | Left-click tray icon → menu |
| Sudo Gate + HITL token | Approve risky cmd → signed token path |
| Path Gate + path token | fs_read outside workspace → Allow once |
| xterm replay | Expand command card → Terminal view |
| Updates | Settings → Updates → Check for updates |
| YOLO sandbox | YOLO + experimental sandbox; verify blocked network reads |

## Manual checklist (copy for QA)

```
[ ] tauri dev starts without panic
[ ] Overlay hidden on launch; shortcut shows it
[ ] Cloud agent: tool loop runs shell + fs tools
[ ] Sudo Gate: unsigned bypass rejected; approve mints token
[ ] Path Gate: unsigned bypass rejected; approve mints token
[ ] Standard mode: workspace fs works; outside path gated
[ ] Settings → Updates: check returns message (even if no new release)
[ ] npm run test && cargo test pass locally
```

## Known test gaps

- No headless E2E (Tauri WebDriver not configured)
- No mocked cloud agent integration test in CI yet
- Updater install path requires signed release artifacts + real pubkey

See also [`QA_CHECKLIST.md`](QA_CHECKLIST.md) and [`CODE_REVIEW.md`](CODE_REVIEW.md).
