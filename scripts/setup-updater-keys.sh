#!/usr/bin/env bash
# Generate Tauri updater minisign keys and print next steps.
# Does NOT commit private keys. See docs/UPDATER.md.
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
KEY_PATH="${TAURI_SIGNER_KEY_PATH:-$HOME/.tauri/gnomad-updater.key}"
CONF="$ROOT/src-tauri/tauri.conf.json"

echo "==> Generating updater key pair at: $KEY_PATH"
mkdir -p "$(dirname "$KEY_PATH")"
cd "$ROOT/src-tauri"
npx tauri signer generate -w "$KEY_PATH"

PUBKEY_FILE="${KEY_PATH}.pub"
if [[ ! -f "$PUBKEY_FILE" ]]; then
  PUBKEY_FILE="${KEY_PATH%.key}.pub"
fi

if [[ -f "$PUBKEY_FILE" ]]; then
  PUBKEY="$(tr -d '\n' < "$PUBKEY_FILE")"
  echo ""
  echo "==> Public key (paste into tauri.conf.json → plugins.updater.pubkey):"
  echo "$PUBKEY"
  echo ""
  echo "==> GitHub Actions secrets:"
  echo "  TAURI_SIGNING_PRIVATE_KEY     = contents of $KEY_PATH"
  echo "  TAURI_SIGNING_PRIVATE_KEY_PASSWORD = (empty or your password)"
  echo ""
  echo "See docs/UPDATER.md and docs/RELEASE_RUNBOOK.md"
else
  echo "Public key file not found next to $KEY_PATH — check tauri signer output."
fi
