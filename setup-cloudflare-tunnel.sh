#!/bin/bash
# ANDE Chain - Cloudflare Tunnel Setup Script

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

echo -e "${GREEN}========================================${NC}"
echo -e "${GREEN}ANDE Chain - Cloudflare Tunnel Setup${NC}"
echo -e "${GREEN}========================================${NC}"

# Install cloudflared
echo -e "${YELLOW}Installing Cloudflare Tunnel...${NC}"
wget -q https://github.com/cloudflare/cloudflared/releases/latest/download/cloudflared-linux-amd64.deb
sudo dpkg -i cloudflared-linux-amd64.deb
rm cloudflared-linux-amd64.deb

echo -e "${GREEN}✓ Cloudflared installed${NC}"

# Create configuration directory
sudo mkdir -p /etc/cloudflared

# Create tunnel configuration
echo -e "${YELLOW}Creating tunnel configuration...${NC}"
sudo tee /etc/cloudflared/config.yml > /dev/null <<EOF
tunnel: 5fced6cf-92eb-4167-abd3-d0b9397613cc
credentials-file: /etc/cloudflared/credentials.json

ingress:
  # RPC Endpoint
  - hostname: rpc.ande.network
    service: http://localhost:8545
    originRequest:
      noTLSVerify: true
      connectTimeout: 30s

  # WebSocket Endpoint
  - hostname: ws.ande.network
    service: ws://localhost:8546
    originRequest:
      noTLSVerify: true
      connectTimeout: 30s

  # Block Explorer API
  - hostname: api.ande.network
    service: http://localhost:8545
    originRequest:
      noTLSVerify: true
      connectTimeout: 30s

  # Metrics/Prometheus
  - hostname: metrics.ande.network
    service: http://localhost:9001
    originRequest:
      noTLSVerify: true

  # Grafana Dashboard (when available)
  - hostname: grafana.ande.network
    service: http://localhost:3000
    originRequest:
      noTLSVerify: true

  # Block Explorer (when available)
  - hostname: explorer.ande.network
    service: http://localhost:4000
    originRequest:
      noTLSVerify: true

  # Catch-all rule
  - service: http_status:404
EOF

echo -e "${GREEN}✓ Tunnel configuration created${NC}"

# Note about credentials
echo -e "${RED}IMPORTANT: You need to add the tunnel credentials!${NC}"
echo -e "${YELLOW}1. Get your tunnel credentials from Cloudflare dashboard${NC}"
echo -e "${YELLOW}2. Create /etc/cloudflared/credentials.json with the tunnel token${NC}"
echo -e "${YELLOW}3. Then run: sudo cloudflared service install${NC}"
echo -e "${YELLOW}4. Start the tunnel: sudo systemctl start cloudflared${NC}"

echo -e "${GREEN}========================================${NC}"
echo -e "${GREEN}Configuration complete!${NC}"
echo -e "${GREEN}========================================${NC}"