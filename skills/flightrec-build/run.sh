#!/usr/bin/env bash
# run.sh — flightrec-build skill entrypoint
# Run from the repository root: bash skills/flightrec-build/run.sh

set -euo pipefail

export PATH="$HOME/.cargo/bin:$PATH"

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$ROOT"

echo "==> Step 1: Format check (cargo fmt --all --check)"
cargo fmt --all --check

echo "==> Step 2: Lint (cargo clippy --all-targets -- -D warnings)"
cargo clippy --all-targets -- -D warnings

echo "==> Step 3: Tests (cargo test --all-targets)"
cargo test --all-targets

echo "==> Step 4: Release build (cargo build --release)"
cargo build --release

echo ""
echo "Build complete. Binary: target/release/flightrec"
