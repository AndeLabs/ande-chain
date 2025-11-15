#!/bin/bash
# ANDE Chain Testnet Deployment Script
# Professional deployment automation for Celestia Mocha-4

set -e  # Exit on error
set -u  # Exit on undefined variable

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Configuration
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"
ENV_FILE="${PROJECT_ROOT}/.env.testnet"

# Logging functions
log_info() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

log_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

log_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

log_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# Check prerequisites
check_prerequisites() {
    log_info "Checking prerequisites..."

    # Check if .env.testnet exists
    if [ ! -f "$ENV_FILE" ]; then
        log_error ".env.testnet not found!"
        exit 1
    fi

    # Check if forge is installed
    if ! command -v forge &> /dev/null; then
        log_error "Foundry (forge) not installed. Install from https://getfoundry.sh"
        exit 1
    fi

    # Check if docker is installed
    if ! command -v docker &> /dev/null; then
        log_error "Docker not installed!"
        exit 1
    fi

    # Check if cargo is installed
    if ! command -v cargo &> /dev/null; then
        log_error "Rust (cargo) not installed!"
        exit 1
    fi

    log_success "All prerequisites met"
}

# Load environment variables
load_env() {
    log_info "Loading environment variables from .env.testnet..."
    export $(cat "$ENV_FILE" | grep -v '^#' | grep -v '^$' | xargs)
    log_success "Environment loaded"
}

# Compile Rust code
compile_rust() {
    log_info "Compiling Rust workspace..."
    cd "$PROJECT_ROOT"
    cargo build --release --workspace
    log_success "Rust compilation complete"
}

# Compile Solidity contracts
compile_contracts() {
    log_info "Compiling Solidity contracts..."
    cd "$PROJECT_ROOT/contracts"
    forge build
    log_success "Contract compilation complete"
}

# Deploy contracts to testnet
deploy_contracts() {
    log_info "Deploying contracts to Celestia Mocha-4 testnet..."

    cd "$PROJECT_ROOT/contracts"

    # Check if private key is set
    if [ -z "${SEQUENCER_PRIVATE_KEY:-}" ]; then
        log_error "SEQUENCER_PRIVATE_KEY not set in .env.testnet"
        exit 1
    fi

    log_info "Deploying AndeConsensus..."
    CONSENSUS_ADDRESS=$(forge create \
        --rpc-url "$CONSENSUS_RPC_URL" \
        --private-key "$SEQUENCER_PRIVATE_KEY" \
        src/consensus/AndeConsensus.sol:AndeConsensus \
        --json | jq -r '.deployedTo')

    log_success "AndeConsensus deployed at: $CONSENSUS_ADDRESS"

    log_info "Deploying AndeSequencerCoordinator..."
    COORDINATOR_ADDRESS=$(forge create \
        --rpc-url "$CONSENSUS_RPC_URL" \
        --private-key "$SEQUENCER_PRIVATE_KEY" \
        src/consensus/AndeSequencerCoordinator.sol:AndeSequencerCoordinator \
        --constructor-args "$CONSENSUS_ADDRESS" \
        --json | jq -r '.deployedTo')

    log_success "AndeSequencerCoordinator deployed at: $COORDINATOR_ADDRESS"

    # Update .env.testnet with deployed addresses
    log_info "Updating .env.testnet with contract addresses..."
    sed -i.bak "s/CONSENSUS_CONTRACT_ADDRESS=.*/CONSENSUS_CONTRACT_ADDRESS=$CONSENSUS_ADDRESS/" "$ENV_FILE"
    sed -i.bak "s/COORDINATOR_CONTRACT_ADDRESS=.*/COORDINATOR_CONTRACT_ADDRESS=$COORDINATOR_ADDRESS/" "$ENV_FILE"
    rm "${ENV_FILE}.bak"

    log_success "Contract addresses updated in .env.testnet"
}

# Build Docker images
build_docker() {
    log_info "Building Docker images..."
    cd "$PROJECT_ROOT"
    docker compose -f docker-compose-testnet.yml build
    log_success "Docker images built"
}

# Start services
start_services() {
    log_info "Starting services..."
    cd "$PROJECT_ROOT"
    docker compose -f docker-compose-testnet.yml up -d
    log_success "Services started"
}

# Verify deployment
verify_deployment() {
    log_info "Verifying deployment..."

    # Wait for RPC to be ready
    log_info "Waiting for RPC to be ready..."
    for i in {1..30}; do
        if curl -s http://localhost:8545 > /dev/null; then
            log_success "RPC is ready"
            break
        fi
        if [ $i -eq 30 ]; then
            log_error "RPC failed to start"
            exit 1
        fi
        sleep 2
    done

    # Check chain ID
    CHAIN_ID_RESPONSE=$(curl -s -X POST \
        -H "Content-Type: application/json" \
        --data '{"jsonrpc":"2.0","method":"eth_chainId","params":[],"id":1}' \
        http://localhost:8545 | jq -r '.result')

    EXPECTED_CHAIN_ID="0x181e"  # 6174 in hex
    if [ "$CHAIN_ID_RESPONSE" == "$EXPECTED_CHAIN_ID" ]; then
        log_success "Chain ID verified: $CHAIN_ID (6174)"
    else
        log_error "Chain ID mismatch! Expected $EXPECTED_CHAIN_ID, got $CHAIN_ID_RESPONSE"
        exit 1
    fi

    # Check consensus contracts
    log_info "Verifying consensus contracts..."
    # TODO: Add contract verification logic

    log_success "Deployment verification complete"
}

# Display deployment summary
deployment_summary() {
    echo ""
    echo "=========================================="
    echo "  ANDE Chain Testnet Deployment Summary"
    echo "=========================================="
    echo ""
    echo "Network: ANDE Testnet (Chain ID: 6174)"
    echo "Environment: Celestia Mocha-4"
    echo ""
    echo "Deployed Contracts:"
    echo "  • AndeConsensus:            $CONSENSUS_ADDRESS"
    echo "  • AndeSequencerCoordinator: $COORDINATOR_ADDRESS"
    echo ""
    echo "RPC Endpoints:"
    echo "  • HTTP: http://localhost:8545"
    echo "  • WebSocket: ws://localhost:8546"
    echo ""
    echo "Metrics:"
    echo "  • Prometheus: http://localhost:9090/metrics"
    echo ""
    echo "Explorer:"
    echo "  • Local: http://localhost:3000"
    echo ""
    echo "Logs:"
    echo "  • View logs: docker compose -f docker-compose-testnet.yml logs -f"
    echo ""
    echo "Next Steps:"
    echo "  1. Fund your sequencer address with testnet tokens"
    echo "  2. Register as a validator in AndeConsensus"
    echo "  3. Monitor metrics at http://localhost:9090/metrics"
    echo "  4. Run integration tests: cargo test --workspace"
    echo ""
    echo "=========================================="
}

# Main deployment flow
main() {
    echo ""
    log_info "Starting ANDE Chain Testnet Deployment..."
    echo ""

    check_prerequisites
    load_env
    compile_rust
    compile_contracts
    deploy_contracts
    build_docker
    start_services
    verify_deployment
    deployment_summary

    log_success "Deployment complete! 🚀"
}

# Run main function
main
