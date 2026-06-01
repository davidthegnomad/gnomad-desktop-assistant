# Release Runbook — Gnomad Desktop Assistant

**Audience:** Maintainers cutting alpha/beta releases  
**Last updated:** June 2026

---

## Overview

Releases are **tag-driven**. Pushing a `v*` tag (or using workflow dispatch) runs [`.github/workflows/release.yml`](../.github/workflows/release.yml), builds installers for all platforms, and attaches them to a GitHub Release.

| Platform | Artifacts |
|----------|-----------|
| macOS | Universal `.dmg` |
| Linux x86_64 | `.deb`, `.rpm`, `.AppImage` |
| Linux ARM64 | `.deb`, `.AppImage` |
| Windows | `.msi`, NSIS `.exe` |

In-app updates require signed artifacts — see [UPDATER.md](UPDATER.md).

---

## Pre-release checklist

1. **Version bump** — align `package.json`, `src-tauri/tauri.conf.json`, and `src/lib/brand.ts` (if applicable).
2. **CHANGELOG** — move `[Unreleased]` items under the new version heading.
3. **Docs** — `npm run docs:export` after any doc edits; verify [GitHub Pages](https://davidthegnomad.github.io/gnomad-desktop-assistant/) links.
4. **Local QA** — `npm run test`, `cd src-tauri && cargo test`, `npm run build`.
5. **Manual smoke** (see [QA_CHECKLIST.md](QA_CHECKLIST.md)):
   - Cloud + local chat
   - Agent shell + fs tools
   - Sudo Gate / Path Gate tokens
   - Settings → Updates (check only; full install needs signed release)
6. **Updater keys** — run `npm run verify:updater` (must pass before tagging); set CI secrets `TAURI_SIGNING_PRIVATE_KEY` and `TAURI_SIGNING_PRIVATE_KEY_PASSWORD` — see [UPDATER.md](UPDATER.md).

---

## Cut a release

### 1. Merge to `main`

All release work should be on `main` (or the branch your workflows watch).

### 2. Create and push a tag

```bash
git tag -a v0.2.0-beta.1 -m "Gnomad v0.2.0-beta.1"
git push origin v0.2.0-beta.1
```

Or use **Actions → Release → Run workflow** with a tag name (creates/updates the release for that tag).

### 3. Monitor CI

Open the **Release** workflow run. Expect four matrix jobs:

- `macos-latest` — universal DMG
- `ubuntu-22.04` — Linux x86_64 deb/rpm/AppImage
- `ubuntu-24.04-arm` — Linux ARM64 deb/AppImage
- `windows-latest` — MSI + NSIS

Each job uploads assets via `tauri-action` with `includeUpdaterJson: true`.

### 4. Verify GitHub Release

- Assets named `gnomad-[name]-[version]-[platform][ext]`
- `latest.json` present (updater manifest)
- Release notes readable; mark **pre-release** for alpha/beta

### 5. Post-release verification

| Check | How |
|-------|-----|
| macOS install | Open DMG, drag to Applications, launch |
| Linux x86_64 | `sudo dpkg -i gnomad_*.deb` or run AppImage |
| Linux ARM64 | Same on ARM board / VM |
| Windows | Run MSI or setup.exe |
| Updater | Install previous build → Settings → Updates → Install (requires valid signing keys) |

Document results in [QA_CHECKLIST.md](QA_CHECKLIST.md) or a release issue.

---

## Hotfix workflow

1. Branch from the release tag or `main`.
2. Fix + tests + CHANGELOG entry.
3. Merge to `main`.
4. Tag patch version (`v0.2.0-beta.2`).

Do **not** force-push tags that users may already have installed.

---

## Rollback

GitHub Releases cannot be “un-published” cleanly for users who already downloaded. For a bad build:

1. Mark the release as **pre-release** or add a prominent warning in release notes.
2. Publish a new patch tag with the fix.
3. If updater keys were compromised, rotate minisign keys and update `pubkey` in `tauri.conf.json` (users on old builds will not auto-update until they install manually once).

---

## CI secrets reference

| Secret | Required for | Notes |
|--------|----------------|-------|
| `GITHUB_TOKEN` | All releases | Provided by Actions |
| `TAURI_SIGNING_PRIVATE_KEY` | Signed updates | Minisign private key contents |
| `TAURI_SIGNING_PRIVATE_KEY_PASSWORD` | Signed updates | Empty string if no password |

---

## Related docs

- [UPDATER.md](UPDATER.md) — key generation and channels
- [BUILD_PLATFORMS.md](BUILD_PLATFORMS.md) — local build commands
- [LINUX_PACKAGES.md](LINUX_PACKAGES.md) — install and ARM64 notes
- [QA_CHECKLIST.md](QA_CHECKLIST.md) — sign-off matrix

---

Built with ❤️ by [Gnomad Studio](https://gnomadstudio.org) 🦙
