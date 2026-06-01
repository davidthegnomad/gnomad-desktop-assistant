#!/usr/bin/env bash
# Build and optionally install Gnomad Flatpak locally.
# Prerequisites: flatpak, flatpak-builder, flathub remote, Node/Rust SDK extensions.
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
MANIFEST="$ROOT/packaging/flatpak/com.gnomadstudio.Gnomad.yml"
BUILD_DIR="${FLATPAK_BUILD_DIR:-$ROOT/packaging/flatpak/build-dir}"
REPO_DIR="${FLATPAK_REPO_DIR:-$ROOT/packaging/flatpak/repo}"

echo "==> Ensuring Flathub remote and SDK extensions..."
flatpak remote-add --if-not-exists flathub https://dl.flathub.org/repo/flathub.flatpakrepo
flatpak install -y flathub org.freedesktop.Platform//23.08 org.freedesktop.Sdk//23.08 \
  org.freedesktop.Sdk.Extension.node20//23.08 org.freedesktop.Sdk.Extension.rust-stable//23.08

echo "==> Building Flatpak (this may take 15–30+ minutes on first run)..."
flatpak-builder --force-clean --repo="$REPO_DIR" "$BUILD_DIR" "$MANIFEST"

if [[ "${1:-}" == "--install" ]]; then
  echo "==> Installing user Flatpak..."
  flatpak-builder --user --install --force-clean "$BUILD_DIR" "$MANIFEST"
  echo "Installed: flatpak run com.gnomadstudio.Gnomad"
else
  echo "Build complete. Install with: $0 --install"
  echo "Or export bundle: flatpak build-bundle $REPO_DIR gnomad.flatpak com.gnomadstudio.Gnomad"
fi
