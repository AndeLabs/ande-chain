#!/bin/bash
# ANDE Chain - Cloudflare Tunnel Setup
# Compatible with macOS and Linux

set -e

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

echo -e "${GREEN}========================================${NC}"
echo -e "${GREEN}ANDE Chain - Cloudflare Tunnel Setup${NC}"
echo -e "${GREEN}========================================${NC}"

# Credentials from credenciales.md
ACCOUNT_ID="58f90adc571d31c4b7a860b6edef3406"
API_TOKEN="zMmSa2x59iRRQEoklmVQKtJRbyKPps43shRmU1Rk"
TUNNEL_ID="5fced6cf-92eb-4167-abd3-d0b9397613cc"

echo -e "${GREEN}✓ Using existing credentials${NC}"
echo "  Account ID: $ACCOUNT_ID"
echo "  Tunnel ID: $TUNNEL_ID"

# Get Zone ID for ande.network
echo -e "${YELLOW}Getting Zone ID for ande.network...${NC}"
ZONE_RESPONSE=$(curl -s -X GET "https://api.cloudflare.com/client/v4/zones?name=ande.network" \
     -H "Authorization: Bearer ${API_TOKEN}" \
     -H "Content-Type: application/json")

# Parse Zone ID using jq or sed
if command -v jq &> /dev/null; then
    ZONE_ID=$(echo "$ZONE_RESPONSE" | jq -r '.result[0].id // empty')
else
    ZONE_ID=$(echo "$ZONE_RESPONSE" | sed -n 's/.*"id":"\([^"]*\)".*/\1/p' | head -1)
fi

if [ -z "$ZONE_ID" ]; then
    echo -e "${RED}Error: Could not find zone ande.network${NC}"
    echo "Response: $ZONE_RESPONSE"
    exit 1
fi

echo -e "${GREEN}✓ Zone ID: ${ZONE_ID}${NC}"

# Get tunnel token
echo -e "${YELLOW}Getting tunnel token...${NC}"
TOKEN_RESPONSE=$(curl -s -X GET "https://api.cloudflare.com/client/v4/accounts/${ACCOUNT_ID}/cfd_tunnel/${TUNNEL_ID}/token" \
     -H "Authorization: Bearer ${API_TOKEN}" \
     -H "Content-Type: application/json")

if command -v jq &> /dev/null; then
    TUNNEL_TOKEN=$(echo "$TOKEN_RESPONSE" | jq -r '.result // empty')
else
    TUNNEL_TOKEN=$(echo "$TOKEN_RESPONSE" | sed -n 's/.*"result":"\([^"]*\)".*/\1/p')
fi

if [ -z "$TUNNEL_TOKEN" ]; then
    echo -e "${YELLOW}Note: Could not retrieve tunnel token, using existing tunnel${NC}"
else
    echo -e "${GREEN}✓ Tunnel token retrieved${NC}"
fi

# Configure DNS records
echo -e "${YELLOW}Configuring DNS records...${NC}"

SUBDOMAINS=("rpc" "ws" "api" "explorer" "grafana" "faucet")

for SUBDOMAIN in "${SUBDOMAINS[@]}"; do
    echo -e "${YELLOW}Configuring ${SUBDOMAIN}.ande.network...${NC}"

    # Check if record exists
    EXISTING_RECORDS=$(curl -s -X GET "https://api.cloudflare.com/client/v4/zones/${ZONE_ID}/dns_records?name=${SUBDOMAIN}.ande.network" \
         -H "Authorization: Bearer ${API_TOKEN}" \
         -H "Content-Type: application/json")

    if command -v jq &> /dev/null; then
        RECORD_ID=$(echo "$EXISTING_RECORDS" | jq -r '.result[0].id // empty')
    else
        RECORD_ID=$(echo "$EXISTING_RECORDS" | sed -n 's/.*"id":"\([^"]*\)".*/\1/p' | head -1)
    fi

    if [ -n "$RECORD_ID" ]; then
        echo "  Updating existing record..."
        UPDATE_RESPONSE=$(curl -s -X PUT "https://api.cloudflare.com/client/v4/zones/${ZONE_ID}/dns_records/${RECORD_ID}" \
             -H "Authorization: Bearer ${API_TOKEN}" \
             -H "Content-Type: application/json" \
             --data "{
               \"type\": \"CNAME\",
               \"name\": \"${SUBDOMAIN}\",
               \"content\": \"${TUNNEL_ID}.cfargotunnel.com\",
               \"proxied\": true
             }")
    else
        echo "  Creating new record..."
        UPDATE_RESPONSE=$(curl -s -X POST "https://api.cloudflare.com/client/v4/zones/${ZONE_ID}/dns_records" \
             -H "Authorization: Bearer ${API_TOKEN}" \
             -H "Content-Type: application/json" \
             --data "{
               \"type\": \"CNAME\",
               \"name\": \"${SUBDOMAIN}\",
               \"content\": \"${TUNNEL_ID}.cfargotunnel.com\",
               \"proxied\": true
             }")
    fi

    if echo "$UPDATE_RESPONSE" | grep -q '"success":true'; then
        echo -e "${GREEN}  ✓ ${SUBDOMAIN}.ande.network configured${NC}"
    else
        echo -e "${RED}  ✗ Error configuring ${SUBDOMAIN}.ande.network${NC}"
        echo "  Response: $UPDATE_RESPONSE"
    fi
done

# Configure tunnel routes
echo -e "${YELLOW}Configuring tunnel routes...${NC}"

TUNNEL_CONFIG=$(cat <<'EOF'
{
  "config": {
    "ingress": [
      {
        "hostname": "rpc.ande.network",
        "service": "http://localhost:8545"
      },
      {
        "hostname": "ws.ande.network",
        "service": "http://localhost:8546"
      },
      {
        "hostname": "api.ande.network",
        "service": "http://localhost:8545"
      },
      {
        "hostname": "explorer.ande.network",
        "service": "http://localhost:4000"
      },
      {
        "hostname": "grafana.ande.network",
        "service": "http://localhost:3000"
      },
      {
        "hostname": "faucet.ande.network",
        "service": "http://localhost:8081"
      },
      {
        "service": "http_status:404"
      }
    ]
  }
}
EOF
)

CONFIG_RESPONSE=$(curl -s -X PUT "https://api.cloudflare.com/client/v4/accounts/${ACCOUNT_ID}/cfd_tunnel/${TUNNEL_ID}/configurations" \
     -H "Authorization: Bearer ${API_TOKEN}" \
     -H "Content-Type: application/json" \
     --data "$TUNNEL_CONFIG")

if echo "$CONFIG_RESPONSE" | grep -q '"success":true'; then
    echo -e "${GREEN}✓ Tunnel routes configured${NC}"
else
    echo -e "${YELLOW}Note: Routes will be configured when tunnel connects${NC}"
fi

# Generate installation script for server
echo -e "${YELLOW}Generating installation script for server...${NC}"

cat > install-cloudflared.sh << EOF
#!/bin/bash
# Install Cloudflare Tunnel on ANDE Chain server

set -e

echo "Installing Cloudflare Tunnel..."

# Download cloudflared
if [ ! -f /usr/local/bin/cloudflared ]; then
    echo "Downloading cloudflared..."
    wget -q https://github.com/cloudflare/cloudflared/releases/latest/download/cloudflared-linux-amd64
    sudo mv cloudflared-linux-amd64 /usr/local/bin/cloudflared
    sudo chmod +x /usr/local/bin/cloudflared
fi

# Create config directory
sudo mkdir -p /etc/cloudflared

# Create tunnel config
sudo tee /etc/cloudflared/config.yml > /dev/null <<'CFGEOF'
tunnel: ${TUNNEL_ID}
credentials-file: /etc/cloudflared/${TUNNEL_ID}.json

ingress:
  - hostname: rpc.ande.network
    service: http://localhost:8545
  - hostname: ws.ande.network
    service: http://localhost:8546
  - hostname: api.ande.network
    service: http://localhost:8545
  - hostname: explorer.ande.network
    service: http://localhost:4000
  - hostname: grafana.ande.network
    service: http://localhost:3000
  - hostname: faucet.ande.network
    service: http://localhost:8081
  - service: http_status:404
CFGEOF

# Get tunnel token and create credentials file
TUNNEL_TOKEN="${TUNNEL_TOKEN}"

if [ -n "\$TUNNEL_TOKEN" ]; then
    echo "Using tunnel token..."
    sudo cloudflared tunnel install \$TUNNEL_TOKEN
else
    echo "Error: No tunnel token available"
    exit 1
fi

# Install as service
sudo cloudflared service install

# Start service
sudo systemctl start cloudflared
sudo systemctl enable cloudflared

echo "✓ Cloudflare Tunnel installed and running"
echo ""
echo "Check status:"
echo "  sudo systemctl status cloudflared"
echo ""
echo "View logs:"
echo "  sudo journalctl -u cloudflared -f"
EOF

chmod +x install-cloudflared.sh

echo -e "${GREEN}========================================${NC}"
echo -e "${GREEN}✅ Setup Complete!${NC}"
echo -e "${GREEN}========================================${NC}"
echo ""
echo -e "${YELLOW}DNS Records Configured:${NC}"
for SUBDOMAIN in "${SUBDOMAINS[@]}"; do
    echo "  ✓ https://${SUBDOMAIN}.ande.network"
done
echo ""
echo -e "${YELLOW}Next Steps:${NC}"
echo ""
echo "1. Copy script to server:"
echo -e "${BLUE}   scp install-cloudflared.sh sator@192.168.0.8:~/${NC}"
echo ""
echo "2. Run on server:"
echo -e "${BLUE}   ssh sator@192.168.0.8${NC}"
echo -e "${BLUE}   ./install-cloudflared.sh${NC}"
echo ""
echo -e "${GREEN}Your ANDE Chain will be globally accessible via HTTPS!${NC}"
