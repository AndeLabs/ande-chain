#!/usr/bin/env bash
set -euo pipefail

echo "=== Building Ande Chain Monorepo ==="

echo "1. Building Rust workspace..."
cargo build --workspace --all-features

echo "2. Building Solidity contracts..."
cd contracts && forge build && cd ..

echo "3. Generating bindings..."
cargo run -p generate-bindings

echo "✓ Build complete!"
