#!/usr/bin/env bash
# Report owner-only blockers — see HUMAN.md
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
CONF="$ROOT/src-tauri/tauri.conf.json"
PLACEHOLDER="dW50cnVzdGVkIGNvbW1lbnQ6IHRhdXJpLXVwZGF0ZXIgcHVibGljIGtleSByZXBsYWNlIG1l"

echo "==> Gnomad human preflight (see HUMAN.md)"
echo ""

BLOCKERS=0
WARNINGS=0

check_fail() {
  echo "  ✗ $1"
  BLOCKERS=$((BLOCKERS + 1))
}

check_warn() {
  echo "  ⚠ $1"
  WARNINGS=$((WARNINGS + 1))
}

check_ok() {
  echo "  ✓ $1"
}

# Updater pubkey
echo "Updater signing"
if grep -q "$PLACEHOLDER" "$CONF" 2>/dev/null; then
  check_fail "Placeholder pubkey in tauri.conf.json — run: npm run setup:updater-keys"
else
  check_ok "Updater pubkey is not the placeholder"
fi

KEY_PATH="${TAURI_SIGNER_KEY_PATH:-$HOME/.tauri/gnomad-updater.key}"
if [[ -f "$KEY_PATH" ]]; then
  check_ok "Private key file exists at $KEY_PATH"
else
  check_warn "No private key at $KEY_PATH (generate with npm run setup:updater-keys)"
fi

echo ""
echo "Release tag"
if git ls-remote --tags origin "refs/tags/v0.2.0-beta.1" 2>/dev/null | grep -q v0.2.0-beta.1; then
  check_ok "Tag v0.2.0-beta.1 exists on origin"
else
  check_warn "Tag v0.2.0-beta.1 not on origin — push tag when keys are ready (HUMAN.md §3)"
fi

echo ""
echo "GitHub CLI (optional)"
if command -v gh >/dev/null 2>&1; then
  if gh auth status >/dev/null 2>&1; then
    REPO="${GITHUB_REPOSITORY:-davidthegnomad/gnomad-desktop-assistant}"
    for secret in TAURI_SIGNING_PRIVATE_KEY TAURI_SIGNING_PRIVATE_KEY_PASSWORD; do
      if gh secret list --repo "$REPO" 2>/dev/null | awk '{print $1}' | grep -qx "$secret"; then
        check_ok "GitHub secret $secret is set"
      else
        check_warn "GitHub secret $secret not found — add in repo Settings"
      fi
    done
    for secret in APPLE_ID APPLE_PASSWORD APPLE_TEAM_ID; do
      if gh secret list --repo "$REPO" 2>/dev/null | awk '{print $1}' | grep -qx "$secret"; then
        check_ok "GitHub secret $secret is set (notarization)"
      else
        check_warn "GitHub secret $secret not set (optional until enterprise macOS)"
      fi
    done
  else
    check_warn "gh not authenticated — run: gh auth login (to check Actions secrets)"
  fi
else
  check_warn "gh CLI not installed — cannot verify GitHub Actions secrets"
fi

echo ""
echo "Apple notarization (local)"
if [[ -n "${APPLE_ID:-}" && -n "${APPLE_PASSWORD:-}" && -n "${APPLE_TEAM_ID:-}" ]]; then
  check_ok "APPLE_* env vars set in this shell"
else
  check_warn "APPLE_ID / APPLE_PASSWORD / APPLE_TEAM_ID not set in this shell"
fi

echo ""
echo "---"
if [[ "$BLOCKERS" -gt 0 ]]; then
  echo "$BLOCKERS blocker(s), $WARNINGS warning(s). See HUMAN.md."
  exit 1
fi
if [[ "$WARNINGS" -gt 0 ]]; then
  echo "0 blockers, $WARNINGS warning(s). See HUMAN.md for optional next steps."
  exit 0
fi
echo "All automated checks passed. Review HUMAN.md for manual sign-offs (security, WCAG, pen test)."
exit 0
