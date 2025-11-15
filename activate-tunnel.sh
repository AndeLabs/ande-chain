#!/bin/bash
# ANDE Chain - Cloudflare Tunnel Activation Script

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

echo -e "${GREEN}========================================${NC}"
echo -e "${GREEN}ANDE Chain - Cloudflare Tunnel Activation${NC}"
echo -e "${GREEN}========================================${NC}"

TUNNEL_ID="5fced6cf-92eb-4167-abd3-d0b9397613cc"

# Check if cloudflared is installed
if ! command -v cloudflared &> /dev/null; then
    echo -e "${RED}✗ Cloudflared is not installed${NC}"
    echo -e "${YELLOW}Please run setup-cloudflare-tunnel.sh first${NC}"
    exit 1
fi

echo -e "${GREEN}✓ Cloudflared is installed${NC}"

# Option selection
echo -e "${BLUE}Choose setup method:${NC}"
echo "1) I have a tunnel token from Cloudflare dashboard"
echo "2) Login with Cloudflare account (browser will open)"
echo "3) I have Cloudflare API credentials"
echo "4) Show manual instructions"
read -p "Select option (1-4): " option

case $option in
    1)
        echo -e "${YELLOW}Please enter your tunnel token:${NC}"
        echo "(It should start with 'eyJ...')"
        read -s TUNNEL_TOKEN
        echo

        # Create credentials file
        echo -e "${YELLOW}Creating credentials file...${NC}"
        sudo tee /etc/cloudflared/credentials.json > /dev/null <<EOF
{
  "TunnelToken": "${TUNNEL_TOKEN}"
}
EOF

        echo -e "${GREEN}✓ Credentials saved${NC}"

        # Install service with token
        echo -e "${YELLOW}Installing service...${NC}"
        sudo cloudflared service install --legacy

        # Start service
        echo -e "${YELLOW}Starting tunnel...${NC}"
        sudo systemctl start cloudflared
        sudo systemctl enable cloudflared

        echo -e "${GREEN}✓ Tunnel service started${NC}"
        ;;

    2)
        echo -e "${YELLOW}Opening browser for Cloudflare login...${NC}"
        cloudflared tunnel login

        echo -e "${YELLOW}Creating tunnel routes...${NC}"
        cloudflared tunnel route dns ${TUNNEL_ID} rpc.ande.network
        cloudflared tunnel route dns ${TUNNEL_ID} ws.ande.network
        cloudflared tunnel route dns ${TUNNEL_ID} api.ande.network
        cloudflared tunnel route dns ${TUNNEL_ID} explorer.ande.network
        cloudflared tunnel route dns ${TUNNEL_ID} grafana.ande.network
        cloudflared tunnel route dns ${TUNNEL_ID} metrics.ande.network

        echo -e "${GREEN}✓ Routes created${NC}"

        echo -e "${YELLOW}Starting tunnel...${NC}"
        cloudflared tunnel run --config /etc/cloudflared/config.yml ${TUNNEL_ID} &

        echo -e "${GREEN}✓ Tunnel started${NC}"
        ;;

    3)
        echo -e "${YELLOW}Enter your Cloudflare email:${NC}"
        read CF_EMAIL
        echo -e "${YELLOW}Enter your Cloudflare API Key:${NC}"
        read -s CF_API_KEY
        echo

        # Save credentials
        export CLOUDFLARE_EMAIL="${CF_EMAIL}"
        export CLOUDFLARE_API_KEY="${CF_API_KEY}"

        echo -e "${YELLOW}Configuring tunnel...${NC}"
        cloudflared tunnel create ${TUNNEL_ID} || true
        cloudflared tunnel route dns ${TUNNEL_ID} "*.ande.network"

        echo -e "${GREEN}✓ Tunnel configured${NC}"
        ;;

    4)
        echo -e "${BLUE}Manual Setup Instructions:${NC}"
        echo
        echo "1. Go to: https://dash.cloudflare.com"
        echo "2. Navigate to: Zero Trust > Access > Tunnels"
        echo "3. Find tunnel ID: ${TUNNEL_ID}"
        echo "4. Copy the tunnel token"
        echo "5. Run this script again and select option 1"
        echo
        echo -e "${YELLOW}Or use the Cloudflare CLI:${NC}"
        echo "cloudflared tunnel login"
        echo "cloudflared tunnel run ${TUNNEL_ID}"
        exit 0
        ;;

    *)
        echo -e "${RED}Invalid option${NC}"
        exit 1
        ;;
esac

# Test the tunnel
echo -e "${YELLOW}Testing tunnel connection...${NC}"
sleep 5

# Check service status
if sudo systemctl is-active --quiet cloudflared; then
    echo -e "${GREEN}✓ Cloudflared service is running${NC}"
else
    echo -e "${RED}✗ Cloudflared service is not running${NC}"
    echo -e "${YELLOW}Check logs: sudo journalctl -u cloudflared -n 50${NC}"
    exit 1
fi

# Test endpoints (will work once DNS propagates)
echo -e "${YELLOW}DNS propagation may take a few minutes...${NC}"
echo
echo -e "${GREEN}========================================${NC}"
echo -e "${GREEN}Tunnel Activation Complete!${NC}"
echo -e "${GREEN}========================================${NC}"
echo
echo -e "${YELLOW}Your endpoints will be available at:${NC}"
echo "  RPC:        https://rpc.ande.network"
echo "  WebSocket:  wss://ws.ande.network"
echo "  API:        https://api.ande.network"
echo "  Explorer:   https://explorer.ande.network"
echo "  Grafana:    https://grafana.ande.network"
echo "  Metrics:    https://metrics.ande.network"
echo
echo -e "${YELLOW}Test with:${NC}"
echo "  curl https://rpc.ande.network"
echo
echo -e "${YELLOW}Monitor tunnel:${NC}"
echo "  sudo journalctl -u cloudflared -f"
echo
echo -e "${GREEN}ANDE Chain is now globally accessible!${NC}"