# ANDE Chain - Production Deployment Guide

## 🚀 Quick Deployment on Ubuntu Server

### Prerequisites
- Ubuntu Server 20.04 LTS or newer
- Minimum 8GB RAM (16GB recommended)
- 100GB+ SSD storage
- Static IP address
- SSH access

### One-Line Installation

```bash
curl -fsSL https://raw.githubusercontent.com/ande-labs/ande-chain/main/deploy.sh | bash
```

### Manual Deployment Steps

#### 1. Connect to your server
```bash
ssh sator@192.168.0.8
```

#### 2. Clone the repository
```bash
git clone https://github.com/ande-labs/ande-chain.git
cd ande-chain
```

#### 3. Run deployment script
```bash
chmod +x deploy.sh
./deploy.sh
```

## 📦 What the deployment script does:

1. **Installs Docker & Docker Compose** if not present
2. **Configures firewall rules** for required ports
3. **Creates necessary directories** for data persistence
4. **Sets up environment variables** from `.env.example`
5. **Starts all services** with Docker Compose
6. **Creates systemd service** for auto-start on boot
7. **Verifies installation** by testing RPC endpoints

## 🔧 Configuration

### Environment Variables
Edit `.env` file after deployment:

```bash
nano .env
```

Key configurations:
- `RUST_LOG`: Logging level (info, debug, trace)
- `ENABLE_PARALLEL_EVM`: Enable/disable parallel execution
- `ENABLE_MEV_DETECTION`: Enable/disable MEV detection
- `FAUCET_PRIVATE_KEY`: Private key for faucet (change in production!)

### Network Ports

| Port | Protocol | Purpose |
|------|----------|---------|
| 8545 | TCP | JSON-RPC HTTP |
| 8546 | TCP | JSON-RPC WebSocket |
| 8551 | TCP | Engine API (JWT auth) |
| 30303 | TCP/UDP | P2P Network |
| 9001 | TCP | Prometheus Metrics |
| 3000 | TCP | Grafana Dashboard |
| 4000 | TCP | Block Explorer |

## 🏗️ Architecture

```
┌─────────────────────────────────────┐
│      Ubuntu Server (192.168.0.8)    │
├─────────────────────────────────────┤
│                                     │
│  Docker Compose Stack:              │
│  ┌─────────────────────────────┐   │
│  │  ANDE Node (Reth-based)     │   │
│  │  - Token Duality Precompile │   │
│  │  - Parallel EVM             │   │
│  │  - MEV Protection           │   │
│  └─────────────────────────────┘   │
│                ↕                    │
│  ┌─────────────────────────────┐   │
│  │  Evolve Sequencer           │   │
│  │  - Block Production         │   │
│  │  - Transaction Ordering     │   │
│  └─────────────────────────────┘   │
│                ↕                    │
│  ┌─────────────────────────────┐   │
│  │  Celestia Light Node        │   │
│  │  - Data Availability        │   │
│  │  - Mocha-4 Testnet          │   │
│  └─────────────────────────────┘   │
│                                     │
└─────────────────────────────────────┘
```

## 🔍 Monitoring & Management

### View Logs
```bash
# All services
docker compose logs -f

# Specific service
docker compose logs -f ande-node

# Last 100 lines
docker compose logs --tail=100 ande-node
```

### Service Management
```bash
# Start services
sudo systemctl start ande-chain

# Stop services
sudo systemctl stop ande-chain

# Restart services
sudo systemctl restart ande-chain

# Check status
sudo systemctl status ande-chain
```

### Health Checks
```bash
# Check chain ID
curl -X POST -H "Content-Type: application/json" \
  --data '{"jsonrpc":"2.0","method":"eth_chainId","params":[],"id":1}' \
  http://192.168.0.8:8545

# Check latest block
curl -X POST -H "Content-Type: application/json" \
  --data '{"jsonrpc":"2.0","method":"eth_blockNumber","params":[],"id":1}' \
  http://192.168.0.8:8545

# Check node info
curl -X POST -H "Content-Type: application/json" \
  --data '{"jsonrpc":"2.0","method":"web3_clientVersion","params":[],"id":1}' \
  http://192.168.0.8:8545
```

## 📊 Monitoring Dashboards

### Grafana
Access at: `http://192.168.0.8:3000`
- Username: `admin`
- Password: `andechain2024` (change in production!)

### Prometheus
Access at: `http://192.168.0.8:9090`

### Block Explorer
Access at: `http://192.168.0.8:4000`

## 🔐 Security Considerations

1. **Change default passwords** in `.env`
2. **Generate new JWT secret** for production
3. **Use SSL/TLS** for external access (nginx reverse proxy)
4. **Configure firewall** to restrict access
5. **Regular backups** of chain data
6. **Monitor logs** for suspicious activity

## 🆘 Troubleshooting

### Node won't start
```bash
# Check Docker status
sudo systemctl status docker

# Check logs
docker compose logs ande-node

# Check disk space
df -h

# Check memory
free -h
```

### Can't connect to RPC
```bash
# Check if port is listening
sudo netstat -tlnp | grep 8545

# Check firewall
sudo ufw status

# Test locally
curl http://localhost:8545
```

### Reset everything
```bash
# Stop services
docker compose down -v

# Remove all data
rm -rf ~/ande-chain-data/*

# Start fresh
docker compose up -d
```

## 📝 Backup & Restore

### Backup
```bash
# Stop services
docker compose down

# Backup data
tar -czf ande-backup-$(date +%Y%m%d).tar.gz \
  ~/ande-chain-data \
  .env \
  docker-compose.yml

# Start services
docker compose up -d
```

### Restore
```bash
# Stop services
docker compose down

# Extract backup
tar -xzf ande-backup-20241114.tar.gz

# Start services
docker compose up -d
```

## 🚀 Updates

To update ANDE Chain:

```bash
# Navigate to directory
cd ~/ande-chain

# Pull latest changes
git pull

# Update containers
docker compose pull
docker compose up -d
```

## 📞 Support

- GitHub: https://github.com/ande-labs/ande-chain
- Discord: https://discord.gg/andechain
- Documentation: https://docs.ande.network

## 📄 License

MIT OR Apache-2.0