#!/bin/bash
# ANDE Chain - Quick Start Script
# Starts the complete stack with all services

set -e

echo "🚀 ANDE Chain - Starting Production Stack"
echo "=========================================="

# Check if .env exists
if [ ! -f .env ]; then
    echo "📝 Creating .env from template..."
    cp .env.example .env
    echo "⚠️  WARNING: Please edit .env and configure your environment!"
    echo "   Especially: FAUCET_PRIVATE_KEY and other secrets"
    read -p "Press Enter to continue or Ctrl+C to exit and configure..."
fi

# Check Docker
if ! command -v docker &> /dev/null; then
    echo "❌ Docker is not installed!"
    exit 1
fi

if ! command -v docker compose &> /dev/null; then
    echo "❌ Docker Compose is not installed!"
    exit 1
fi

echo ""
echo "📦 Building ANDE Node image..."
docker compose build ande-node

echo ""
echo "🔧 Starting services..."
docker compose up -d

echo ""
echo "⏳ Waiting for services to be ready..."
sleep 10

echo ""
echo "📊 Service Status:"
docker compose ps

echo ""
echo "✅ ANDE Chain is starting!"
echo ""
echo "📍 Access Points:"
echo "   • RPC Endpoint:      http://localhost:8545"
echo "   • WebSocket:         ws://localhost:8546"
echo "   • Block Explorer:    http://localhost:4000"
echo "   • Faucet:            http://localhost:8081"
echo "   • Grafana:           http://localhost:3000 (admin/andechain2024)"
echo "   • Prometheus:        http://localhost:9090"
echo ""
echo "📝 View logs:"
echo "   docker compose logs -f ande-node"
echo "   docker compose logs -f evolve"
echo ""
echo "🛑 Stop:"
echo "   docker compose down"
echo ""
echo "💡 Tip: Wait 2-3 minutes for full initialization"
