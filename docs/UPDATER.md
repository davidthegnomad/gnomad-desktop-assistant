# Auto-updater signing keys

Gnomad uses the [Tauri updater plugin](https://v2.tauri.app/plugin/updater/) with minisign-signed release artifacts.

## One-time setup

1. Generate a key pair (keep the private key secret):

```bash
npm run setup:updater-keys
# or manually:
cd src-tauri && npx tauri signer generate -w ~/.tauri/gnomad-updater.key
```

2. Copy the **public** key contents into `src-tauri/tauri.conf.json` → `plugins.updater.pubkey` (full string, not a file path).

3. Add GitHub Actions secrets for release builds:

| Secret | Value |
|--------|--------|
| `TAURI_SIGNING_PRIVATE_KEY` | Contents of the private key file |
| `TAURI_SIGNING_PRIVATE_KEY_PASSWORD` | Key password (empty string if none) |

## Channels

| Channel | Endpoint |
|---------|----------|
| **Stable** | `…/releases/latest/download/latest.json` |
| **Beta** | `…/releases/download/v0.1.0-alpha/latest.json` (pre-releases) |

Users choose the channel in **Settings → Updates**. The release workflow uploads `latest.json` when `includeUpdaterJson: true`.

## Verify locally

Before tagging a release:

```bash
npm run verify:updater
```

After a tagged release, install the previous version and use **Check for updates** in Settings. Updates only install when the artifact signature matches the embedded public key.
