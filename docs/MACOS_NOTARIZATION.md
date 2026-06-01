# macOS Notarization — Gnomad Desktop Assistant

**Status:** v1.0 P0 — not yet wired in CI  
**Audience:** Maintainers shipping signed `.dmg` builds for enterprise macOS

---

## Why notarize

Apple Gatekeeper requires **Developer ID signing** and **notarization** for apps distributed outside the Mac App Store. Without it, users see “Gnomad cannot be opened because the developer cannot be verified.”

Gnomad CI currently builds a universal `.dmg` on `macos-latest`. Notarization is a **manual or CI-secret step** before calling a build “enterprise ready.”

---

## Prerequisites

1. **Apple Developer Program** membership ($99/yr)
2. **Developer ID Application** certificate in Keychain Access
3. **App-specific password** for `notarytool` ([appleid.apple.com](https://appleid.apple.com) → Sign-In and Security → App-Specific Passwords)
4. **Team ID** — [developer.apple.com/account](https://developer.apple.com/account) → Membership details

Store these as secrets (never commit):

| Secret | Purpose |
|--------|---------|
| `APPLE_CERTIFICATE` | Base64 `.p12` Developer ID cert |
| `APPLE_CERTIFICATE_PASSWORD` | Export password for `.p12` |
| `APPLE_SIGNING_IDENTITY` | e.g. `Developer ID Application: Gnomad Studio (TEAMID)` |
| `APPLE_ID` | Apple ID email for notary |
| `APPLE_PASSWORD` | App-specific password |
| `APPLE_TEAM_ID` | 10-character team id |

---

## Local build + notarize

From repo root after a release build:

```bash
npm run tauri:build:mac
bash scripts/notarize-macos.sh
```

The script:

1. Finds `Gnomad.app` and `.dmg` under `src-tauri/target/release/bundle/`
2. Verifies codesign on the `.app`
3. Submits the `.dmg` to Apple via `xcrun notarytool submit`
4. Waits for approval and runs `xcrun stapler staple` on the `.dmg`

### Required environment

```bash
export APPLE_ID="you@example.com"
export APPLE_PASSWORD="xxxx-xxxx-xxxx-xxxx"   # app-specific password
export APPLE_TEAM_ID="XXXXXXXXXX"
# Optional override:
export GNOMAD_DMG="path/to/Gnomad_0.2.0_universal.dmg"
```

---

## CI integration (recommended for v1.0)

Add to `.github/workflows/release.yml` on the macOS job **after** `tauri build`:

```yaml
- name: Import signing certificate
  if: runner.os == 'macOS'
  env:
    APPLE_CERTIFICATE: ${{ secrets.APPLE_CERTIFICATE }}
    APPLE_CERTIFICATE_PASSWORD: ${{ secrets.APPLE_CERTIFICATE_PASSWORD }}
  run: |
    echo "$APPLE_CERTIFICATE" | base64 --decode > certificate.p12
    security create-keychain -p actions build.keychain
    security default-keychain -s build.keychain
    security unlock-keychain -p actions build.keychain
    security import certificate.p12 -k build.keychain -P "$APPLE_CERTIFICATE_PASSWORD" -T /usr/bin/codesign
    security set-key-partition-list -S apple-tool:,apple:,codesign: -s -k actions build.keychain

- name: Notarize DMG
  if: runner.os == 'macOS'
  env:
    APPLE_ID: ${{ secrets.APPLE_ID }}
    APPLE_PASSWORD: ${{ secrets.APPLE_PASSWORD }}
    APPLE_TEAM_ID: ${{ secrets.APPLE_TEAM_ID }}
  run: bash scripts/notarize-macos.sh
```

Tauri v2 can also sign during build when `APPLE_SIGNING_IDENTITY` and entitlements are configured — see [Tauri macOS code signing](https://v2.tauri.app/distribute/sign/macos/).

---

## Verify a notarized build

On a clean Mac (or after `spctl --assess`):

```bash
spctl -a -vv -t install path/to/Gnomad.dmg
codesign -dv --verbose=4 path/to/Gnomad.app
xcrun stapler validate path/to/Gnomad.dmg
```

Expected: `source=Notarized Developer ID`.

---

## Related

- [BUILD_PLATFORMS.md](BUILD_PLATFORMS.md) — macOS build commands
- [RELEASE_RUNBOOK.md](RELEASE_RUNBOOK.md) — release checklist
- [ENTERPRISE.md](ENTERPRISE.md) — MDM deployment

---

Built with ❤️ by [Gnomad Studio](https://gnomadstudio.org) 🦙
