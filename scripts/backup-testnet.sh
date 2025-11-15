#!/bin/bash
# ANDE Chain Testnet Backup Script
# Automated backup for sequencer data and volumes

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
BACKUP_DIR="${PROJECT_ROOT}/backups"
DATE=$(date +%Y%m%d_%H%M%S)

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

# Create backup directory
create_backup_dir() {
    log_info "Creating backup directory..."
    mkdir -p "${BACKUP_DIR}"
    log_success "Backup directory created: ${BACKUP_DIR}"
}

# Backup sequencer data
backup_sequencer() {
    local SEQUENCER_ID=$1
    log_info "Backing up sequencer-${SEQUENCER_ID} data..."

    docker run --rm \
        -v ande-chain_sequencer-${SEQUENCER_ID}-data:/data \
        -v "${BACKUP_DIR}:/backup" \
        alpine:3.22.0 \
        tar czf "/backup/sequencer-${SEQUENCER_ID}-data_${DATE}.tar.gz" /data

    log_success "Sequencer-${SEQUENCER_ID} data backed up"
}

# Backup sequencer logs
backup_logs() {
    local SEQUENCER_ID=$1
    log_info "Backing up sequencer-${SEQUENCER_ID} logs..."

    docker run --rm \
        -v ande-chain_sequencer-${SEQUENCER_ID}-logs:/logs \
        -v "${BACKUP_DIR}:/backup" \
        alpine:3.22.0 \
        tar czf "/backup/sequencer-${SEQUENCER_ID}-logs_${DATE}.tar.gz" /logs

    log_success "Sequencer-${SEQUENCER_ID} logs backed up"
}

# Backup Prometheus data
backup_prometheus() {
    log_info "Backing up Prometheus data..."

    docker run --rm \
        -v ande-chain_prometheus-data:/prometheus \
        -v "${BACKUP_DIR}:/backup" \
        alpine:3.22.0 \
        tar czf "/backup/prometheus-data_${DATE}.tar.gz" /prometheus

    log_success "Prometheus data backed up"
}

# Backup Grafana data
backup_grafana() {
    log_info "Backing up Grafana data..."

    docker run --rm \
        -v ande-chain_grafana-data:/var/lib/grafana \
        -v "${BACKUP_DIR}:/backup" \
        alpine:3.22.0 \
        tar czf "/backup/grafana-data_${DATE}.tar.gz" /var/lib/grafana

    log_success "Grafana data backed up"
}

# Backup Celestia data
backup_celestia() {
    log_info "Backing up Celestia light node data..."

    docker run --rm \
        -v ande-chain_celestia-data:/celestia \
        -v "${BACKUP_DIR}:/backup" \
        alpine:3.22.0 \
        tar czf "/backup/celestia-data_${DATE}.tar.gz" /celestia

    log_success "Celestia data backed up"
}

# Backup configuration files
backup_config() {
    log_info "Backing up configuration files..."

    cd "${PROJECT_ROOT}"
    tar czf "${BACKUP_DIR}/config_${DATE}.tar.gz" \
        .env.testnet \
        docker-compose-testnet.yml \
        monitoring/ \
        2>/dev/null || log_warning "Some config files may not exist"

    log_success "Configuration files backed up"
}

# Clean old backups (keep last 30 days)
cleanup_old_backups() {
    log_info "Cleaning up old backups (keeping last 30 days)..."

    find "${BACKUP_DIR}" -type f -name "*.tar.gz" -mtime +30 -delete

    log_success "Old backups cleaned up"
}

# Calculate backup size
calculate_backup_size() {
    log_info "Calculating backup size..."

    TOTAL_SIZE=$(du -sh "${BACKUP_DIR}" | cut -f1)
    log_success "Total backup size: ${TOTAL_SIZE}"
}

# Main backup flow
main() {
    echo ""
    log_info "Starting ANDE Chain Testnet Backup..."
    echo ""

    create_backup_dir

    # Backup all sequencers
    for i in 1 2 3; do
        backup_sequencer $i
        backup_logs $i
    done

    # Backup monitoring
    backup_prometheus
    backup_grafana

    # Backup Celestia
    backup_celestia

    # Backup configuration
    backup_config

    # Cleanup old backups
    cleanup_old_backups

    # Show backup size
    calculate_backup_size

    echo ""
    log_success "Backup complete! Backup location: ${BACKUP_DIR}"
    log_info "Backup date: ${DATE}"
    echo ""
}

# Run main function
main
