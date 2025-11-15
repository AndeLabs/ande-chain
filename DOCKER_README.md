# 🐳 ANDE Chain Docker Deployment

Complete production stack for running ANDE Chain with all services.

## 📋 Prerequisites

- Docker Engine 24.0+
- Docker Compose 2.20+
- 16GB RAM minimum (32GB recommended)
- 100GB free disk space

## 🚀 Quick Start

### 1. Configuration

```bash
# Copy environment template
cp .env.example .env

# Edit configuration (IMPORTANT: Change FAUCET_PRIVATE_KEY and secrets)
nano .env
```

### 2. Build & Start

```bash
# Build the ANDE node image
docker compose build ande-node

# Start all services
docker compose up -d

# View logs
docker compose logs -f ande-node
docker compose logs -f evolve
```

### 3. Check Status

```bash
# Check all services
docker compose ps

# Test RPC
curl -X POST -H "Content-Type: application/json" \
  --data '{"jsonrpc":"2.0","method":"eth_blockNumber","params":[],"id":1}' \
  http://localhost:8545
```

## 📊 Services Overview

| Service | Port | Description |
|---------|------|-------------|
| **ande-node** | 8545 | HTTP RPC Endpoint |
| | 8546 | WebSocket Endpoint |
| | 8551 | Engine API (JWT auth) |
| | 9001-9092 | Metrics (Prometheus) |
| | 30303 | P2P Network |
| **evolve** | 7331 | Sequencer RPC |
| | 7676 | Sequencer P2P |
| | 26660 | Sequencer Metrics |
| **celestia** | 26658 | Celestia Light Node RPC |
| | 2121 | Celestia P2P |
| **prometheus** | 9090 | Metrics Dashboard |
| **grafana** | 3000 | Monitoring Dashboard |
| **blockscout** | 4000 | Block Explorer |
| **faucet** | 8081 | Testnet Faucet |
| **nginx** | 80/443 | Reverse Proxy |

## 🔧 Service Management

### Start/Stop Services

```bash
# Start all
docker compose up -d

# Start specific service
docker compose up -d ande-node

# Stop all
docker compose down

# Stop and remove volumes (⚠️ deletes all data)
docker compose down -v
```

### View Logs

```bash
# All services
docker compose logs -f

# Specific service
docker compose logs -f ande-node

# Last 100 lines
docker compose logs --tail=100 ande-node
```

### Restart Services

```bash
# Restart all
docker compose restart

# Restart specific service
docker compose restart ande-node
```

## 🏗️ Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                      ANDE Chain Stack                        │
├─────────────────────────────────────────────────────────────┤
│                                                               │
│  ┌──────────────┐    ┌──────────────┐    ┌──────────────┐  │
│  │   Nginx      │───▶│  Blockscout  │───▶│  PostgreSQL  │  │
│  │ Reverse Proxy│    │   Explorer   │    │   Database   │  │
│  └──────────────┘    └──────────────┘    └──────────────┘  │
│         │                    │                               │
│         ▼                    ▼                               │
│  ┌──────────────────────────────────────────────────────┐  │
│  │            ANDE Node (Execution Layer)                │  │
│  │  • Custom Reth v1.8.2                                 │  │
│  │  • ANDE Precompile (0x00..FD)                        │  │
│  │  • Parallel EVM (Block-STM)                          │  │
│  │  • MEV Detection & Protection                         │  │
│  │  • HTTP/WS RPC (8545/8546)                           │  │
│  └───────────────────┬──────────────────────────────────┘  │
│                      │ Engine API (JWT)                      │
│                      ▼                                        │
│  ┌──────────────────────────────────────────────────────┐  │
│  │           Evolve Sequencer (Consensus)                │  │
│  │  • Block Production (Adaptive 1s-5s)                  │  │
│  │  • Transaction Ordering                               │  │
│  │  • DA Batch Submission                                │  │
│  └───────────────────┬──────────────────────────────────┘  │
│                      │ DA Submission                         │
│                      ▼                                        │
│  ┌──────────────────────────────────────────────────────┐  │
│  │        Celestia Light Node (DA Layer)                 │  │
│  │  • Mocha-4 Testnet                                    │  │
│  │  • Data Availability Sampling                         │  │
│  │  • Namespace: andechain-v1                           │  │
│  └──────────────────────────────────────────────────────┘  │
│                                                               │
│  ┌──────────────┐    ┌──────────────┐    ┌──────────────┐  │
│  │  Prometheus  │───▶│   Grafana    │    │    Loki      │  │
│  │   Metrics    │    │  Dashboards  │    │     Logs     │  │
│  └──────────────┘    └──────────────┘    └──────────────┘  │
│                                                               │
└─────────────────────────────────────────────────────────────┘
```

## ⚙️ Advanced Configuration

### Custom Genesis

Edit `specs/genesis.json` before first start:

```json
{
  "chainId": 6174,
  "alloc": {
    "0xYourAddress": {
      "balance": "0x1000000000000000000000"
    }
  }
}
```

### Resource Limits

Edit `docker-compose.yml` to adjust:

```yaml
services:
  ande-node:
    deploy:
      resources:
        limits:
          cpus: '8'      # Increase for more parallel workers
          memory: 32G    # Increase for larger state
```

### Enable/Disable Features

Via environment variables in `.env`:

```bash
# Disable parallel EVM
ENABLE_PARALLEL_EVM=false

# Disable MEV detection
ENABLE_MEV_DETECTION=false

# Increase logging
RUST_LOG=trace
```

## 🔍 Monitoring

### Access Dashboards

- **Grafana**: http://localhost:3000 (admin/andechain2024)
- **Prometheus**: http://localhost:9090
- **Blockscout**: http://localhost:4000

### Key Metrics

```bash
# Current block number
curl -X POST -H "Content-Type: application/json" \
  --data '{"jsonrpc":"2.0","method":"eth_blockNumber","params":[],"id":1}' \
  http://localhost:8545

# Pending transactions
curl -X POST -H "Content-Type: application/json" \
  --data '{"jsonrpc":"2.0","method":"eth_getBlockTransactionCountByNumber","params":["pending"],"id":1}' \
  http://localhost:8545

# Check Prometheus metrics
curl http://localhost:9001/metrics
```

## 🐛 Troubleshooting

### Node won't start

```bash
# Check logs
docker compose logs ande-node

# Common issues:
# 1. JWT not generated - wait for jwt-init to complete
# 2. Port conflict - check if ports are already in use
# 3. Permissions - check data directory permissions
```

### Sequencer can't connect

```bash
# Verify ande-node is running
docker compose ps ande-node

# Check Engine API
curl -X POST -H "Content-Type: application/json" \
  -H "Authorization: Bearer $(cat jwttoken/jwt.hex)" \
  --data '{"jsonrpc":"2.0","method":"engine_exchangeCapabilities","params":[[]],"id":1}' \
  http://localhost:8551
```

### Celestia sync issues

```bash
# Check Celestia logs
docker compose logs celestia

# Restart Celestia
docker compose restart celestia
```

### Reset Everything

```bash
# ⚠️ WARNING: This deletes ALL data!
docker compose down -v
rm -rf jwttoken/*
docker compose up -d
```

## 📦 Backup & Restore

### Backup

```bash
# Stop services
docker compose down

# Backup volumes
docker run --rm -v ande-node-data:/data -v $(pwd):/backup \
  alpine tar czf /backup/ande-node-backup.tar.gz -C /data .

# Backup database
docker run --rm -v postgres-data:/data -v $(pwd):/backup \
  alpine tar czf /backup/postgres-backup.tar.gz -C /data .
```

### Restore

```bash
# Restore ande-node data
docker run --rm -v ande-node-data:/data -v $(pwd):/backup \
  alpine sh -c "cd /data && tar xzf /backup/ande-node-backup.tar.gz"

# Restore database
docker run --rm -v postgres-data:/data -v $(pwd):/backup \
  alpine sh -c "cd /data && tar xzf /backup/postgres-backup.tar.gz"
```

## 🔐 Security Best Practices

1. **Change default passwords** in `.env`
2. **Generate new JWT secret** for production
3. **Use firewall** to restrict external access
4. **Enable SSL** via nginx + certbot
5. **Backup regularly** 
6. **Monitor logs** for suspicious activity

## 📚 Additional Resources

- [ANDE Chain Documentation](https://docs.ande.network)
- [Reth Documentation](https://paradigmxyz.github.io/reth/)
- [Celestia Docs](https://docs.celestia.org)
- [Production Deployment Guide](./docs/PRODUCTION_DEPLOYMENT.md)

## 🆘 Support

- GitHub Issues: https://github.com/ande-labs/ande-chain/issues
- Discord: https://discord.gg/andechain
- Documentation: https://docs.ande.network
