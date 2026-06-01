#!/usr/bin/env bash
# Download a small GGUF model for embedded-llm local development.
# Usage: bash scripts/download-gguf-model.sh [output_directory]
set -euo pipefail

DEFAULT_URL="${GGUF_URL:-https://huggingface.co/Qwen/Qwen2.5-Coder-1.5B-Instruct-GGUF/resolve/main/qwen2.5-coder-1.5b-instruct-q4_k_m.gguf}"
OUT_DIR="${1:-${HOME}/.gnomad/models}"
FILENAME="$(basename "${DEFAULT_URL%%\?*}")"
DEST="${OUT_DIR}/${FILENAME}"

mkdir -p "$OUT_DIR"

if [[ -f "$DEST" ]]; then
  echo "Already exists: $DEST"
  echo "Set this path in Settings → Agent access → GGUF path"
  exit 0
fi

echo "==> Downloading GGUF to $DEST"
echo "    URL: $DEFAULT_URL"
echo "    (This may take several minutes.)"

if command -v curl >/dev/null 2>&1; then
  curl -L --progress-bar -o "$DEST" "$DEFAULT_URL"
elif command -v wget >/dev/null 2>&1; then
  wget -O "$DEST" "$DEFAULT_URL"
else
  echo "Install curl or wget to download."
  exit 1
fi

echo ""
echo "Done. GGUF path for Settings:"
echo "  $DEST"
echo ""
echo "Run: npm run tauri:dev:embedded"
