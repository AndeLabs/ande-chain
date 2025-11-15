#!/bin/bash
# ANDE Chain - Ubuntu Server Deployment Script
# For server: sator@192.168.0.8

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

echo -e "${GREEN}========================================${NC}"
echo -e "${GREEN}ANDE Chain - Ubuntu Server Deployment${NC}"
echo -e "${GREEN}========================================${NC}"

# Check if running on Linux
if [[ "$OSTYPE" != "linux-gnu"* ]]; then
    echo -e "${YELLOW}Warning: This script is designed for Linux Ubuntu Server${NC}"
    read -p "Continue anyway? (y/n): " -n 1 -r
    echo
    if [[ ! $REPLY =~ ^[Yy]$ ]]; then
        exit 1
    fi
fi

# Function to check if command exists
command_exists() {
    command -v "$1" >/dev/null 2>&1
}

# 1. Install Docker if not present
echo -e "${YELLOW}Checking Docker installation...${NC}"
if ! command_exists docker; then
    echo -e "${YELLOW}Installing Docker...${NC}"
    curl -fsSL https://get.docker.com -o get-docker.sh
    sudo sh get-docker.sh
    sudo usermod -aG docker $USER
    rm get-docker.sh
    echo -e "${GREEN}Docker installed successfully${NC}"
    echo -e "${YELLOW}Please logout and login again for group changes to take effect${NC}"
else
    echo -e "${GREEN}✓ Docker is installed${NC}"
fi

# 2. Install Docker Compose if not present
if ! command_exists docker-compose && ! docker compose version >/dev/null 2>&1; then
    echo -e "${YELLOW}Installing Docker Compose...${NC}"
    sudo curl -L "https://github.com/docker/compose/releases/latest/download/docker-compose-$(uname -s)-$(uname -m)" -o /usr/local/bin/docker-compose
    sudo chmod +x /usr/local/bin/docker-compose
    echo -e "${GREEN}Docker Compose installed${NC}"
else
    echo -e "${GREEN}✓ Docker Compose is installed${NC}"
fi

# 3. Install essential tools
echo -e "${YELLOW}Installing essential tools...${NC}"
sudo apt-get update
sudo apt-get install -y git curl wget jq net-tools htop

# 4. Configure firewall (UFW)
echo -e "${YELLOW}Configuring firewall...${NC}"
if command_exists ufw; then
    sudo ufw allow 22/tcp     # SSH
    sudo ufw allow 8545/tcp   # RPC HTTP
    sudo ufw allow 8546/tcp   # RPC WebSocket
    sudo ufw allow 30303/tcp  # P2P TCP
    sudo ufw allow 30303/udp  # P2P UDP
    sudo ufw allow 9001/tcp   # Metrics
    echo -e "${GREEN}✓ Firewall configured${NC}"
else
    echo -e "${YELLOW}UFW not installed, skipping firewall configuration${NC}"
fi

# 5. Create necessary directories
echo -e "${YELLOW}Creating directories...${NC}"
mkdir -p ~/ande-chain-data
mkdir -p ~/ande-chain-logs

# 6. Clone or update repository
if [ ! -d "ande-chain" ]; then
    echo -e "${YELLOW}Cloning ANDE Chain repository...${NC}"
    git clone https://github.com/ande-labs/ande-chain.git
    cd ande-chain
else
    echo -e "${YELLOW}Updating ANDE Chain repository...${NC}"
    cd ande-chain
    git pull
fi

# 7. Create .env file if not exists
if [ ! -f .env ]; then
    echo -e "${YELLOW}Creating .env file...${NC}"
    cp .env.example .env
    echo -e "${RED}IMPORTANT: Edit .env file to configure your settings!${NC}"
fi

# 8. Stop existing containers (if any)
echo -e "${YELLOW}Stopping existing containers...${NC}"
docker compose down 2>/dev/null || true

# 9. Pull latest images
echo -e "${YELLOW}Pulling Docker images...${NC}"
docker compose pull

# 10. Start ANDE Chain
echo -e "${GREEN}Starting ANDE Chain...${NC}"
docker compose up -d

# 11. Wait for services to start
echo -e "${YELLOW}Waiting for services to start...${NC}"
sleep 10

# 12. Check status
echo -e "${GREEN}Checking service status...${NC}"
docker compose ps

# 13. Test RPC endpoint
echo -e "${YELLOW}Testing RPC endpoint...${NC}"
if curl -s -X POST -H "Content-Type: application/json" \
    --data '{"jsonrpc":"2.0","method":"eth_chainId","params":[],"id":1}' \
    http://localhost:8545 | grep -q "0x181e"; then
    echo -e "${GREEN}✓ RPC endpoint is working!${NC}"
    echo -e "${GREEN}✓ Chain ID: 6174 (0x181e)${NC}"
else
    echo -e "${RED}✗ RPC endpoint test failed${NC}"
fi

# 14. Create systemd service for auto-start
echo -e "${YELLOW}Creating systemd service...${NC}"
sudo tee /etc/systemd/system/ande-chain.service > /dev/null <<EOF
[Unit]
Description=ANDE Chain - Sovereign Rollup
After=docker.service
Requires=docker.service

[Service]
Type=oneshot
RemainAfterExit=true
User=$USER
WorkingDirectory=$HOME/ande-chain
ExecStart=/usr/bin/docker compose up -d
ExecStop=/usr/bin/docker compose down
ExecReload=/usr/bin/docker compose restart

[Install]
WantedBy=multi-user.target
EOF

sudo systemctl daemon-reload
sudo systemctl enable ande-chain.service
echo -e "${GREEN}✓ Systemd service created and enabled${NC}"

# 15. Display information
echo -e "${GREEN}========================================${NC}"
echo -e "${GREEN}ANDE Chain Deployment Complete!${NC}"
echo -e "${GREEN}========================================${NC}"
echo
echo -e "${YELLOW}Access Points:${NC}"
echo "  RPC HTTP:    http://$(hostname -I | awk '{print $1}'):8545"
echo "  WebSocket:   ws://$(hostname -I | awk '{print $1}'):8546"
echo "  Metrics:     http://$(hostname -I | awk '{print $1}'):9001/metrics"
echo
echo -e "${YELLOW}Useful Commands:${NC}"
echo "  View logs:           docker compose logs -f ande-node"
echo "  Check status:        docker compose ps"
echo "  Stop services:       docker compose down"
echo "  Start services:      docker compose up -d"
echo "  Restart services:    docker compose restart"
echo "  System service:      sudo systemctl status ande-chain"
echo
echo -e "${GREEN}Chain is ready for production use!${NC}"