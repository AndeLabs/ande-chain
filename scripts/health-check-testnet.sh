#!/bin/bash
# ANDE Chain Testnet Health Check Script
# Comprehensive health monitoring for all testnet services

set -e
set -u

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Configuration
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"

# Counters
TOTAL_CHECKS=0
PASSED_CHECKS=0
FAILED_CHECKS=0

# Logging functions
log_info() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

log_success() {
    echo -e "${GREEN}[✓]${NC} $1"
    ((PASSED_CHECKS++))
    ((TOTAL_CHECKS++))
}

log_warning() {
    echo -e "${YELLOW}[⚠]${NC} $1"
    ((TOTAL_CHECKS++))
}

log_error() {
    echo -e "${RED}[✗]${NC} $1"
    ((FAILED_CHECKS++))
    ((TOTAL_CHECKS++))
}

# Check Docker service
check_docker() {
    log_info "Checking Docker service..."
    if docker info >/dev/null 2>&1; then
        log_success "Docker is running"
    else
        log_error "Docker is not running"
    fi
}

# Check container status
check_container() {
    local CONTAINER_NAME=$1
    log_info "Checking container: ${CONTAINER_NAME}..."

    if docker ps --format '{{.Names}}' | grep -q "^${CONTAINER_NAME}$"; then
        STATUS=$(docker inspect --format='{{.State.Status}}' "${CONTAINER_NAME}")
        if [ "$STATUS" == "running" ]; then
            log_success "${CONTAINER_NAME} is running"
        else
            log_error "${CONTAINER_NAME} is not running (status: ${STATUS})"
        fi
    else
        log_error "${CONTAINER_NAME} not found"
    fi
}

# Check RPC endpoint
check_rpc() {
    local PORT=$1
    local NAME=$2
    log_info "Checking RPC endpoint: ${NAME} (port ${PORT})..."

    RESPONSE=$(curl -s -X POST \
        -H "Content-Type: application/json" \
        --data '{"jsonrpc":"2.0","method":"eth_blockNumber","params":[],"id":1}' \
        "http://localhost:${PORT}" || echo "")

    if echo "$RESPONSE" | grep -q "result"; then
        BLOCK_NUMBER=$(echo "$RESPONSE" | jq -r '.result' | xargs printf "%d")
        log_success "${NAME} RPC is responding (block: ${BLOCK_NUMBER})"
    else
        log_error "${NAME} RPC is not responding"
    fi
}

# Check WebSocket endpoint
check_ws() {
    local PORT=$1
    local NAME=$2
    log_info "Checking WebSocket endpoint: ${NAME} (port ${PORT})..."

    # Simple check using curl with upgrade header
    if curl -s -o /dev/null -w "%{http_code}" \
        -H "Connection: Upgrade" \
        -H "Upgrade: websocket" \
        "http://localhost:${PORT}" | grep -q "101\|400\|426"; then
        log_success "${NAME} WebSocket is available"
    else
        log_error "${NAME} WebSocket is not available"
    fi
}

# Check Prometheus metrics
check_prometheus() {
    log_info "Checking Prometheus metrics..."

    RESPONSE=$(curl -s "http://localhost:9093/-/healthy" || echo "")
    if echo "$RESPONSE" | grep -q "Prometheus"; then
        log_success "Prometheus is healthy"
    else
        log_error "Prometheus is not healthy"
    fi
}

# Check Grafana
check_grafana() {
    log_info "Checking Grafana API..."

    RESPONSE=$(curl -s "http://localhost:3001/api/health" || echo "")
    if echo "$RESPONSE" | grep -q "ok"; then
        log_success "Grafana is healthy"
    else
        log_error "Grafana is not healthy"
    fi
}

# Check HAProxy stats
check_haproxy() {
    log_info "Checking HAProxy stats..."

    RESPONSE=$(curl -s "http://localhost:8404/stats" || echo "")
    if echo "$RESPONSE" | grep -q "HAProxy"; then
        log_success "HAProxy is healthy"
    else
        log_error "HAProxy is not healthy"
    fi
}

# Check Celestia light node
check_celestia() {
    log_info "Checking Celestia light node..."

    RESPONSE=$(curl -s "http://localhost:26658/health" || echo "")
    if [ ! -z "$RESPONSE" ]; then
        log_success "Celestia light node is responding"
    else
        log_error "Celestia light node is not responding"
    fi
}

# Check disk space
check_disk_space() {
    log_info "Checking disk space..."

    DISK_USAGE=$(df -h . | awk 'NR==2 {print $5}' | sed 's/%//')
    if [ "$DISK_USAGE" -lt 80 ]; then
        log_success "Disk space is adequate (${DISK_USAGE}% used)"
    elif [ "$DISK_USAGE" -lt 90 ]; then
        log_warning "Disk space is getting low (${DISK_USAGE}% used)"
    else
        log_error "Disk space is critically low (${DISK_USAGE}% used)"
    fi
}

# Check memory usage
check_memory() {
    log_info "Checking memory usage..."

    if command -v free >/dev/null 2>&1; then
        MEM_USAGE=$(free | awk 'NR==2 {printf "%.0f", $3*100/$2}')
        if [ "$MEM_USAGE" -lt 80 ]; then
            log_success "Memory usage is normal (${MEM_USAGE}%)"
        elif [ "$MEM_USAGE" -lt 90 ]; then
            log_warning "Memory usage is high (${MEM_USAGE}%)"
        else
            log_error "Memory usage is critically high (${MEM_USAGE}%)"
        fi
    else
        log_warning "Cannot check memory (free command not available)"
    fi
}

# Check Docker volumes
check_volumes() {
    log_info "Checking Docker volumes..."

    VOLUMES=(
        "ande-chain_sequencer-1-data"
        "ande-chain_sequencer-2-data"
        "ande-chain_sequencer-3-data"
        "ande-chain_prometheus-data"
        "ande-chain_grafana-data"
        "ande-chain_celestia-data"
    )

    for VOLUME in "${VOLUMES[@]}"; do
        if docker volume ls --format '{{.Name}}' | grep -q "^${VOLUME}$"; then
            log_success "Volume ${VOLUME} exists"
        else
            log_error "Volume ${VOLUME} not found"
        fi
    done
}

# Check chain synchronization
check_sync() {
    log_info "Checking chain synchronization..."

    # Get block numbers from all sequencers
    BLOCK_1=$(curl -s -X POST -H "Content-Type: application/json" \
        --data '{"jsonrpc":"2.0","method":"eth_blockNumber","params":[],"id":1}' \
        http://localhost:8545 | jq -r '.result' | xargs printf "%d" 2>/dev/null || echo "0")

    BLOCK_2=$(curl -s -X POST -H "Content-Type: application/json" \
        --data '{"jsonrpc":"2.0","method":"eth_blockNumber","params":[],"id":1}' \
        http://localhost:8547 | jq -r '.result' | xargs printf "%d" 2>/dev/null || echo "0")

    BLOCK_3=$(curl -s -X POST -H "Content-Type: application/json" \
        --data '{"jsonrpc":"2.0","method":"eth_blockNumber","params":[],"id":1}' \
        http://localhost:8549 | jq -r '.result' | xargs printf "%d" 2>/dev/null || echo "0")

    # Check if all sequencers are within 5 blocks of each other
    MAX_BLOCK=$(echo -e "$BLOCK_1\n$BLOCK_2\n$BLOCK_3" | sort -nr | head -1)
    MIN_BLOCK=$(echo -e "$BLOCK_1\n$BLOCK_2\n$BLOCK_3" | sort -n | head -1)
    DIFF=$((MAX_BLOCK - MIN_BLOCK))

    if [ "$DIFF" -le 5 ]; then
        log_success "Sequencers are in sync (diff: ${DIFF} blocks)"
    else
        log_warning "Sequencers may be out of sync (diff: ${DIFF} blocks)"
    fi
}

# Display summary
display_summary() {
    echo ""
    echo "========================================="
    echo "  ANDE Chain Testnet Health Check"
    echo "========================================="
    echo ""
    echo "Total Checks: ${TOTAL_CHECKS}"
    echo -e "${GREEN}Passed: ${PASSED_CHECKS}${NC}"
    echo -e "${RED}Failed: ${FAILED_CHECKS}${NC}"
    echo ""

    if [ "$FAILED_CHECKS" -eq 0 ]; then
        echo -e "${GREEN}✓ All checks passed!${NC}"
        echo ""
        return 0
    else
        echo -e "${RED}✗ Some checks failed. Please investigate.${NC}"
        echo ""
        return 1
    fi
}

# Main health check flow
main() {
    echo ""
    log_info "Starting ANDE Chain Testnet Health Check..."
    echo ""

    # Infrastructure checks
    check_docker
    check_disk_space
    check_memory
    check_volumes

    echo ""
    log_info "Checking containers..."
    echo ""

    # Container checks
    check_container "ande-sequencer-1"
    check_container "ande-sequencer-2"
    check_container "ande-sequencer-3"
    check_container "ande-prometheus"
    check_container "ande-grafana"
    check_container "celestia-light"
    check_container "ande-haproxy"

    echo ""
    log_info "Checking endpoints..."
    echo ""

    # RPC endpoint checks
    check_rpc 8545 "Sequencer-1"
    check_rpc 8547 "Sequencer-2"
    check_rpc 8549 "Sequencer-3"

    # WebSocket checks
    check_ws 8546 "Sequencer-1"
    check_ws 8548 "Sequencer-2"
    check_ws 8550 "Sequencer-3"

    echo ""
    log_info "Checking services..."
    echo ""

    # Service checks
    check_prometheus
    check_grafana
    check_haproxy
    check_celestia

    echo ""
    log_info "Checking synchronization..."
    echo ""

    # Sync checks
    check_sync

    # Display summary
    display_summary
}

# Run main function
main
