# ANDE Chain Deployment Guide

## Prerequisites

### System Requirements

**Minimum**:
- CPU: 4 cores
- RAM: 16 GB
- Storage: 500 GB SSD
- Network: 100 Mbps

**Recommended**:
- CPU: 8+ cores
- RAM: 32 GB
- Storage: 1 TB NVMe SSD
- Network: 1 Gbps

### Software Dependencies

```bash
# Rust toolchain (1.88+)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
rustup update

# Build essentials
sudo apt-get update
sudo apt-get install -y build-essential pkg-config libssl-dev

# Docker (optional, for containerized deployment)
curl -fsSL https://get.docker.com -o get-docker.sh
sudo sh get-docker.sh
```

## Building from Source

### 1. Clone Repository

```bash
git clone https://github.com/AndeLabs/ande-chain.git
cd ande-chain
```

### 2. Build Release Binary

```bash
# Full release build (optimized)
cargo build --release

# Binary location
ls -lh target/release/ande-reth
```

**Build time**: ~20-30 minutes (first build)

### 3. Verify Build

```bash
./target/release/ande-reth --version
```

## Configuration

### 1. Environment Variables

Create `.env` file:

```bash
# Network Configuration
CHAIN_ID=6174
NETWORK_ID=6174

# Consensus Configuration (Multi-Validator)
ANDE_CONSENSUS_ENABLED=true
ANDE_CONSENSUS_VALIDATORS='[
  {"address":"0x0000000000000000000000000000000000000001","weight":100},
  {"address":"0x0000000000000000000000000000000000000002","weight":50}
]'
ANDE_CONSENSUS_THRESHOLD=67

# MEV Configuration
ANDE_MEV_ENABLED=true
ANDE_MEV_SINK=0x0000000000000000000000000000000000000042
ANDE_MEV_MIN_THRESHOLD=1000000000000000

# RPC Configuration
HTTP_RPC_ADDR=0.0.0.0
HTTP_RPC_PORT=8545
WS_RPC_ADDR=0.0.0.0
WS_RPC_PORT=8546

# P2P Configuration
P2P_PORT=30303
DISCOVERY_PORT=30303

# Logging
RUST_LOG=info,ande_reth=debug,ande_consensus=debug
```

### 2. Genesis Configuration

Create `genesis.json`:

```json
{
  "config": {
    "chainId": 6174,
    "homesteadBlock": 0,
    "eip150Block": 0,
    "eip155Block": 0,
    "eip158Block": 0,
    "byzantiumBlock": 0,
    "constantinopleBlock": 0,
    "petersburgBlock": 0,
    "istanbulBlock": 0,
    "berlinBlock": 0,
    "londonBlock": 0,
    "shanghaiBlock": 0,
    "cancunBlock": 0,
    "andeConfig": {
      "tokenDualityPrecompile": "0x00000000000000000000000000000000000000fd",
      "mevDistributionContract": "0x0000000000000000000000000000000000000042"
    }
  },
  "nonce": "0x0",
  "timestamp": "0x0",
  "extraData": "0x",
  "gasLimit": "0x1c9c380",
  "difficulty": "0x1",
  "mixHash": "0x0000000000000000000000000000000000000000000000000000000000000000",
  "coinbase": "0x0000000000000000000000000000000000000000",
  "alloc": {
    "0x0000000000000000000000000000000000000001": {
      "balance": "0x200000000000000000000000000000000000000000000000000000000000000"
    }
  }
}
```

## Deployment Options

### Option 1: Standalone Node

```bash
# Initialize database with genesis
./target/release/ande-reth init --chain genesis.json

# Start node
./target/release/ande-reth node \
  --chain genesis.json \
  --http \
  --http.addr 0.0.0.0 \
  --http.port 8545 \
  --http.api eth,net,web3,debug,trace \
  --ws \
  --ws.addr 0.0.0.0 \
  --ws.port 8546 \
  --authrpc.addr 127.0.0.1 \
  --authrpc.port 8551 \
  --authrpc.jwtsecret /path/to/jwt.hex
```

### Option 2: Docker Deployment

**Dockerfile**:

```dockerfile
FROM rust:1.88 as builder

WORKDIR /app
COPY . .

RUN cargo build --release

FROM ubuntu:22.04

RUN apt-get update && apt-get install -y \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

COPY --from=builder /app/target/release/ande-reth /usr/local/bin/

EXPOSE 8545 8546 8551 30303 30303/udp

ENTRYPOINT ["ande-reth"]
CMD ["node"]
```

**Docker Compose**:

```yaml
version: '3.8'

services:
  ande-node:
    build: .
    container_name: ande-chain
    ports:
      - "8545:8545"
      - "8546:8546"
      - "8551:8551"
      - "30303:30303"
      - "30303:30303/udp"
    environment:
      - ANDE_CONSENSUS_ENABLED=true
      - ANDE_MEV_ENABLED=true
      - RUST_LOG=info
    volumes:
      - ./data:/data
      - ./genesis.json:/genesis.json
      - ./jwt.hex:/jwt.hex
    command:
      - node
      - --chain
      - /genesis.json
      - --datadir
      - /data
      - --http
      - --http.addr
      - 0.0.0.0
      - --http.port
      - "8545"
      - --authrpc.jwtsecret
      - /jwt.hex
    restart: unless-stopped
```

### Option 3: Systemd Service

Create `/etc/systemd/system/ande-chain.service`:

```ini
[Unit]
Description=ANDE Chain Node
After=network.target

[Service]
Type=simple
User=ande
Group=ande
WorkingDirectory=/opt/ande-chain
EnvironmentFile=/opt/ande-chain/.env
ExecStart=/opt/ande-chain/ande-reth node \
  --chain /opt/ande-chain/genesis.json \
  --datadir /opt/ande-chain/data \
  --http \
  --http.addr 0.0.0.0 \
  --http.port 8545 \
  --ws \
  --ws.addr 0.0.0.0 \
  --ws.port 8546 \
  --authrpc.jwtsecret /opt/ande-chain/jwt.hex

Restart=on-failure
RestartSec=10s

# Security hardening
NoNewPrivileges=true
PrivateTmp=true
ProtectSystem=strict
ProtectHome=true
ReadWritePaths=/opt/ande-chain/data

[Install]
WantedBy=multi-user.target
```

Enable and start:

```bash
sudo systemctl daemon-reload
sudo systemctl enable ande-chain
sudo systemctl start ande-chain
sudo systemctl status ande-chain
```

## Validator Setup

### 1. Generate Validator Keys

```bash
# Generate new validator address (use secure key management)
cast wallet new

# Fund validator address with ANDE tokens
# Address needs stake to participate in consensus
```

### 2. Configure Validator

Update `.env`:

```bash
ANDE_CONSENSUS_VALIDATORS='[
  {"address":"YOUR_VALIDATOR_ADDRESS","weight":100}
]'
```

### 3. Register Validator (Smart Contract)

```bash
# Deploy validator registration (future)
# Will be handled via ValidatorRegistry.sol contract
```

## Monitoring

### Logs

```bash
# Follow logs (systemd)
journalctl -u ande-chain -f

# Docker logs
docker logs -f ande-chain

# Direct output
tail -f /opt/ande-chain/data/logs/ande-reth.log
```

### Metrics

```bash
# Check sync status
curl -X POST http://localhost:8545 \
  -H "Content-Type: application/json" \
  -d '{"jsonrpc":"2.0","method":"eth_syncing","params":[],"id":1}'

# Get block number
curl -X POST http://localhost:8545 \
  -H "Content-Type: application/json" \
  -d '{"jsonrpc":"2.0","method":"eth_blockNumber","params":[],"id":1}'

# Check peer count
curl -X POST http://localhost:8545 \
  -H "Content-Type: application/json" \
  -d '{"jsonrpc":"2.0","method":"net_peerCount","params":[],"id":1}'
```

### Prometheus Metrics (Optional)

Add to node command:

```bash
--metrics \
--metrics.addr 0.0.0.0 \
--metrics.port 9001
```

Access metrics at `http://localhost:9001/metrics`

## Testing Deployment

### 1. Test Token Duality Precompile

```bash
# Call balanceOf via precompile
cast call 0x00000000000000000000000000000000000000fd \
  "balanceOf(address)(uint256)" \
  0x0000000000000000000000000000000000000001 \
  --rpc-url http://localhost:8545
```

### 2. Test Transaction Submission

```bash
# Send test transaction
cast send 0xRecipientAddress \
  --value 1ether \
  --private-key YOUR_PRIVATE_KEY \
  --rpc-url http://localhost:8545
```

### 3. Test BFT Consensus

```bash
# Check current proposer (logs)
journalctl -u ande-chain | grep "Current proposer"

# Verify block production
watch -n 2 'cast block-number --rpc-url http://localhost:8545'
```

## Troubleshooting

### Node Won't Start

```bash
# Check logs
journalctl -u ande-chain -n 100

# Verify genesis hash
./target/release/ande-reth db stats --chain genesis.json

# Clear database if needed (CAUTION: Data loss)
rm -rf data/db
./target/release/ande-reth init --chain genesis.json
```

### Sync Issues

```bash
# Check peers
cast rpc net_peerCount --rpc-url http://localhost:8545

# Add bootnodes
--bootnodes enode://...@ip:port
```

### High Memory Usage

```bash
# Monitor memory
htop

# Adjust cache size
--db.cache-size 2GB

# Enable MDBX optimization
--db.max-size 1TB
```

## Security Best Practices

### 1. Firewall Configuration

```bash
# Allow only necessary ports
sudo ufw allow 22/tcp     # SSH
sudo ufw allow 8545/tcp   # HTTP RPC
sudo ufw allow 8546/tcp   # WS RPC
sudo ufw allow 30303      # P2P
sudo ufw enable
```

### 2. JWT Secret Protection

```bash
# Generate secure JWT secret
openssl rand -hex 32 > jwt.hex
chmod 600 jwt.hex
chown ande:ande jwt.hex
```

### 3. RPC Access Control

```bash
# Bind to localhost only for production
--http.addr 127.0.0.1
--ws.addr 127.0.0.1

# Use reverse proxy (nginx) with authentication
# See: docs/NGINX_PROXY_SETUP.md
```

### 4. Regular Backups

```bash
# Backup data directory
tar -czf ande-backup-$(date +%Y%m%d).tar.gz data/

# Backup configuration
cp genesis.json .env jwt.hex backup/
```

## Upgrade Procedure

### 1. Prepare Upgrade

```bash
# Stop node
sudo systemctl stop ande-chain

# Backup current state
tar -czf pre-upgrade-backup.tar.gz data/
```

### 2. Build New Version

```bash
git pull origin main
cargo build --release
```

### 3. Verify Compatibility

```bash
# Check changelog
cat CHANGELOG.md

# Test new binary
./target/release/ande-reth --version
```

### 4. Deploy Upgrade

```bash
# Replace binary
sudo cp target/release/ande-reth /opt/ande-chain/

# Restart node
sudo systemctl start ande-chain

# Monitor logs
journalctl -u ande-chain -f
```

## Production Checklist

- [ ] Hardware meets recommended specs
- [ ] Rust toolchain installed and updated
- [ ] ANDE Chain built in release mode
- [ ] Genesis configuration created
- [ ] Environment variables configured
- [ ] JWT secret generated securely
- [ ] Firewall rules configured
- [ ] Systemd service configured
- [ ] Monitoring setup (logs, metrics)
- [ ] Backup strategy implemented
- [ ] Security hardening applied
- [ ] Test transactions successful
- [ ] Validator keys secured
- [ ] Documentation reviewed

## Support

- **Documentation**: `docs/`
- **Issues**: https://github.com/AndeLabs/ande-chain/issues
- **Discord**: https://discord.gg/andelabs
- **Email**: support@andelabs.io

---

**Last Updated**: 2025-11-16  
**Version**: 1.0.0
