# ANDE Chain - Deployment Guide

> Complete guide for deploying ANDE Chain to production and testnet environments

Last Updated: 2025-11-16

---

## 📋 Overview

This guide covers:
- Local development setup
- Testnet deployment
- Production deployment
- Docker deployment
- Monitoring setup

---

## 🔧 Prerequisites

### System Requirements

```bash
# Minimum specs
CPU: 4 cores
RAM: 16 GB
Storage: 500 GB SSD
Network: 100 Mbps

# Recommended specs  
CPU: 8+ cores
RAM: 32 GB
Storage: 1 TB NVMe SSD
Network: 1 Gbps
```

### Software Requirements

```bash
# Rust nightly
rustup toolchain install nightly-2024-10-18
rustup default nightly-2024-10-18

# Docker (for containerized deployment)
docker --version  # >= 24.0
docker compose version  # >= 2.20

# Foundry (for contracts)
foundryup
```

---

## 🏠 Local Development

### 1. Build from Source

```bash
# Clone repository
git clone https://github.com/AndeLabs/ande-chain.git
cd ande-chain

# Build in release mode
cargo build --release

# Binary location
./target/release/ande-node
```

### 2. Setup Genesis

```bash
# Genesis file is at specs/genesis.json
# Customize if needed:
vim specs/genesis.json

# Important fields:
# - chainId: 6174 (ANDE testnet)
# - gasLimit: 30000000
# - alloc: Initial token distribution
```

### 3. Run Local Node

```bash
# Start node
./target/release/ande-node \
  --chain specs/genesis.json \
  --datadir ~/.ande \
  --http \
  --http.addr 0.0.0.0 \
  --http.port 8545 \
  --ws \
  --ws.addr 0.0.0.0 \
  --ws.port 8546

# With debug logging
RUST_LOG=debug ./target/release/ande-node ...
```

### 4. Verify Node is Running

```bash
# Check RPC
curl -X POST http://localhost:8545 \
  -H "Content-Type: application/json" \
  -d '{"jsonrpc":"2.0","method":"eth_chainId","params":[],"id":1}'

# Should return: {"jsonrpc":"2.0","result":"0x181e","id":1}
# (0x181e = 6174 in hex)
```

---

## 🧪 Testnet Deployment

### 1. Server Setup

```bash
# SSH into server
ssh user@your-server.com

# Update system
sudo apt update && sudo apt upgrade -y

# Install dependencies
sudo apt install -y build-essential curl git

# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
rustup toolchain install nightly-2024-10-18
rustup default nightly-2024-10-18
```

### 2. Clone and Build

```bash
# Clone repo
git clone https://github.com/AndeLabs/ande-chain.git
cd ande-chain

# Build
cargo build --release

# Move binary to system path
sudo cp target/release/ande-node /usr/local/bin/
```

### 3. Create Systemd Service

```bash
# Create service file
sudo vim /etc/systemd/system/ande-node.service
```

Content:
```ini
[Unit]
Description=ANDE Chain Node
After=network.target

[Service]
Type=simple
User=ande
WorkingDirectory=/home/ande/ande-chain
ExecStart=/usr/local/bin/ande-node \
  --chain /home/ande/ande-chain/specs/genesis.json \
  --datadir /var/lib/ande \
  --http \
  --http.addr 0.0.0.0 \
  --http.port 8545 \
  --ws \
  --ws.addr 0.0.0.0 \
  --ws.port 8546 \
  --metrics \
  --metrics.addr 0.0.0.0 \
  --metrics.port 9001

Restart=always
RestartSec=10

Environment="RUST_LOG=info"
Environment="RUST_BACKTRACE=1"

[Install]
WantedBy=multi-user.target
```

### 4. Start Service

```bash
# Reload systemd
sudo systemctl daemon-reload

# Enable service
sudo systemctl enable ande-node

# Start service
sudo systemctl start ande-node

# Check status
sudo systemctl status ande-node

# View logs
sudo journalctl -u ande-node -f
```

### 5. Configure Firewall

```bash
# Allow RPC
sudo ufw allow 8545/tcp

# Allow WebSocket
sudo ufw allow 8546/tcp

# Allow P2P (if running discovery)
sudo ufw allow 30303/tcp
sudo ufw allow 30303/udp

# Allow metrics (only from monitoring server)
sudo ufw allow from <monitoring-ip> to any port 9001

# Enable firewall
sudo ufw enable
```

---

## 🐳 Docker Deployment

### 1. Using Docker Compose

```bash
# Navigate to project
cd ande-chain

# Build and start
docker compose up -d

# View logs
docker compose logs -f ande-node

# Stop
docker compose down
```

### 2. Custom Docker Build

```bash
# Build image
docker build -t ande-chain:latest .

# Run container
docker run -d \
  --name ande-node \
  -p 8545:8545 \
  -p 8546:8546 \
  -p 9001:9001 \
  -v ande-data:/data \
  ande-chain:latest
```

### 3. Docker Compose File

See `docker-compose.yml` in project root.

Key services:
- `ande-node`: Main ANDE node
- `prometheus`: Metrics collection
- `grafana`: Metrics visualization

---

## 🚀 Production Deployment

### Architecture

```
┌─────────────────────────────────────────┐
│         Load Balancer (Nginx)           │
│              SSL/TLS                     │
└─────────────────────────────────────────┘
                  ↓
┌─────────────────────────────────────────┐
│         ANDE Node Cluster               │
│  ┌──────────┐  ┌──────────┐            │
│  │  Node 1  │  │  Node 2  │            │
│  └──────────┘  └──────────┘            │
└─────────────────────────────────────────┘
                  ↓
┌─────────────────────────────────────────┐
│        Monitoring Stack                 │
│  Prometheus + Grafana + Alertmanager    │
└─────────────────────────────────────────┘
```

### 1. Load Balancer Setup

**Nginx Configuration** (`/etc/nginx/sites-available/ande-rpc`):

```nginx
upstream ande_rpc {
    least_conn;
    server 10.0.1.10:8545 max_fails=3 fail_timeout=30s;
    server 10.0.1.11:8545 max_fails=3 fail_timeout=30s;
}

server {
    listen 443 ssl http2;
    server_name rpc.andechain.com;

    ssl_certificate /etc/letsencrypt/live/andechain.com/fullchain.pem;
    ssl_certificate_key /etc/letsencrypt/live/andechain.com/privkey.pem;

    location / {
        proxy_pass http://ande_rpc;
        proxy_http_version 1.1;
        proxy_set_header Upgrade $http_upgrade;
        proxy_set_header Connection "upgrade";
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto $scheme;
        
        # Timeouts
        proxy_connect_timeout 60s;
        proxy_send_timeout 60s;
        proxy_read_timeout 60s;
        
        # Rate limiting
        limit_req zone=rpc_limit burst=20 nodelay;
    }
}
```

### 2. SSL/TLS Setup

```bash
# Install Certbot
sudo apt install certbot python3-certbot-nginx

# Get certificate
sudo certbot --nginx -d rpc.andechain.com

# Auto-renewal
sudo certbot renew --dry-run
```

### 3. Database Backup

```bash
# Backup script
#!/bin/bash
BACKUP_DIR="/backup/ande"
DATE=$(date +%Y%m%d_%H%M%S)

# Stop node
sudo systemctl stop ande-node

# Backup data directory
tar -czf $BACKUP_DIR/ande-data-$DATE.tar.gz /var/lib/ande

# Restart node
sudo systemctl start ande-node

# Keep only last 7 days
find $BACKUP_DIR -name "ande-data-*.tar.gz" -mtime +7 -delete
```

Schedule with cron:
```bash
# Daily backup at 2 AM
0 2 * * * /usr/local/bin/ande-backup.sh
```

---

## 📊 Monitoring Setup

### Prometheus Configuration

**File**: `prometheus.yml`

```yaml
global:
  scrape_interval: 15s
  evaluation_interval: 15s

scrape_configs:
  - job_name: 'ande-node'
    static_configs:
      - targets: ['localhost:9001']
        labels:
          instance: 'ande-node-1'
```

### Grafana Dashboard

Import dashboard from `monitoring/grafana-dashboard.json`

Key metrics:
- Block height
- Transaction throughput
- Gas usage
- Peer count
- Memory usage
- Disk I/O

### Alerting

**Alert Rules** (`alerts.yml`):

```yaml
groups:
  - name: ande-node
    interval: 30s
    rules:
      - alert: NodeDown
        expr: up{job="ande-node"} == 0
        for: 5m
        annotations:
          summary: "ANDE node is down"
          
      - alert: HighMemoryUsage
        expr: process_resident_memory_bytes > 28e9
        for: 10m
        annotations:
          summary: "High memory usage on ANDE node"
          
      - alert: BlockProductionStalled
        expr: increase(ande_chain_height[5m]) == 0
        for: 5m
        annotations:
          summary: "Block production has stalled"
```

---

## 🔐 Security Checklist

### Before Production

- [ ] Firewall configured
- [ ] SSL/TLS enabled
- [ ] SSH key-based auth only
- [ ] Fail2ban installed
- [ ] Regular backups scheduled
- [ ] Monitoring active
- [ ] Alerts configured
- [ ] Rate limiting enabled
- [ ] DDoS protection (Cloudflare)
- [ ] Secrets in environment variables (not code)

### Ongoing

- [ ] Regular security updates
- [ ] Log rotation configured
- [ ] Monitor for unusual activity
- [ ] Test disaster recovery
- [ ] Keep backups offsite

---

## 🆘 Troubleshooting

### Node Won't Start

```bash
# Check logs
sudo journalctl -u ande-node -n 100

# Common issues:
# 1. Port already in use
sudo lsof -i :8545

# 2. Database corrupted
rm -rf /var/lib/ande/db
# Restart node (will resync)

# 3. Out of disk space
df -h
```

### High Memory Usage

```bash
# Check memory
free -h

# Check node process
ps aux | grep ande-node

# Restart node
sudo systemctl restart ande-node
```

### Network Issues

```bash
# Check connectivity
ping rpc.andechain.com

# Check ports
sudo netstat -tulpn | grep ande-node

# Check firewall
sudo ufw status
```

---

## 📚 Additional Resources

- **Docker Guide**: `DOCKER_README.md`
- **Monitoring Access**: `MONITORING_ACCESS.md`  
- **Infrastructure Status**: See monitoring dashboard

---

## 📝 Post-Deployment

### Verify Deployment

```bash
# 1. Check node is syncing
curl -X POST https://rpc.andechain.com \
  -H "Content-Type: application/json" \
  -d '{"jsonrpc":"2.0","method":"eth_syncing","params":[],"id":1}'

# 2. Check block number
curl -X POST https://rpc.andechain.com \
  -H "Content-Type: application/json" \
  -d '{"jsonrpc":"2.0","method":"eth_blockNumber","params":[],"id":1}'

# 3. Check peer count
curl -X POST https://rpc.andechain.com \
  -H "Content-Type: application/json" \
  -d '{"jsonrpc":"2.0","method":"net_peerCount","params":[],"id":1}'
```

### Update Deployment

```bash
# 1. Pull latest code
git pull origin main

# 2. Rebuild
cargo build --release

# 3. Stop service
sudo systemctl stop ande-node

# 4. Update binary
sudo cp target/release/ande-node /usr/local/bin/

# 5. Start service
sudo systemctl start ande-node

# 6. Verify
sudo systemctl status ande-node
```

---

**Maintained by**: ANDE Labs DevOps Team  
**Last Updated**: 2025-11-16  
**Version**: 1.0.0

For issues, see [Troubleshooting](#troubleshooting) or contact DevOps team.
