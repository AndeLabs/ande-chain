#!/usr/bin/env bash
#
# Professional Migration Script for Ande Chain Monorepo
# Migrates content from separate repos (ande + ev-reth) into unified monorepo
#

set -euo pipefail

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Logging functions
log_info() { echo -e "${BLUE}[INFO]${NC} $1"; }
log_success() { echo -e "${GREEN}[✓]${NC} $1"; }
log_warn() { echo -e "${YELLOW}[⚠]${NC} $1"; }
log_error() { echo -e "${RED}[✗]${NC} $1"; }

# Configuration
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
MONOREPO_ROOT="$(dirname "$SCRIPT_DIR")"
PARENT_DIR="$(dirname "$MONOREPO_ROOT")"
ANDE_REPO="$PARENT_DIR/ande"
EV_RETH_REPO="$PARENT_DIR/ev-reth"

log_info "=== Ande Chain Repository Migration ==="
log_info "Monorepo root: $MONOREPO_ROOT"
log_info "Source repos: $ANDE_REPO, $EV_RETH_REPO"

# Phase 1: Validation
log_info "Phase 1: Validating source repositories..."

if [ ! -d "$ANDE_REPO" ]; then
    log_error "ande repository not found at $ANDE_REPO"
    exit 1
fi

if [ ! -d "$EV_RETH_REPO" ]; then
    log_error "ev-reth repository not found at $EV_RETH_REPO"
    exit 1
fi

log_success "Source repositories found"

# Phase 2: Copy contracts from ande
log_info "Phase 2: Migrating Solidity contracts from ande..."

if [ -d "$ANDE_REPO/contracts" ]; then
    cp -r "$ANDE_REPO/contracts"/* "$MONOREPO_ROOT/contracts/" 2>/dev/null || true
    log_success "Contracts migrated"
else
    log_warn "No contracts directory found in ande repo"
fi

# Phase 3: Copy crates from ev-reth
log_info "Phase 3: Migrating Rust crates from ev-reth..."

if [ -d "$EV_RETH_REPO/crates" ]; then
    # Copy evolve crate (main EVM customization)
    if [ -d "$EV_RETH_REPO/crates/evolve" ]; then
        cp -r "$EV_RETH_REPO/crates/evolve"/* "$MONOREPO_ROOT/crates/ande-evm/" 2>/dev/null || true
        log_success "EVM crate migrated"
    fi
    
    # Copy other crates as needed
    for crate_dir in "$EV_RETH_REPO/crates"/*; do
        crate_name=$(basename "$crate_dir")
        log_info "Found crate: $crate_name"
    done
else
    log_warn "No crates directory found in ev-reth repo"
fi

# Phase 4: Copy infrastructure
log_info "Phase 4: Migrating infrastructure configs..."

if [ -d "$ANDE_REPO/infra" ]; then
    cp -r "$ANDE_REPO/infra"/* "$MONOREPO_ROOT/infra/" 2>/dev/null || true
    log_success "Infrastructure configs migrated"
fi

# Phase 5: Copy specs
log_info "Phase 5: Migrating chain specifications..."

if [ -d "$EV_RETH_REPO/specs" ]; then
    cp -r "$EV_RETH_REPO/specs"/* "$MONOREPO_ROOT/specs/" 2>/dev/null || true
    log_success "Chain specs migrated"
fi

log_success "Migration complete!"
log_info ""
log_info "Next steps:"
log_info "1. Review migrated files"
log_info "2. Update Cargo.toml dependencies"
log_info "3. Run: cargo build --workspace"
log_info "4. Run: forge build"
log_info "5. Run: cargo test --workspace"
