#!/bin/bash
# ANDE Chain - Automatic Backup Script
# Backs up blockchain data, database, and configurations

set -e

# Configuration
BACKUP_DIR="${BACKUP_DIR:-/backup/ande-chain}"
DATE=$(date +%Y%m%d_%H%M%S)
RETENTION_DAYS=7

# Colors for output
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

echo -e "${GREEN}========================================${NC}"
echo -e "${GREEN}ANDE Chain Backup - $DATE${NC}"
echo -e "${GREEN}========================================${NC}"

# Create backup directory
mkdir -p "$BACKUP_DIR"

# 1. Backup ANDE Node Data
echo -e "${YELLOW}Backing up ANDE Node data...${NC}"
docker run --rm \
  -v ande-node-data:/data \
  -v "$BACKUP_DIR":/backup \
  alpine tar czf /backup/ande-node-$DATE.tar.gz /data

# 2. Backup PostgreSQL
echo -e "${YELLOW}Backing up PostgreSQL database...${NC}"
docker exec blockscout-db pg_dump -U blockscout blockscout \
  | gzip > "$BACKUP_DIR/postgres-$DATE.sql.gz"

# 3. Backup Configurations
echo -e "${YELLOW}Backing up configurations...${NC}"
tar czf "$BACKUP_DIR/configs-$DATE.tar.gz" \
  -C "$(dirname "$(pwd)")" \
  "$(basename "$(pwd)")/.env" \
  "$(basename "$(pwd)")/docker-compose.yml" \
  "$(basename "$(pwd)")/specs/genesis.json" \
  2>/dev/null || echo "Some config files not found, continuing..."

# 4. Cleanup old backups
echo -e "${YELLOW}Cleaning up backups older than $RETENTION_DAYS days...${NC}"
find "$BACKUP_DIR" -type f -mtime +$RETENTION_DAYS -delete

# 5. Backup summary
echo -e "${GREEN}========================================${NC}"
echo -e "${GREEN}Backup completed successfully!${NC}"
echo -e "${GREEN}========================================${NC}"
echo "Location: $BACKUP_DIR"
echo "Files created:"
ls -lh "$BACKUP_DIR"/*$DATE* 2>/dev/null || echo "No files found"

# Calculate total backup size
TOTAL_SIZE=$(du -sh "$BACKUP_DIR" | cut -f1)
echo -e "\nTotal backup size: $TOTAL_SIZE"

# Exit
echo -e "${GREEN}Done!${NC}"
