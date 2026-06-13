#!/usr/bin/env bash
# Build, test, and run gnomad-gtk --doctor (KDE-first CI/local gate).
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"

echo "==> cargo test -p gnomad-core"
cargo test -p gnomad-core

echo "==> cargo build -p gnomad-gtk --release"
cargo build -p gnomad-gtk --release

echo "==> gnomad --doctor"
./target/release/gnomad --doctor

echo "==> gtk preflight OK"
