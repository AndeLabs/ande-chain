#!/usr/bin/env bash
set -euo pipefail

echo "=== Testing Ande Chain Monorepo ==="

echo "1. Testing Rust workspace..."
cargo test --workspace --all-features

echo "2. Testing Solidity contracts..."
cd contracts && forge test -vvv && cd ..

echo "3. Running integration tests..."
cargo test --package ande-tests

echo "✓ All tests passed!"
