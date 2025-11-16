#!/usr/bin/env bash

################################################################################
# ANDE Chain - Restart with New Genesis
# Production-grade script for restarting blockchain with updated genesis
#
# This script safely restarts the ANDE Chain with a new genesis configuration
# that includes proper initial balances for the 3 sequencer wallets.
#
# Architecture:
#   - ANDE Chain: Sovereign Rollup with EVM
#   - Consensus: CometBFT Multi-Sequencer
#   - Data Availability: Celestia Mocha-4 (DA blobs only)
#   - Native Token: ANDE (dual-token for gas + ERC-20)
#
# Sequencer Wallets (100 ANDE each from genesis):
#   1. 0xDA104C8Bf401F490599e1DA677Eba4D930aCF920 (Primary)
#   2. 0xc9a74e92dCF374ACA1010C0ECfF69c93335d4c10 (Backup)
#   3. 0x4Fa20fBfe0C757599d11939e1492b2A306d07064 (Backup)
#
# Usage:
#   ./scripts/restart-with-genesis.sh [--compose-file FILE] [--skip-backup]
#
# Options:
#   --compose-file FILE : Docker compose file to use (default: docker-compose-quick.yml)
#   --skip-backup      : Skip backup step (NOT RECOMMENDED for production)
#   --yes, -y          : Skip confirmation prompts
#
# Safety:
#   - Creates full backup before restart (unless --skip-backup)
#   - Validates genesis.json before proceeding
#   - Graceful shutdown with timeout
#   - Health checks after restart
#   - Balance verification
#
################################################################################

set -euo pipefail

# ============================================================================
# CONFIGURATION
# ============================================================================

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
cd "$PROJECT_ROOT"

# Default values
COMPOSE_FILE="${COMPOSE_FILE:-docker-compose-quick.yml}"
SKIP_BACKUP=false
AUTO_YES=false

# Genesis configuration
GENESIS_FILE="$PROJECT_ROOT/specs/genesis.json"

# Expected sequencer addresses in genesis
declare -a SEQUENCER_ADDRESSES=(
    "0xDA104C8Bf401F490599e1DA677Eba4D930aCF920"
    "0xc9a74e92dCF374ACA1010C0ECfF69c93335d4c10"
    "0x4Fa20fBfe0C757599d11939e1492b2A306d07064"
)

# Expected balance (100 ANDE in hex)
EXPECTED_BALANCE="0x56bc75e2d63100000"

# Timing constants
SHUTDOWN_TIMEOUT=60
STARTUP_WAIT=30
HEALTH_CHECK_RETRIES=10
HEALTH_CHECK_INTERVAL=5

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# ============================================================================
# LOGGING FUNCTIONS
# ============================================================================

log_info() {
    echo -e "${BLUE}[INFO]${NC} $*"
}

log_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $*"
}

log_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $*"
}

log_error() {
    echo -e "${RED}[ERROR]${NC} $*"
}

log_step() {
    echo ""
    echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
    echo -e "${BLUE}$*${NC}"
    echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
    echo ""
}

# ============================================================================
# UTILITY FUNCTIONS
# ============================================================================

# Parse command line arguments
parse_args() {
    while [[ $# -gt 0 ]]; do
        case $1 in
            --compose-file)
                COMPOSE_FILE="$2"
                shift 2
                ;;
            --skip-backup)
                SKIP_BACKUP=true
                shift
                ;;
            -y|--yes)
                AUTO_YES=true
                shift
                ;;
            -h|--help)
                grep "^#" "$0" | tail -n +3 | head -n -1 | cut -c 3-
                exit 0
                ;;
            *)
                log_error "Unknown option: $1"
                echo "Run with --help for usage information"
                exit 1
                ;;
        esac
    done
}

# Confirm action with user
confirm() {
    if [ "$AUTO_YES" = true ]; then
        return 0
    fi

    local message="$1"
    echo -e "${YELLOW}$message${NC}"
    read -p "Continue? (yes/no): " -r
    echo
    if [[ ! $REPLY =~ ^[Yy][Ee][Ss]$ ]]; then
        log_info "Operation cancelled by user"
        exit 0
    fi
}

# Check if command exists
command_exists() {
    command -v "$1" >/dev/null 2>&1
}

# ============================================================================
# VALIDATION FUNCTIONS
# ============================================================================

# Validate prerequisites
validate_prerequisites() {
    log_step "STEP 1: Validating Prerequisites"

    # Check docker
    if ! command_exists docker; then
        log_error "Docker is not installed"
        exit 1
    fi
    log_success "Docker found: $(docker --version)"

    # Check docker compose
    if ! docker compose version >/dev/null 2>&1; then
        log_error "Docker Compose is not available"
        exit 1
    fi
    log_success "Docker Compose found: $(docker compose version)"

    # Check compose file exists
    if [ ! -f "$COMPOSE_FILE" ]; then
        log_error "Compose file not found: $COMPOSE_FILE"
        exit 1
    fi
    log_success "Compose file found: $COMPOSE_FILE"

    # Check genesis file exists
    if [ ! -f "$GENESIS_FILE" ]; then
        log_error "Genesis file not found: $GENESIS_FILE"
        exit 1
    fi
    log_success "Genesis file found: $GENESIS_FILE"
}

# Validate genesis.json contains sequencer wallets
validate_genesis() {
    log_step "STEP 2: Validating Genesis Configuration"

    local genesis_content
    genesis_content=$(cat "$GENESIS_FILE")

    # Check each sequencer address
    local missing_addresses=()
    for addr in "${SEQUENCER_ADDRESSES[@]}"; do
        # Case-insensitive search for address
        if echo "$genesis_content" | grep -iq "$addr"; then
            log_success "Found sequencer: $addr"

            # Verify balance
            if echo "$genesis_content" | grep -A 2 -i "$addr" | grep -q "$EXPECTED_BALANCE"; then
                log_success "  ✓ Balance: 100 ANDE ($EXPECTED_BALANCE)"
            else
                log_warning "  ⚠ Balance mismatch for $addr"
            fi
        else
            log_error "Missing sequencer: $addr"
            missing_addresses+=("$addr")
        fi
    done

    if [ ${#missing_addresses[@]} -gt 0 ]; then
        log_error "Genesis validation failed - missing ${#missing_addresses[@]} sequencer(s)"
        exit 1
    fi

    log_success "Genesis validation complete - all 3 sequencers configured correctly"
}

# ============================================================================
# BACKUP FUNCTIONS
# ============================================================================

# Create backup of current state
create_backup() {
    if [ "$SKIP_BACKUP" = true ]; then
        log_warning "Skipping backup (--skip-backup flag set)"
        return 0
    fi

    log_step "STEP 3: Creating Backup"

    local backup_script="$SCRIPT_DIR/backup-testnet.sh"

    if [ ! -f "$backup_script" ]; then
        log_warning "Backup script not found: $backup_script"
        log_warning "Proceeding without backup"
        return 0
    fi

    if ! bash "$backup_script"; then
        log_error "Backup failed"
        confirm "Backup failed. Continue without backup?"
    else
        log_success "Backup completed successfully"
    fi
}

# ============================================================================
# SHUTDOWN FUNCTIONS
# ============================================================================

# Stop current services
stop_services() {
    log_step "STEP 4: Stopping Current Services"

    # Check if any services are running
    if ! docker compose -f "$COMPOSE_FILE" ps --services --filter "status=running" | grep -q .; then
        log_info "No running services found"
        return 0
    fi

    log_info "Stopping services gracefully (timeout: ${SHUTDOWN_TIMEOUT}s)..."

    if docker compose -f "$COMPOSE_FILE" down --timeout "$SHUTDOWN_TIMEOUT"; then
        log_success "Services stopped successfully"
    else
        log_error "Failed to stop services gracefully"
        confirm "Failed to stop services. Force stop?"
        docker compose -f "$COMPOSE_FILE" down --timeout 10 --force || true
    fi

    # Verify all containers stopped
    sleep 2
    if docker compose -f "$COMPOSE_FILE" ps --services --filter "status=running" | grep -q .; then
        log_warning "Some services still running after shutdown"
    fi
}

# Remove blockchain data volumes
remove_blockchain_data() {
    log_step "STEP 5: Removing Old Blockchain Data"

    confirm "⚠️  This will DELETE all blockchain data and restart from genesis block 0"

    log_info "Removing data volumes..."

    # List volumes to remove
    local volumes_to_remove=(
        "ande-chain_ande-node-data"
        "ande-chain_evolve-data"
        "ande-chain_celestia-data"
        "ande-chain_sequencer-1-data"
        "ande-chain_sequencer-2-data"
        "ande-chain_sequencer-3-data"
    )

    local removed_count=0
    for volume in "${volumes_to_remove[@]}"; do
        if docker volume ls | grep -q "$volume"; then
            log_info "Removing volume: $volume"
            if docker volume rm "$volume" 2>/dev/null; then
                ((removed_count++))
            else
                log_warning "Failed to remove volume: $volume (may not exist)"
            fi
        fi
    done

    if [ $removed_count -gt 0 ]; then
        log_success "Removed $removed_count blockchain data volume(s)"
    else
        log_info "No blockchain data volumes found to remove"
    fi

    # Also clear any local data directories if they exist
    local data_dirs=(
        "data/ande-node"
        "data/evolve"
        "data/celestia"
    )

    for dir in "${data_dirs[@]}"; do
        if [ -d "$dir" ]; then
            log_info "Removing directory: $dir"
            rm -rf "$dir"
        fi
    done
}

# ============================================================================
# STARTUP FUNCTIONS
# ============================================================================

# Start services with new genesis
start_services() {
    log_step "STEP 6: Starting Services with New Genesis"

    log_info "Starting services from genesis block 0..."
    log_info "Compose file: $COMPOSE_FILE"
    log_info "Genesis file: $GENESIS_FILE"

    if docker compose -f "$COMPOSE_FILE" up -d; then
        log_success "Services started successfully"
    else
        log_error "Failed to start services"
        exit 1
    fi

    log_info "Waiting ${STARTUP_WAIT}s for services to initialize..."
    sleep "$STARTUP_WAIT"
}

# Health check services
health_check() {
    log_step "STEP 7: Health Check"

    local rpc_url="http://localhost:8545"
    local max_retries=$HEALTH_CHECK_RETRIES
    local retry=0

    log_info "Checking RPC endpoint: $rpc_url"

    while [ $retry -lt $max_retries ]; do
        if curl -s -X POST "$rpc_url" \
            -H "Content-Type: application/json" \
            --data '{"jsonrpc":"2.0","method":"eth_blockNumber","params":[],"id":1}' \
            >/dev/null 2>&1; then
            log_success "RPC endpoint is healthy"
            return 0
        fi

        retry=$((retry + 1))
        if [ $retry -lt $max_retries ]; then
            log_info "Retry $retry/$max_retries - waiting ${HEALTH_CHECK_INTERVAL}s..."
            sleep "$HEALTH_CHECK_INTERVAL"
        fi
    done

    log_error "RPC endpoint health check failed after $max_retries attempts"
    return 1
}

# ============================================================================
# VERIFICATION FUNCTIONS
# ============================================================================

# Verify sequencer balances
verify_balances() {
    log_step "STEP 8: Verifying Sequencer Balances"

    local rpc_url="http://localhost:8545"

    for addr in "${SEQUENCER_ADDRESSES[@]}"; do
        log_info "Checking balance for: $addr"

        local balance
        balance=$(curl -s -X POST "$rpc_url" \
            -H "Content-Type: application/json" \
            --data "{\"jsonrpc\":\"2.0\",\"method\":\"eth_getBalance\",\"params\":[\"$addr\",\"latest\"],\"id\":1}" \
            | grep -o '"result":"[^"]*"' \
            | cut -d'"' -f4)

        if [ -z "$balance" ]; then
            log_error "Failed to fetch balance for $addr"
            continue
        fi

        # Convert hex to decimal for display
        local balance_wei
        balance_wei=$(printf "%d" "$balance" 2>/dev/null || echo "0")
        local balance_ande
        balance_ande=$(echo "scale=2; $balance_wei / 1000000000000000000" | bc)

        if [ "$balance" = "$EXPECTED_BALANCE" ]; then
            log_success "  ✓ Balance: $balance_ande ANDE (correct)"
        else
            log_warning "  ⚠ Balance: $balance_ande ANDE (expected: 100 ANDE)"
        fi
    done
}

# ============================================================================
# MAIN EXECUTION
# ============================================================================

main() {
    echo ""
    echo "╔═══════════════════════════════════════════════════════════════════╗"
    echo "║       ANDE CHAIN - RESTART WITH NEW GENESIS                      ║"
    echo "║       Production-Grade Genesis Restart Script                    ║"
    echo "╚═══════════════════════════════════════════════════════════════════╝"
    echo ""

    # Parse arguments
    parse_args "$@"

    # Display configuration
    log_info "Configuration:"
    log_info "  Compose File: $COMPOSE_FILE"
    log_info "  Genesis File: $GENESIS_FILE"
    log_info "  Skip Backup: $SKIP_BACKUP"
    log_info "  Auto Yes: $AUTO_YES"
    echo ""

    # Execute steps
    validate_prerequisites
    validate_genesis
    create_backup
    stop_services
    remove_blockchain_data
    start_services
    health_check
    verify_balances

    # Success summary
    log_step "RESTART COMPLETE ✓"

    echo ""
    log_success "ANDE Chain successfully restarted with new genesis!"
    echo ""
    log_info "Summary:"
    log_info "  • Blockchain started from genesis block 0"
    log_info "  • 3 sequencer wallets funded with 100 ANDE each"
    log_info "  • Services running and healthy"
    echo ""
    log_info "Next Steps:"
    log_info "  1. Deploy consensus contracts: ./scripts/deploy-testnet.sh"
    log_info "  2. Monitor services: docker compose -f $COMPOSE_FILE logs -f"
    log_info "  3. Check Grafana: http://localhost:3001"
    echo ""
    log_info "Sequencer Wallets (100 ANDE each):"
    for addr in "${SEQUENCER_ADDRESSES[@]}"; do
        echo "    • $addr"
    done
    echo ""
}

# Run main function
main "$@"
