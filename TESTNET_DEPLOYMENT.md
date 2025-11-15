# ANDE Chain Testnet Deployment Guide

## 🎯 Overview

This guide provides professional deployment instructions for ANDE Chain testnet on Celestia Mocha-4 with multi-sequencer consensus.

**Network Details:**
- Chain ID: 6174
- Network Name: ANDE Testnet
- Consensus: CometBFT Multi-Sequencer
- Data Availability: Celestia Mocha-4
- Block Time: ~2 seconds

---

## 📋 Prerequisites

### Required Software

```bash
# Rust (latest stable)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Foundry (for Solidity contracts)
curl -L https://foundry.paradigm.xyz | bash
foundryup

# Docker & Docker Compose
# See: https://docs.docker.com/get-docker/

# jq (for JSON processing)
sudo apt-get install jq  # Ubuntu/Debian
brew install jq          # macOS
```

### Testnet Tokens

You'll need testnet tokens for:
1. **TIA** (Celestia Mocha-4) - For DA submissions
   - Faucet: https://faucet.celestia-mocha.com/
2. **ETH** (Sepolia) - For consensus contract deployment
   - Faucet: https://sepoliafaucet.com/

---

## 🔧 Configuration

### 1. Environment Setup

Copy and configure the testnet environment file:

```bash
cp .env.testnet .env.testnet.local
```

**Critical variables to configure:**

```bash
# Sequencer wallets (generate 3 separate wallets)
SEQUENCER_1_ADDRESS=0x...
SEQUENCER_1_PRIVATE_KEY=0x...

SEQUENCER_2_ADDRESS=0x...
SEQUENCER_2_PRIVATE_KEY=0x...

SEQUENCER_3_ADDRESS=0x...
SEQUENCER_3_PRIVATE_KEY=0x...

# Celestia authentication
CELESTIA_AUTH_TOKEN=your_auth_token_here

# RPC endpoint (use your own or public)
CONSENSUS_RPC_URL=https://rpc.celestia-mocha.com
CONSENSUS_WS_URL=wss://rpc.celestia-mocha.com/websocket
```

### 2. Generate Sequencer Wallets

```bash
# Using cast from Foundry
cast wallet new

# Repeat 3 times for 3 sequencers
# Save addresses and private keys in .env.testnet.local
```

### 3. Fund Wallets

Fund each sequencer address with:
- **TIA**: For DA submissions (~10 TIA per sequencer)
- **ETH**: For gas fees (~0.1 ETH per sequencer)

---

## 🚀 Deployment

### Automated Deployment

Run the automated deployment script:

```bash
./scripts/deploy-testnet.sh
```

This script will:
1. ✅ Compile Rust workspace
2. ✅ Compile Solidity contracts
3. ✅ Deploy consensus contracts
4. ✅ Build Docker images
5. ✅ Start all services
6. ✅ Verify deployment

### Manual Deployment

If you prefer manual deployment:

#### Step 1: Compile Code

```bash
# Compile Rust
cargo build --release --workspace

# Compile Solidity
cd contracts
forge build
```

#### Step 2: Deploy Contracts

```bash
cd contracts

# Deploy AndeConsensus
forge create \
  --rpc-url $CONSENSUS_RPC_URL \
  --private-key $SEQUENCER_1_PRIVATE_KEY \
  src/consensus/AndeConsensus.sol:AndeConsensus

# Deploy AndeSequencerCoordinator
forge create \
  --rpc-url $CONSENSUS_RPC_URL \
  --private-key $SEQUENCER_1_PRIVATE_KEY \
  --constructor-args <CONSENSUS_ADDRESS> \
  src/consensus/AndeSequencerCoordinator.sol:AndeSequencerCoordinator
```

Update contract addresses in `.env.testnet.local`.

#### Step 3: Start Services

```bash
docker compose -f docker-compose-testnet.yml up -d
```

---

## 📊 Monitoring & Verification

### Health Checks

```bash
# Check all services
docker compose -f docker-compose-testnet.yml ps

# View logs
docker compose -f docker-compose-testnet.yml logs -f

# Check specific service
docker compose -f docker-compose-testnet.yml logs -f ande-sequencer-1
```

### RPC Verification

```bash
# Check chain ID
curl -X POST \
  -H "Content-Type: application/json" \
  --data '{"jsonrpc":"2.0","method":"eth_chainId","params":[],"id":1}' \
  http://localhost:8545

# Expected: {"jsonrpc":"2.0","id":1,"result":"0x181e"}  # 6174 in hex

# Check block number
curl -X POST \
  -H "Content-Type: application/json" \
  --data '{"jsonrpc":"2.0","method":"eth_blockNumber","params":[],"id":1}' \
  http://localhost:8545
```

### Access Monitoring Dashboards

- **Prometheus**: http://localhost:9093
- **Grafana**: http://localhost:3001 (admin/ande2024)
- **Explorer**: http://localhost:3000
- **HAProxy Stats**: http://localhost:8404/stats

---

## 🔄 Multi-Sequencer Operations

### Register as Validator

```bash
# Using cast to call AndeConsensus
cast send \
  --rpc-url http://localhost:8545 \
  --private-key $SEQUENCER_1_PRIVATE_KEY \
  $CONSENSUS_CONTRACT_ADDRESS \
  "registerValidator(bytes32,string,uint256)" \
  <P2P_PEER_ID> \
  "http://sequencer-1:8545" \
  1000000000000000000000  # 1000 ANDE stake
```

### Check Consensus State

```bash
# Get active validators
cast call \
  --rpc-url http://localhost:8545 \
  $CONSENSUS_CONTRACT_ADDRESS \
  "getActiveValidators()" \
  | jq

# Get current proposer
cast call \
  --rpc-url http://localhost:8545 \
  $CONSENSUS_CONTRACT_ADDRESS \
  "getCurrentProposer()"

# Get validator info
cast call \
  --rpc-url http://localhost:8545 \
  $CONSENSUS_CONTRACT_ADDRESS \
  "getValidatorInfo(address)" \
  $SEQUENCER_1_ADDRESS
```

---

## 🧪 Testing

### Integration Tests

```bash
# Run all tests
cargo test --workspace

# Run consensus tests
cargo test -p ande-consensus

# Run with logs
RUST_LOG=debug cargo test --workspace -- --nocapture
```

### Load Testing

```bash
# Install artillery
npm install -g artillery

# Run load test (create load-test.yml first)
artillery run load-test.yml
```

---

## 🔒 Security Checklist

- [ ] All private keys stored securely (never commit to git)
- [ ] .env files added to .gitignore
- [ ] Firewall configured (allow only necessary ports)
- [ ] JWT token generated securely
- [ ] Regular backups configured
- [ ] Monitoring alerts set up
- [ ] Rate limiting enabled
- [ ] HTTPS/TLS configured for production

---

## 🛠 Maintenance

### Backup Database

```bash
# Stop services
docker compose -f docker-compose-testnet.yml stop

# Backup volumes
docker run --rm \
  -v ande-chain_sequencer-1-data:/data \
  -v $(pwd)/backups:/backup \
  alpine tar czf /backup/sequencer-1-$(date +%Y%m%d).tar.gz /data

# Restart services
docker compose -f docker-compose-testnet.yml start
```

### Update Deployment

```bash
# Pull latest changes
git pull origin main

# Rebuild and restart
docker compose -f docker-compose-testnet.yml down
docker compose -f docker-compose-testnet.yml build
docker compose -f docker-compose-testnet.yml up -d
```

### View Metrics

```bash
# Consensus metrics
curl http://localhost:9090/metrics | grep ande_consensus

# EVM metrics
curl http://localhost:9090/metrics | grep ande_evm
```

---

## 🆘 Troubleshooting

### Service Won't Start

```bash
# Check logs
docker compose -f docker-compose-testnet.yml logs <service-name>

# Check disk space
df -h

# Check ports in use
sudo lsof -i :<port>
```

### Consensus Not Syncing

1. Verify contract addresses in .env
2. Check sequencer has sufficient stake
3. Verify P2P connectivity between nodes
4. Check Celestia light node is synced

### Performance Issues

1. Check system resources: `docker stats`
2. Verify database isn't corrupted
3. Check network latency to Celestia
4. Review Prometheus metrics for bottlenecks

---

## 📚 Additional Resources

- **Documentation**: https://docs.andechain.com
- **Explorer**: https://testnet-explorer.andechain.com
- **Faucet**: https://faucet.andechain.com
- **Discord**: https://discord.gg/andechain
- **GitHub**: https://github.com/AndeLabs/ande-chain

---

## 🤝 Support

For support:
1. Check documentation
2. Search GitHub issues
3. Join Discord #testnet channel
4. Create GitHub issue with logs

---

**⚠️ DISCLAIMER**: This is testnet software. Not for production use with real value.
