# AndeChain Production Contracts - Classification & Deployment Strategy

> **Status**: Production Ready ✅  
> **Last Updated**: 2024  
> **Maintainer**: Ande Labs Team

---

## 🎯 Contract Classification

### 🔴 **TIER 1 - CORE (Must Deploy First)**

These contracts are essential for the chain to function and must be deployed in order:

| Contract | Path | Purpose | Dependencies | Status |
|----------|------|---------|--------------|--------|
| **ANDETokenDuality** | `src/ANDETokenDuality.sol` | Native + ERC20 token with voting | None | ✅ Ready |
| **AndeNativeStaking** | `src/staking/AndeNativeStaking.sol` | Staking system (3 pools) | ANDETokenDuality | ✅ Ready |
| **AndeSequencerRegistry** | `src/sequencer/AndeSequencerRegistry.sol` | Sequencer management | AndeNativeStaking | ✅ Ready |

**Deployment Order**: 
1. ANDETokenDuality
2. AndeNativeStaking
3. AndeSequencerRegistry

---

### 🟡 **TIER 2 - GOVERNANCE (Deploy After Core)**

Essential for on-chain governance and protocol upgrades:

| Contract | Path | Purpose | Dependencies | Status |
|----------|------|---------|--------------|--------|
| **AndeTimelockController** | `src/governance/AndeTimelockController.sol` | Timelock for governance | None | ✅ Ready |
| **AndeGovernor** | `src/governance/AndeGovernor.sol` | On-chain governance | ANDETokenDuality, Timelock | ⚠️ Complex |
| **AndeRollupGovernance** | `src/governance/AndeRollupGovernance.sol` | L2-specific governance | AndeGovernor | ⚠️ Review |

**Deployment Order**:
1. AndeTimelockController
2. AndeGovernor
3. AndeRollupGovernance (optional)

---

### 🟢 **TIER 3 - INFRASTRUCTURE (Deploy When Needed)**

Important infrastructure but not blocking:

| Contract | Path | Purpose | Status |
|----------|------|---------|--------|
| **AndeChainBridge** | `src/bridge/AndeChainBridge.sol` | Cross-chain bridge | 🔄 Review |
| **LazybridgeRelay** | `src/lazybridge/LazybridgeRelay.sol` | ZK lazy bridging | 🔄 Review |
| **WAndeVault** | `src/vaults/WAndeVault.sol` | Wrapped ANDE vault | ✅ Ready |
| **AndeFeeDistributor** | `src/tokenomics/AndeFeeDistributor.sol` | Fee distribution | ✅ Ready |
| **AndeVesting** | `src/tokenomics/AndeVesting.sol` | Token vesting | ✅ Ready |

---

### 🔵 **TIER 4 - DEFI ECOSYSTEM (Deploy Post-Launch)**

DeFi features to be deployed after mainnet launch:

#### DEX (Decentralized Exchange)
| Contract | Path | Status |
|----------|------|--------|
| **AndeSwapFactory** | `src/dex/AndeSwapFactory.sol` | ✅ Ready |
| **AndeSwapRouter** | `src/dex/AndeSwapRouter.sol` | ✅ Ready |
| **AndeSwapPair** | `src/dex/AndeSwapPair.sol` | ✅ Ready |

#### Lending
| Contract | Path | Status |
|----------|------|--------|
| **AndeLend** | `src/lending/AndeLend.sol` | 🔄 Audit |
| **AToken** | `src/lending/AToken.sol` | 🔄 Audit |

#### Gauges & ve-Economics
| Contract | Path | Status |
|----------|------|--------|
| **VotingEscrow** | `src/gauges/VotingEscrow.sol` | 🔄 Review |
| **GaugeController** | `src/gauges/GaugeController.sol` | 🔄 Review |
| **LiquidityGaugeV1** | `src/gauges/LiquidityGaugeV1.sol` | 🔄 Review |

---

### 🟣 **TIER 5 - ADVANCED FEATURES (Future)**

Advanced features for later phases:

| Feature | Contracts | Status |
|---------|-----------|--------|
| **Account Abstraction (ERC-4337)** | `src/account/*` | 🔄 ERC-4337 |
| **MEV Protection** | `src/mev/*` | 🔄 Review |
| **Perpetuals** | `src/perpetuals/AndePerpetuals.sol` | 🔄 Future |
| **Launchpad** | `src/launchpad/*` | 🔄 Future |
| **Oracles** | `src/oracles/AndeOracleAggregator.sol` | 🔄 Review |

---

## 📦 Deployment Scripts Structure

### Current Scripts (Need Cleanup)

```
contracts/script/
├── DeployANDETokenDuality.s.sol     ✅ Keep - Core token
├── DeployStaking.s.sol              ✅ Keep - Core staking
├── DeployGovernance.s.sol           ⚠️ Review - Complex
├── DeployEcosystem.s.sol            ❌ Delete - Uses wrong token
├── DeployProduction.s.sol           ❌ Delete - Has bugs
├── DeployProductionFixed.s.sol      ⚠️ Incomplete
├── FundStaking.s.sol                ✅ Keep - Utility
├── SaveAddresses.s.sol              ✅ Keep - Utility
└── TestANDEDuality.s.sol            ✅ Keep - Testing
```

### Proposed New Structure

```
contracts/script/
├── 00_DeployCore.s.sol              # TIER 1: Token + Staking + Sequencer
├── 01_DeployGovernance.s.sol        # TIER 2: Timelock + Governor
├── 02_DeployInfrastructure.s.sol    # TIER 3: Bridge, Vaults, etc.
├── 03_DeployDeFi.s.sol              # TIER 4: DEX, Lending
├── utils/
│   ├── FundStaking.s.sol
│   ├── VerifyContracts.s.sol
│   └── SaveAddresses.s.sol
└── test/
    ├── TestToken.s.sol
    ├── TestStaking.s.sol
    └── TestGovernance.s.sol
```

---

## 🚀 Deployment Strategy

### Phase 1: Core Launch (Week 1)
```bash
# Deploy TIER 1 only
make deploy-core

# Contracts:
- ANDETokenDuality
- AndeNativeStaking  
- AndeSequencerRegistry
```

**Success Criteria:**
- ✅ Token minting works
- ✅ Staking operational (3 pools)
- ✅ Sequencers can register
- ✅ Frontend connects successfully

---

### Phase 2: Governance (Week 2)
```bash
# Deploy TIER 2
make deploy-governance

# Contracts:
- AndeTimelockController
- AndeGovernor
```

**Success Criteria:**
- ✅ Proposals can be created
- ✅ Voting works
- ✅ Timelock executes
- ✅ Upgrades possible via governance

---

### Phase 3: Infrastructure (Week 3-4)
```bash
# Deploy TIER 3
make deploy-infrastructure

# Contracts:
- WAndeVault
- AndeFeeDistributor
- AndeVesting
- AndeChainBridge (optional)
```

---

### Phase 4: DeFi Ecosystem (Month 2+)
```bash
# Deploy TIER 4
make deploy-defi

# Contracts:
- AndeSwap (DEX)
- AndeLend (Lending)
- VotingEscrow + Gauges
```

---

## 🔒 Security Considerations

### Pre-Deployment Checklist

- [ ] **Smart Contract Audits**
  - [ ] TIER 1 contracts audited by reputable firm
  - [ ] TIER 2 contracts audited
  - [ ] All critical paths tested

- [ ] **Testing Coverage**
  - [ ] Unit tests: >95% coverage
  - [ ] Integration tests: All user flows
  - [ ] Fuzzing: Critical functions
  - [ ] Invariant tests: Core invariants

- [ ] **Access Control**
  - [ ] Multi-sig for admin roles
  - [ ] Timelock for all critical operations
  - [ ] Emergency pause mechanisms

- [ ] **Upgrade Strategy**
  - [ ] UUPS proxy pattern
  - [ ] Upgrade paths documented
  - [ ] Rollback procedures ready

---

## 📝 Deployment Commands

### Local Development
```bash
# Deploy core (TIER 1)
forge script script/00_DeployCore.s.sol:DeployCore \
  --rpc-url http://localhost:8545 \
  --broadcast \
  --private-key $PRIVATE_KEY

# Verify deployment
forge script script/utils/VerifyContracts.s.sol:VerifyCore \
  --rpc-url http://localhost:8545
```

### Testnet
```bash
# Deploy core with verification
forge script script/00_DeployCore.s.sol:DeployCore \
  --rpc-url $TESTNET_RPC \
  --broadcast \
  --verify \
  --etherscan-api-key $ETHERSCAN_KEY \
  --private-key $PRIVATE_KEY
```

### Mainnet
```bash
# Deploy core (requires multi-sig confirmation)
forge script script/00_DeployCore.s.sol:DeployCore \
  --rpc-url $MAINNET_RPC \
  --broadcast \
  --verify \
  --private-key $DEPLOYER_KEY \
  --slow  # Add delays between txs
```

---

## 🗂️ Contract Addresses Tracking

All deployed addresses should be saved in:
```
contracts/deployments/
├── local/
│   ├── core.json
│   ├── governance.json
│   └── infrastructure.json
├── testnet/
│   └── ...
└── mainnet/
    └── ...
```

**Format:**
```json
{
  "network": "andechain-local",
  "chainId": 1234,
  "deployer": "0x...",
  "timestamp": 1234567890,
  "contracts": {
    "ANDETokenDuality": {
      "proxy": "0x...",
      "implementation": "0x..."
    },
    "AndeNativeStaking": {
      "proxy": "0x...",
      "implementation": "0x..."
    }
  }
}
```

---

## ⚠️ Contracts to Archive/Remove

These contracts are obsolete or deprecated:

```
src/.archive/deprecated-*/
├── ANDEToken.sol           # Old token (use ANDETokenDuality)
├── AbobToken.sol           # Project pivot - not used
├── AuctionManager.sol      # Old ABOB system
├── CollateralManager.sol   # Old ABOB system
├── StakingVault.sol        # Replaced by AndeNativeStaking
└── DualTrackBurnEngine.sol # Old tokenomics
```

**Action**: Already archived in `.archive/` directory ✅

---

## 🎯 Immediate Action Items

### 1. Clean Up Scripts (Priority: HIGH)
- [ ] Create `00_DeployCore.s.sol` with ONLY Tier 1 contracts
- [ ] Test deployment on fresh local node
- [ ] Verify all contracts deploy successfully
- [ ] Document deployment process

### 2. Update Makefile (Priority: HIGH)
- [ ] Add `make deploy-core` command
- [ ] Add `make deploy-governance` command
- [ ] Remove references to obsolete scripts
- [ ] Add verification commands

### 3. Testing (Priority: CRITICAL)
- [ ] Write integration tests for core deployment
- [ ] Test staking reward distribution
- [ ] Test sequencer registration flow
- [ ] Test emergency pause mechanisms

### 4. Documentation (Priority: MEDIUM)
- [ ] Document each contract's purpose
- [ ] Create deployment runbooks
- [ ] Document upgrade procedures
- [ ] Create troubleshooting guide

---

## 📚 Resources

### Contract Documentation
- ANDETokenDuality: See `src/ANDETokenDuality.sol`
- AndeNativeStaking: See `src/staking/AndeNativeStaking.sol`
- Governance: See `docs/governance.md`

### External Dependencies
- OpenZeppelin Contracts: v5.1.0
- Foundry: Latest stable
- Solidity: 0.8.25

---

## 🔄 Version History

| Version | Date | Changes |
|---------|------|---------|
| 1.0.0 | 2024-XX-XX | Initial production classification |

---

**Next Review Date**: Before mainnet launch  
**Owner**: Ande Labs Core Team