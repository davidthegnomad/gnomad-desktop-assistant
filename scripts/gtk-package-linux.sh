#!/usr/bin/env bash
# Build gnomad release binary and produce .deb + AppImage artifacts.
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"

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
  x86_64) DEB_ARCH=amd64; DEPLOY_ARCH=x86_64 ;;
  aarch64) DEB_ARCH=arm64; DEPLOY_ARCH=aarch64 ;;
  *) echo "unsupported arch: $ARCH"; exit 1 ;;
esac

OUT_DIR="$ROOT/target/gtk-packages"
PKG_ROOT="$OUT_DIR/gnomad_${VERSION}_${DEB_ARCH}"
APPDIR="$OUT_DIR/Gnomad.AppDir"

echo "==> cargo test -p gnomad-core"
cargo test -p gnomad-core

echo "==> cargo build -p gnomad-gtk --release"
cargo build -p gnomad-gtk --release

BIN="$ROOT/target/release/gnomad"
if [[ ! -x "$BIN" ]]; then
  echo "missing release binary: $BIN"
  exit 1
fi

rm -rf "$OUT_DIR"
mkdir -p "$OUT_DIR"

echo "==> build .deb"
rm -rf "$PKG_ROOT"
mkdir -p "$PKG_ROOT/DEBIAN"
mkdir -p "$PKG_ROOT/usr/bin"
mkdir -p "$PKG_ROOT/usr/share/applications"
mkdir -p "$PKG_ROOT/usr/share/doc/gnomad"
mkdir -p "$PKG_ROOT/usr/share/icons/hicolor/128x128/apps"

install -m 0755 "$BIN" "$PKG_ROOT/usr/bin/gnomad"
install -m 0644 crates/gnomad-gtk/packaging/com.gnomadstudio.gnomad.desktop \
  "$PKG_ROOT/usr/share/applications/com.gnomadstudio.gnomad.desktop"
install -m 0644 crates/gnomad-gtk/resources/GNOMAD_HELP.md \
  "$PKG_ROOT/usr/share/doc/gnomad/GNOMAD_HELP.md"
install -m 0644 crates/gnomad-gtk/icons/128x128.png \
  "$PKG_ROOT/usr/share/icons/hicolor/128x128/apps/com.gnomadstudio.gnomad.png"

cat >"$PKG_ROOT/DEBIAN/control" <<EOF
Package: gnomad
Version: ${VERSION}
Architecture: ${DEB_ARCH}
Maintainer: Gnomad Studio <dev@gnomadstudio.org>
Depends: libgtk-4-1 (>= 4.14), libadwaita-1-0 (>= 1.5)
Recommends: libvte-2.91-gtk4, wl-clipboard, bubblewrap, wmctrl, ollama
Description: Gnomad Linux-native desktop assistant (GTK4)
 Linux-first Gnomad shell using GTK 4 and Libadwaita (no WebKitGTK).
EOF

DEB_FILE="$OUT_DIR/gnomad_${VERSION}_${DEB_ARCH}.deb"
dpkg-deb --build "$PKG_ROOT" "$DEB_FILE"
echo "Wrote $DEB_FILE"

echo "==> build AppImage"
rm -rf "$APPDIR"
mkdir -p "$APPDIR/usr/bin" "$APPDIR/usr/share/applications" \
  "$APPDIR/usr/share/icons/hicolor/128x128/apps"

install -m 0755 "$BIN" "$APPDIR/usr/bin/gnomad"
install -m 0644 crates/gnomad-gtk/packaging/com.gnomadstudio.gnomad.desktop \
  "$APPDIR/usr/share/applications/com.gnomadstudio.gnomad.desktop"
install -m 0644 crates/gnomad-gtk/icons/128x128.png \
  "$APPDIR/usr/share/icons/hicolor/128x128/apps/com.gnomadstudio.gnomad.png"

cp crates/gnomad-gtk/packaging/com.gnomadstudio.gnomad.desktop "$APPDIR/"
cp crates/gnomad-gtk/icons/128x128.png "$APPDIR/com.gnomadstudio.gnomad.png"

cat >"$APPDIR/AppRun" <<'EOF'
#!/usr/bin/env bash
HERE="$(dirname "$(readlink -f "$0")")"
export GSK_RENDERER="${GSK_RENDERER:-cairo}"
exec "${HERE}/usr/bin/gnomad" "$@"
EOF
chmod +x "$APPDIR/AppRun"

LINUXDEPLOY="$OUT_DIR/linuxdeploy-${DEPLOY_ARCH}.AppImage"
GTK_PLUGIN="$OUT_DIR/linuxdeploy-plugin-gtk-${DEPLOY_ARCH}.AppImage"
if [[ ! -x "$LINUXDEPLOY" ]]; then
  curl -fsSL -o "$LINUXDEPLOY" \
    "https://github.com/linuxdeploy/linuxdeploy/releases/download/continuous/linuxdeploy-${DEPLOY_ARCH}.AppImage"
  chmod +x "$LINUXDEPLOY"
fi
if [[ ! -x "$GTK_PLUGIN" ]]; then
  curl -fsSL -o "$GTK_PLUGIN" \
    "https://github.com/linuxdeploy/linuxdeploy-plugin-gtk/releases/download/continuous/linuxdeploy-plugin-gtk-${DEPLOY_ARCH}.AppImage"
  chmod +x "$GTK_PLUGIN"
fi

export ARCH="$DEPLOY_ARCH"
APPIMAGE_FILE="$OUT_DIR/gnomad_${VERSION}_${DEB_ARCH}.AppImage"
"$LINUXDEPLOY" --appdir "$APPDIR" --plugin gtk --output appimage -o "$APPIMAGE_FILE"
chmod +x "$APPIMAGE_FILE"
echo "Wrote $APPIMAGE_FILE"

echo "==> gtk package build OK"
ls -la "$OUT_DIR"/*.deb "$OUT_DIR"/*.AppImage
