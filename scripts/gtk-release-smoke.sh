#!/usr/bin/env bash
# Build GTK packages and smoke-test the .deb on this machine (no root required).
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"

echo "==> gtk-package-linux"
bash scripts/gtk-package-linux.sh

VERSION="${GTK_VERSION:-$(python3 - <<'PY'
import json, subprocess
data = json.loads(subprocess.check_output(
    ["cargo", "metadata", "--format-version", "1", "--no-deps"], text=True))
for pkg in data["packages"]:
    if pkg["name"] == "gnomad-gtk":
        print(pkg["version"])
        break
PY
)}"

ARCH="$(uname -m)"
case "$ARCH" in
  x86_64) DEB_ARCH=amd64 ;;
  aarch64) DEB_ARCH=arm64 ;;
  *) echo "unsupported arch: $ARCH"; exit 1 ;;
esac

DEB="$ROOT/target/gtk-packages/gnomad_${VERSION}_${DEB_ARCH}.deb"
APPIMAGE="$ROOT/target/gtk-packages/gnomad_${VERSION}_${DEB_ARCH}.AppImage"
EXTRACT="$ROOT/target/gtk-packages/smoke-root"

test -f "$DEB" || { echo "missing $DEB"; exit 1; }
test -f "$APPIMAGE" || { echo "missing $APPIMAGE"; exit 1; }
test -x "$APPIMAGE" || chmod +x "$APPIMAGE"

echo "==> extract .deb"
rm -rf "$EXTRACT"
mkdir -p "$EXTRACT"
dpkg-deb -x "$DEB" "$EXTRACT"

BIN="$EXTRACT/usr/bin/gnomad"
test -x "$BIN" || { echo "missing packaged binary"; exit 1; }

echo "==> gnomad --doctor (extracted .deb binary)"
"$BIN" --doctor

echo "==> gnomad --migrate-audit"
"$BIN" --migrate-audit

echo "==> package contents"
dpkg-deb -c "$DEB" | head -20

echo ""
echo "GTK release smoke OK"
echo "  deb:      $DEB"
echo "  appimage: $APPIMAGE"
echo ""
echo "Tag a release with: git tag v${VERSION}-gtk && git push origin v${VERSION}-gtk"
