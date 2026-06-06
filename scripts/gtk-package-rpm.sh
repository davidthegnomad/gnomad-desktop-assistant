#!/usr/bin/env bash
# Build Fedora/Nobara RPM for gnomad-gtk from the workspace spec.
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"

VERSION="${GTK_VERSION:-0.1.0}"
NAME=gnomad
STAGING="$ROOT/target/rpm-staging"
TARBALL="$ROOT/target/${NAME}-${VERSION}.tar.gz"

echo "==> cargo build -p gnomad-gtk --release"
cargo build -p gnomad-gtk --release

echo "==> stage source tree"
rm -rf "$STAGING/${NAME}-${VERSION}"
mkdir -p "$STAGING/${NAME}-${VERSION}"
rsync -a \
  --exclude target \
  --exclude node_modules \
  --exclude .git \
  Cargo.toml Cargo.lock \
  crates/ \
  "$STAGING/${NAME}-${VERSION}/"
cp -a crates/gnomad-gtk/packaging "$STAGING/${NAME}-${VERSION}/"
cp -a crates/gnomad-gtk/resources "$STAGING/${NAME}-${VERSION}/crates/gnomad-gtk/"

tar -C "$STAGING" -czf "$TARBALL" "${NAME}-${VERSION}"

if ! command -v rpmbuild >/dev/null 2>&1; then
  echo "rpmbuild not found — tarball ready at $TARBALL"
  echo "Install rpm-build, then: rpmbuild -bb crates/gnomad-gtk/packaging/gnomad-gtk.spec"
  exit 0
fi

echo "==> rpmbuild"
mkdir -p "$HOME/rpmbuild/SOURCES"
cp "$TARBALL" "$HOME/rpmbuild/SOURCES/"
rpmbuild -bb "$ROOT/crates/gnomad-gtk/packaging/gnomad-gtk.spec"

echo "==> RPM artifacts in ~/rpmbuild/RPMS/*/"
