#!/bin/bash

# ANDE Chain - Deployment Script
# Deploys latest changes to production server

set -e

# Configuration
SERVER_USER="sator"
SERVER_HOST="192.168.0.8"
SERVER_PASSWORD="1992"
REMOTE_DIR="ande-chain"
BINARY_NAME="ande-reth"

echo "🚀 ANDE Chain Deployment Script"
echo "================================"
echo ""

# Colors
GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m' # No Color

# Step 1: Verify local changes are committed
echo -e "${BLUE}📋 Step 1: Verifying local repository...${NC}"
if [ -n "$(git status --porcelain)" ]; then
    echo -e "${RED}❌ Error: Uncommitted changes detected${NC}"
    echo "Please commit or stash your changes first"
    exit 1
fi
echo -e "${GREEN}✅ Local repository clean${NC}"
echo ""

# Step 2: Push to GitHub
echo -e "${BLUE}📤 Step 2: Pushing to GitHub...${NC}"
CURRENT_BRANCH=$(git branch --show-current)
git push origin $CURRENT_BRANCH
echo -e "${GREEN}✅ Pushed to GitHub${NC}"
echo ""

# Step 3: Pull on server
echo -e "${BLUE}📥 Step 3: Pulling latest changes on server...${NC}"
sshpass -p "$SERVER_PASSWORD" ssh -o StrictHostKeyChecking=no \
    $SERVER_USER@$SERVER_HOST \
    "cd $REMOTE_DIR && git fetch origin && git reset --hard origin/$CURRENT_BRANCH"
echo -e "${GREEN}✅ Latest code pulled on server${NC}"
echo ""

# Step 4: Build on server
echo -e "${BLUE}🔨 Step 4: Building release binary on server...${NC}"
echo -e "${YELLOW}⏳ This may take 20-30 minutes...${NC}"
sshpass -p "$SERVER_PASSWORD" ssh -o StrictHostKeyChecking=no \
    $SERVER_USER@$SERVER_HOST \
    "cd $REMOTE_DIR && cargo build --release 2>&1 | tail -30"
echo -e "${GREEN}✅ Build completed${NC}"
echo ""

# Step 5: Verify binary
echo -e "${BLUE}✅ Step 5: Verifying binary...${NC}"
sshpass -p "$SERVER_PASSWORD" ssh -o StrictHostKeyChecking=no \
    $SERVER_USER@$SERVER_HOST \
    "cd $REMOTE_DIR && ./target/release/$BINARY_NAME --version"
echo -e "${GREEN}✅ Binary verified${NC}"
echo ""

# Step 6: Restart service (if running)
echo -e "${BLUE}🔄 Step 6: Checking if service needs restart...${NC}"
SERVICE_RUNNING=$(sshpass -p "$SERVER_PASSWORD" ssh -o StrictHostKeyChecking=no \
    $SERVER_USER@$SERVER_HOST \
    "pgrep -f $BINARY_NAME || echo 'not_running'")

if [ "$SERVICE_RUNNING" != "not_running" ]; then
    echo -e "${YELLOW}⚠️  Service is running. Restart manually if needed.${NC}"
    echo "  Command: ssh $SERVER_USER@$SERVER_HOST"
    echo "  Then: pkill -f $BINARY_NAME && cd $REMOTE_DIR && nohup ./target/release/$BINARY_NAME node &"
else
    echo -e "${GREEN}✅ No service running (manual start required)${NC}"
fi
echo ""

# Step 7: Show deployment info
echo -e "${GREEN}════════════════════════════════════════${NC}"
echo -e "${GREEN}✅ Deployment Successful!${NC}"
echo -e "${GREEN}════════════════════════════════════════${NC}"
echo ""
echo "Server: $SERVER_HOST"
echo "Binary: $REMOTE_DIR/target/release/$BINARY_NAME"
echo "Status: Ready to start"
echo ""
echo "To start the node:"
echo "  ssh $SERVER_USER@$SERVER_HOST"
echo "  cd $REMOTE_DIR"
echo "  ./target/release/$BINARY_NAME node"
echo ""
echo "To check status:"
echo "  curl -X POST http://$SERVER_HOST:8545 -H \"Content-Type: application/json\" -d '{\"jsonrpc\":\"2.0\",\"method\":\"eth_blockNumber\",\"params\":[],\"id\":1}'"
echo ""
