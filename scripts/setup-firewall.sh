#!/bin/bash
# ANDE Chain - Firewall Setup Script
# Configures UFW firewall for production security

set -e

# Colors
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m'

echo -e "${GREEN}========================================${NC}"
echo -e "${GREEN}ANDE Chain Firewall Setup${NC}"
echo -e "${GREEN}========================================${NC}"

# Check if running as root
if [ "$EUID" -ne 0 ]; then
    echo -e "${RED}Please run as root (sudo)${NC}"
    exit 1
fi

# Install UFW if not present
if ! command -v ufw &> /dev/null; then
    echo -e "${YELLOW}Installing UFW...${NC}"
    apt-get update
    apt-get install -y ufw
fi

# Reset UFW to default
echo -e "${YELLOW}Resetting UFW to default...${NC}"
ufw --force reset

# Default policies
ufw default deny incoming
ufw default allow outgoing

# Allow SSH (CRITICAL - don't lock yourself out!)
echo -e "${YELLOW}Allowing SSH...${NC}"
ufw allow 22/tcp comment 'SSH'

# Allow Public Services
echo -e "${YELLOW}Allowing public services...${NC}"
ufw allow 8545/tcp comment 'RPC HTTP'
ufw allow 8546/tcp comment 'RPC WebSocket'
ufw allow 4000/tcp comment 'Block Explorer'
ufw allow 8081/tcp comment 'Faucet'

# Allow P2P
ufw allow 30303/tcp comment 'P2P TCP'
ufw allow 30303/udp comment 'P2P UDP'

# DENY direct database access
echo -e "${YELLOW}Blocking direct database access...${NC}"
ufw deny 7432/tcp comment 'PostgreSQL - DENY'
ufw deny 6380/tcp comment 'Redis - DENY'

# DENY Engine API (internal only)
ufw deny 8551/tcp comment 'Engine API - DENY'

# Optional: Allow admin services from specific IP
# Replace YOUR_IP_HERE with your actual IP
# ufw allow from YOUR_IP_HERE to any port 3000 comment 'Grafana - Admin Only'
# ufw allow from YOUR_IP_HERE to any port 9090 comment 'Prometheus - Admin Only'

echo -e "${YELLOW}Current UFW rules:${NC}"
ufw show added

# Enable UFW
echo -e "${YELLOW}Enabling UFW...${NC}"
ufw --force enable

# Show status
echo -e "${GREEN}========================================${NC}"
echo -e "${GREEN}Firewall configured successfully!${NC}"
echo -e "${GREEN}========================================${NC}"
ufw status numbered

echo -e "\n${YELLOW}IMPORTANT:${NC} Make sure SSH (port 22) is allowed before disconnecting!"
echo -e "${YELLOW}To allow admin services from your IP, run:${NC}"
echo -e "sudo ufw allow from YOUR_IP to any port 3000"
echo -e "sudo ufw allow from YOUR_IP to any port 9090"
