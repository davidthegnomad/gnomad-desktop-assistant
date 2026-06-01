#!/usr/bin/env bash
# Submit Gnomad macOS DMG to Apple notary service and staple the ticket.
# Requires: APPLE_ID, APPLE_PASSWORD (app-specific), APPLE_TEAM_ID
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
BUNDLE_DIR="$ROOT/src-tauri/target/release/bundle"

if [[ "$(uname -s)" != "Darwin" ]]; then
  echo "notarize-macos.sh must run on macOS." >&2
  exit 1
fi

for var in APPLE_ID APPLE_PASSWORD APPLE_TEAM_ID; do
  if [[ -z "${!var:-}" ]]; then
    echo "Missing required env: $var" >&2
    exit 1
  fi
done

if [[ -n "${GNOMAD_DMG:-}" ]]; then
  DMG="$GNOMAD_DMG"
else
  DMG="$(find "$BUNDLE_DIR/macos" -maxdepth 1 -name '*.dmg' 2>/dev/null | head -1 || true)"
  if [[ -z "$DMG" ]]; then
    DMG="$(find "$BUNDLE_DIR/dmg" -maxdepth 1 -name '*.dmg' 2>/dev/null | head -1 || true)"
  fi
fi

if [[ -z "$DMG" || ! -f "$DMG" ]]; then
  echo "No .dmg found. Run npm run tauri:build:mac first or set GNOMAD_DMG." >&2
  exit 1
fi

APP="$(find "$BUNDLE_DIR/macos" -maxdepth 1 -name '*.app' 2>/dev/null | head -1 || true)"
if [[ -n "$APP" && -d "$APP" ]]; then
  echo "Verifying codesign on $APP …"
  codesign --verify --deep --strict --verbose=2 "$APP"
fi

echo "Submitting $DMG to Apple notary …"
SUBMIT_OUT="$(mktemp)"
xcrun notarytool submit "$DMG" \
  --apple-id "$APPLE_ID" \
  --password "$APPLE_PASSWORD" \
  --team-id "$APPLE_TEAM_ID" \
  --wait \
  --output-format json > "$SUBMIT_OUT"

STATUS="$(python3 -c "import json,sys; print(json.load(open(sys.argv[1])).get('status',''))" "$SUBMIT_OUT")"
echo "Notarization status: $STATUS"

if [[ "$STATUS" != "Accepted" ]]; then
  echo "Notarization failed. Log:" >&2
  cat "$SUBMIT_OUT" >&2
  rm -f "$SUBMIT_OUT"
  exit 1
fi

rm -f "$SUBMIT_OUT"

echo "Stapling ticket to $DMG …"
xcrun stapler staple "$DMG"
xcrun stapler validate "$DMG"

echo "Done. Notarized DMG: $DMG"
