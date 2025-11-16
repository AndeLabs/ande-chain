# ANDE Chain Multi-Sequencer Deployment Plan

**Version**: v1.0
**Date**: November 15, 2024
**Status**: PLANNING
**Target**: Deploy 3-sequencer configuration with CometBFT consensus

---

## 🎯 Objective

Migrate from single-node architecture to **3-sequencer multi-node consensus** with:
- CometBFT consensus engine (2000+ lines implemented)
- Automatic proposer rotation every 100 blocks
- Byzantine fault tolerance (BFT)
- Load balancing via HAProxy
- Complete monitoring of all nodes

---

## 📊 Current Architecture

```
Current (Single Node):
┌─────────────┐
│  ande-node  │  Port 8545 (RPC)
│  (Reth)     │  Port 8546 (WS)
└─────────────┘
       │
       ▼
  ┌─────────┐
  │ evolve  │  Sequencer
  └─────────┘
       │
       ▼
  ┌──────────┐
  │celestia  │  DA Layer
  └──────────┘

Monitoring:
- Prometheus (9093)
- Grafana (3001)
```

---

## 🏗️ Target Architecture

```
Target (Multi-Sequencer):
                    ┌─────────────┐
                    │   HAProxy   │  Port 80/443
                    │Load Balancer│  (Round-robin)
                    └─────────────┘
                           │
        ┌──────────────────┼──────────────────┐
        │                  │                  │
        ▼                  ▼                  ▼
┌─────────────┐    ┌─────────────┐    ┌─────────────┐
│ Sequencer 1 │    │ Sequencer 2 │    │ Sequencer 3 │
│  (Primary)  │    │  (Backup)   │    │  (Backup)   │
│  :8545      │    │  :8547      │    │  :8549      │
│  :8546      │    │  :8548      │    │  :8550      │
│  :9090      │    │  :9091      │    │  :9092      │
└─────────────┘    └─────────────┘    └─────────────┘
        │                  │                  │
        └──────────────────┼──────────────────┘
                           │
                    P2P Network (:30303-30305)
                           │
        ┌──────────────────┼──────────────────┐
        │                  │                  │
        ▼                  ▼                  ▼
┌─────────────┐    ┌──────────────┐   ┌──────────────┐
│  Celestia   │    │  Prometheus  │   │   Grafana    │
│ (DA Layer)  │    │  (Metrics)   │   │ (Dashboard)  │
└─────────────┘    └──────────────┘   └──────────────┘

Consensus Layer (CometBFT):
- Validator Set: 3 nodes
- Rotation: Every 100 blocks
- Timeout: 10 blocks
- Block Time: ~2 seconds (target)
```

---

## 📝 Implementation Components

### 1. Consensus Engine (✅ Implemented)
**Location**: `crates/ande-consensus/`

**Components**:
- `config.rs`: Chain configuration and validator setup (236 lines)
- `types.rs`: Core types (Block, Vote, ValidatorInfo) (214 lines)
- `engine.rs`: Consensus state machine (440 lines)
- `validator_set.rs`: Validator management (520 lines)
- `contract_client.rs`: Ethereum contract integration (321 lines)
- `metrics.rs`: Prometheus metrics (245 lines)
- `error.rs`: Error handling (114 lines)

**Total**: 2000+ lines of production consensus code

### 2. Smart Contracts (✅ Implemented)
**Location**: `contracts/src/consensus/`

**Contracts**:
- `AndeConsensus.sol`: Main consensus contract
- `AndeSequencerCoordinator.sol`: Sequencer coordination

**Features**:
- Validator registration
- Proposer rotation
- Timeout handling
- Slashing mechanisms

### 3. Integration Layer (✅ Implemented)
**Location**: `crates/ande-node/src/consensus_integration.rs`

**Features**:
- Payload builder integration
- Transaction ordering with consensus
- Block production coordination
- Proposer rotation handling

### 4. Docker Infrastructure (✅ Configured)
**Files**:
- `docker-compose-testnet.yml`: Full 3-sequencer setup
- `.env.testnet`: Environment configuration
- `monitoring/`: Prometheus + Grafana configs

---

## 🔄 Migration Strategy

### Phase 1: Preparation (Pre-deployment)
1. ✅ Code verification (all compiling)
2. ✅ Integration tests passed
3. ✅ Documentation complete
4. ⏳ Generate sequencer wallets (3 wallets)
5. ⏳ Fund wallets with testnet tokens
6. ⏳ Deploy consensus contracts to testnet

### Phase 2: Contract Deployment
1. Deploy `AndeConsensus` contract
2. Deploy `AndeSequencerCoordinator` contract
3. Register 3 validators
4. Verify contract deployment
5. Update `.env.testnet` with addresses

### Phase 3: Infrastructure Deployment
1. Stop current single-node stack
2. Backup current state
3. Deploy 3-sequencer docker stack
4. Configure P2P networking
5. Initialize consensus layer

### Phase 4: Verification
1. Verify all 3 nodes are running
2. Check P2P connectivity
3. Monitor consensus rotation
4. Test load balancer
5. Verify Prometheus metrics

### Phase 5: Monitoring Setup
1. Configure Prometheus for 3 targets
2. Update Grafana dashboards
3. Set up alerts
4. Verify HAProxy stats

---

## 📋 Pre-Deployment Checklist

### Environment Setup
- [ ] Generate 3 sequencer wallets
- [ ] Fund wallets with TIA (Celestia Mocha-4)
- [ ] Fund wallets with ETH (for gas)
- [ ] Configure `.env.testnet.local` with private keys

### Contract Deployment
- [ ] Compile Solidity contracts
- [ ] Deploy `AndeConsensus` to testnet
- [ ] Deploy `AndeSequencerCoordinator` to testnet
- [ ] Verify contracts on explorer
- [ ] Update `.env.testnet` with addresses

### Infrastructure
- [ ] Backup current blockchain state
- [ ] Prepare docker volumes for 3 nodes
- [ ] Configure networking (ports 8545-8550)
- [ ] Set up JWT tokens
- [ ] Configure HAProxy rules

### Monitoring
- [ ] Update Prometheus config for 3 targets
- [ ] Create multi-sequencer Grafana dashboards
- [ ] Set up alert rules
- [ ] Configure HAProxy stats

---

## 🚀 Deployment Steps

### Step 1: Generate Wallets
```bash
# Generate 3 wallets using Foundry
cast wallet new > sequencer-1-wallet.txt
cast wallet new > sequencer-2-wallet.txt
cast wallet new > sequencer-3-wallet.txt

# Extract addresses and private keys
# Add to .env.testnet.local
```

### Step 2: Fund Wallets
```bash
# Get testnet tokens from faucets
# TIA: https://faucet.celestia-mocha.com/
# ETH Sepolia: https://sepoliafaucet.com/

# Verify balances
cast balance SEQUENCER_1_ADDRESS --rpc-url $CONSENSUS_RPC_URL
cast balance SEQUENCER_2_ADDRESS --rpc-url $CONSENSUS_RPC_URL
cast balance SEQUENCER_3_ADDRESS --rpc-url $CONSENSUS_RPC_URL
```

### Step 3: Deploy Contracts
```bash
cd contracts

# Deploy AndeConsensus
forge create \
  --rpc-url $CONSENSUS_RPC_URL \
  --private-key $SEQUENCER_1_PRIVATE_KEY \
  src/consensus/AndeConsensus.sol:AndeConsensus \
  --json | tee ande-consensus-deployment.json

# Deploy AndeSequencerCoordinator
CONSENSUS_ADDRESS=$(jq -r '.deployedTo' ande-consensus-deployment.json)
forge create \
  --rpc-url $CONSENSUS_RPC_URL \
  --private-key $SEQUENCER_1_PRIVATE_KEY \
  --constructor-args $CONSENSUS_ADDRESS \
  src/consensus/AndeSequencerCoordinator.sol:AndeSequencerCoordinator \
  --json | tee coordinator-deployment.json
```

### Step 4: Register Validators
```bash
# Register sequencer 1
cast send \
  --rpc-url $CONSENSUS_RPC_URL \
  --private-key $SEQUENCER_1_PRIVATE_KEY \
  $CONSENSUS_ADDRESS \
  "registerValidator(bytes32,string,uint256)" \
  $P2P_PEER_ID_1 \
  "http://sequencer-1:8545" \
  1000000000000000000000  # 1000 ANDE stake

# Repeat for sequencers 2 and 3
```

### Step 5: Deploy Multi-Sequencer Stack
```bash
# Stop current stack
docker-compose -f docker-compose-quick.yml down

# Deploy 3-sequencer stack
docker-compose -f docker-compose-testnet.yml up -d

# Monitor logs
docker-compose -f docker-compose-testnet.yml logs -f
```

### Step 6: Verify Deployment
```bash
# Check all containers
docker ps

# Verify RPC endpoints
curl http://localhost:8545 -X POST --data '{"jsonrpc":"2.0","method":"eth_blockNumber","params":[],"id":1}'
curl http://localhost:8547 -X POST --data '{"jsonrpc":"2.0","method":"eth_blockNumber","params":[],"id":1}'
curl http://localhost:8549 -X POST --data '{"jsonrpc":"2.0","method":"eth_blockNumber","params":[],"id":1}'

# Check consensus state
cast call \
  --rpc-url http://localhost:8545 \
  $CONSENSUS_ADDRESS \
  "getActiveValidators()" | jq
```

---

## 🎯 Success Criteria

### Deployment Success
- ✅ All 3 sequencer containers running
- ✅ All RPC endpoints responding
- ✅ P2P network established
- ✅ Consensus contract deployed
- ✅ All validators registered

### Operational Success
- ✅ Blocks producing consistently
- ✅ Proposer rotating every 100 blocks
- ✅ All nodes synced (within 5 blocks)
- ✅ HAProxy load balancing working
- ✅ Prometheus collecting metrics from all 3 nodes
- ✅ Grafana dashboards showing all data

### Performance Targets
- **Block Time**: ~2 seconds (target)
- **Rotation**: Automatic every 100 blocks
- **Sync Latency**: <5 blocks difference
- **RPC Latency**: <200ms
- **Uptime**: >99%

---

## ⚠️ Risks & Mitigation

### Risk 1: Consensus Failure
**Risk**: Nodes fail to reach consensus
**Mitigation**:
- Thorough pre-testing
- Gradual rollout
- Quick rollback plan
- Monitor logs closely

### Risk 2: Network Partitioning
**Risk**: P2P network splits
**Mitigation**:
- Configure bootnodes properly
- Monitor P2P connections
- Set up network alerts

### Risk 3: State Inconsistency
**Risk**: Nodes have different state
**Mitigation**:
- Start from same genesis
- Sync before enabling consensus
- Verify state hashes match

### Risk 4: Performance Degradation
**Risk**: System slower with 3 nodes
**Mitigation**:
- Load testing before production
- Optimize P2P networking
- Tune consensus parameters

---

## 🔄 Rollback Plan

If deployment fails:

1. **Stop new stack**:
   ```bash
   docker-compose -f docker-compose-testnet.yml down
   ```

2. **Restore previous stack**:
   ```bash
   docker-compose -f docker-compose-quick.yml up -d
   ```

3. **Verify restoration**:
   ```bash
   curl http://localhost:8545 -X POST --data '{"jsonrpc":"2.0","method":"eth_blockNumber","params":[],"id":1}'
   ```

4. **Analyze logs**:
   ```bash
   docker-compose -f docker-compose-testnet.yml logs > deployment-failure.log
   ```

5. **Debug and retry**

---

## 📊 Monitoring During Deployment

### Key Metrics to Watch

**Consensus Metrics**:
```promql
# Active validators
ande_consensus_active_validators

# Current proposer
ande_consensus_current_proposer

# Rotation count
ande_consensus_rotation_count

# Timeout events
ande_consensus_timeout_events_total
```

**Node Health**:
```promql
# Node uptime
up{job=~"sequencer.*"}

# Block height
ande_chain_block_height{instance=~"sequencer.*"}

# Sync lag
ande_chain_sync_lag_blocks
```

**Performance**:
```promql
# Block time
ande_consensus_block_time_seconds

# TPS
rate(ande_evm_transactions_total[1m])

# P2P connections
ande_p2p_peers_count
```

---

## 📚 Documentation Updates Needed

Post-deployment documentation:
1. Update `DEPLOYMENT_STATUS.md` with 3-sequencer info
2. Create `CONSENSUS_OPERATIONS.md` guide
3. Update `MONITORING_ACCESS.md` with new metrics
4. Document validator registration process
5. Create troubleshooting guide for consensus issues

---

## ✅ Next Steps

**Immediate** (this session):
1. Generate sequencer wallets
2. Configure `.env.testnet.local`
3. Plan contract deployment strategy

**Short-term** (next 24h):
1. Deploy contracts to Celestia Mocha-4
2. Register validators
3. Deploy 3-sequencer stack
4. Verify consensus rotation

**Mid-term** (next week):
1. Load testing
2. Optimize block time
3. Fine-tune consensus parameters
4. Production hardening

---

**Status**: Ready for execution
**Next Action**: Generate wallets and fund with testnet tokens
**Expected Duration**: 2-3 hours for complete deployment
**Risk Level**: Medium (comprehensive testing done)

---

**Created**: November 15, 2024
**Version**: v1.0
**Author**: ANDE Labs Team
