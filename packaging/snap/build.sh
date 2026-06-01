#!/usr/bin/env bash
# Build Gnomad snap from a release .deb produced by Tauri.
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
cd "$ROOT"

if ! command -v snapcraft >/dev/null 2>&1; then
  echo "Install snapcraft: sudo snap install snapcraft --classic"
  exit 1
fi

if ! ls src-tauri/target/release/bundle/deb/*.deb >/dev/null 2>&1; then
  echo "==> Building .deb first..."
  npm run tauri:build:linux:deb
fi

echo "==> Building snap (classic confinement)..."
cd packaging/snap
snapcraft pack --destructive-mode --output gnomad.snap

echo "Done. Install with: sudo snap install --dangerous packaging/snap/gnomad.snap"
