#!/usr/bin/env bash
# KDE Wayland smoke helper — automated tray/process checks + manual checklist.
# Run on a graphical KDE session (Nobara/Fedora primary).
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"

BIN="$ROOT/target/release/gnomad"
TRAY_ID="com.gnomadstudio.gnomad"
PASS=0
FAIL=0
MANUAL=0

ok() { echo "  [✓] $1"; PASS=$((PASS + 1)); }
fail() { echo "  [✗] $1"; FAIL=$((FAIL + 1)); }
manual() { echo "  [ ] $1 (manual)"; MANUAL=$((MANUAL + 1)); }

echo "==> gtk:preflight"
bash scripts/gtk-preflight.sh

if [[ -z "${WAYLAND_DISPLAY:-}${DISPLAY:-}" ]]; then
  echo ""
  echo "No graphical session — skipping live tray smoke."
  echo "Run this script from a KDE desktop session for full checks."
  exit 0
fi

if [[ ! -x "$BIN" ]]; then
  fail "release binary missing at $BIN"
  exit 1
fi
ok "release binary present"

echo "==> launch gnomad (background, 20s max)"
"$BIN" &
APP_PID=$!
cleanup() {
  kill "$APP_PID" 2>/dev/null || true
  wait "$APP_PID" 2>/dev/null || true
}
trap cleanup EXIT

tray_registered=false
for _ in $(seq 1 40); do
  if ! kill -0 "$APP_PID" 2>/dev/null; then
    fail "gnomad exited before tray registered"
    break
  fi
  if command -v qdbus >/dev/null 2>&1; then
    if qdbus org.kde.StatusNotifierWatcher /StatusNotifierWatcher GetRegisteredStatusNotifierItems 2>/dev/null \
      | grep -q "$TRAY_ID"; then
      tray_registered=true
      break
    fi
  elif command -v busctl >/dev/null 2>&1; then
    if busctl --user call org.kde.StatusNotifierWatcher /StatusNotifierWatcher \
      org.kde.StatusNotifierWatcher GetRegisteredStatusNotifierItems 2>/dev/null \
      | grep -q "$TRAY_ID"; then
      tray_registered=true
      break
    fi
  elif command -v gdbus >/dev/null 2>&1; then
    if gdbus call --session --dest org.kde.StatusNotifierWatcher \
      --object-path /StatusNotifierWatcher \
      --method org.kde.StatusNotifierWatcher.GetRegisteredStatusNotifierItems 2>/dev/null \
      | grep -q "$TRAY_ID"; then
      tray_registered=true
      break
    fi
  fi
  sleep 0.5
done

if kill -0 "$APP_PID" 2>/dev/null; then
  ok "gnomad still running after startup"
else
  fail "gnomad crashed during startup"
fi

if $tray_registered; then
  ok "tray registered with StatusNotifierWatcher ($TRAY_ID)"
else
  fail "tray not seen on SNI watcher within 20s (check ksni / AppIndicator)"
fi

echo ""
echo "==> KDE interactive smoke — manual steps"
manual "Left-click tray → Window mode appears; composer is clickable"
manual "Right-click tray → menu items work; Quit exits cleanly"
manual "Pop Out mode → floating window stays above a normal app (XWayland + wmctrl)"
manual "Ctrl+Shift+Space toggles Window mode while Gnomad is focused"
manual "Help ? dialog opens; agent can answer setup questions"
manual "Attach a .md file via paperclip; send; model receives inlined text"

echo ""
echo "Summary: $PASS automated passed, $FAIL automated failed, $MANUAL manual items listed"
if [[ "$FAIL" -gt 0 ]]; then
  exit 1
fi
