#!/usr/bin/env bash
# Fail if tauri.conf.json still contains the placeholder updater public key.
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
CONF="$ROOT/src-tauri/tauri.conf.json"

if [[ ! -f "$CONF" ]]; then
  echo "Missing $CONF"
  exit 1
fi

PLACEHOLDER="dW50cnVzdGVkIGNvbW1lbnQ6IHRhdXJpLXVwZGF0ZXIgcHVibGljIGtleSByZXBsYWNlIG1l"

if grep -q "$PLACEHOLDER" "$CONF"; then
  echo "FAIL: Placeholder updater pubkey still in tauri.conf.json"
  echo "Run: npm run setup:updater-keys"
  echo "Then paste the public key into plugins.updater.pubkey"
  exit 1
fi

if ! grep -q '"pubkey"' "$CONF"; then
  echo "FAIL: No plugins.updater.pubkey found in tauri.conf.json"
  exit 1
fi

echo "OK: Updater pubkey appears configured (not placeholder)."
echo "Ensure GitHub secrets TAURI_SIGNING_PRIVATE_KEY and TAURI_SIGNING_PRIVATE_KEY_PASSWORD are set for releases."
