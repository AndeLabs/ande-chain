# ANDE Chain - Smart Contracts Documentation

> Consolidated documentation for ANDE Chain smart contracts

Last Updated: 2025-11-16

---

## 📋 Overview

ANDE Chain smart contracts implement:
- **Consensus System**: Validator management and block attestation
- **Token Duality**: Native ANDE token as ERC-20
- **Account Abstraction**: ERC-4337 compatible smart accounts
- **DeFi Primitives**: Staking, rewards, and treasury management

---

## 🏗️ Contract Architecture

### Core Contracts

```
contracts/src/
├── consensus/              # Consensus & validator management
│   ├── AndeBFTConsensus.sol       - Main consensus contract
│   ├── ValidatorSetManager.sol    - Validator registration & management
│   └── BlockAttestation.sol       - Block attestation system
│
├── token/                  # Token contracts
│   ├── AndeToken.sol              - ERC-20 token (for reference)
│   └── TokenDualityBridge.sol     - Bridge to native token
│
├── account/                # Account abstraction
│   ├── AndeSmartAccount.sol       - ERC-4337 smart account
│   └── AndeAccountFactory.sol     - Account factory
│
└── staking/               # Staking & rewards
    ├── AndeStaking.sol            - Staking management
    └── RewardsDistributor.sol     - Reward distribution
```

---

## 📚 Detailed Documentation

### Consensus Contracts

**Location**: `contracts/src/consensus/`

**Main Contract**: `AndeBFTConsensus.sol`
- Validator set management
- Block proposal and attestation
- Slashing for misbehavior
- Epoch transitions

**Key Functions**:
```solidity
function registerValidator(bytes32 blsPublicKey, uint256 stake) external
function attestBlock(uint256 blockNumber, bytes32 blockHash, bytes signature) external
function proposeBlock(uint256 blockNumber, bytes32 blockHash) external
function slashValidator(address validator, string memory reason) external
```

**Security Features**:
- ✅ Minimum stake requirement
- ✅ BLS signature verification
- ✅ Slashing for double-signing
- ✅ Upgradeable via proxy pattern

---

### Token Duality

**Concept**: Native ANDE token accessible as ERC-20

**Implementation**:
- **Native Layer**: Token Duality Precompile at `0xFD`
- **Contract Layer**: `TokenDualityBridge.sol` for contract interactions

**How It Works**:
```
User Balance
    ↓
Native (wei) ←→ ERC-20 (tokens)
    ↑              ↑
Precompile      Bridge Contract
```

**Usage**:
```solidity
// Transfer using ERC-20 interface
IERC20(ANDE_TOKEN).transfer(recipient, amount);

// Balance automatically synced with native balance
```

---

### Account Abstraction

**Location**: `contracts/src/account/`

**ERC-4337 Compatible**: Full support for account abstraction

**Features**:
- Custom validation logic
- Multi-sig capabilities
- Social recovery
- Gas sponsorship

**Deployment**:
```solidity
AndeAccountFactory factory = new AndeAccountFactory();
address account = factory.createAccount(owner, salt);
```

---

### Staking System

**Location**: `contracts/src/staking/`

**Key Features**:
- Validator staking
- Delegated staking
- Reward distribution
- Unbonding period

**Stake Flow**:
```
1. User stakes ANDE → AndeSt aking.stake()
2. Validators earn rewards
3. Rewards distributed → RewardsDistributor
4. Users can unstake after unbonding period
```

---

## 🔒 Security

### Audits Completed

**Token Duality Precompile** (2025-11-15):
- ✅ 0 Critical vulnerabilities
- ✅ 0 High vulnerabilities  
- ✅ Minor improvements implemented
- 📄 Report: `docs/SECURITY_AUDIT_PRECOMPILE.md`

**Smart Contracts**:
- ✅ Slither analysis passed
- ✅ OpenZeppelin best practices
- ✅ Upgradeable via UUPS pattern
- 📄 Report: `contracts/SECURITY_AUDIT_REPORT.md`

### Security Features

- **Access Control**: Role-based permissions
- **Upgradeability**: UUPS proxy pattern
- **Reentrancy Protection**: ReentrancyGuard
- **Input Validation**: Comprehensive checks
- **Emergency Stop**: Pausable functionality

---

## 🚀 Deployment

### Current Deployments

**Testnet** (Chain ID: 6174):
```
AndeBFTConsensus:        0x5FC8d32690cc91D4c39d9d3abcBD16989F875707
AndeStaking:             0xa513E6E4b8f2a923D98304ec87F64353C4D5C853
TokenDualityBridge:      0x...
AndeAccountFactory:      0x...
```

**Deployment Guide**: See `contracts/DEPLOY_MANUAL.md`

### Deploy New Contracts

```bash
# Setup
cd contracts
forge install

# Deploy to testnet
forge script script/DeployAll.s.sol \
  --rpc-url $RPC_URL \
  --private-key $PRIVATE_KEY \
  --broadcast

# Verify on explorer
forge verify-contract <address> <contract> \
  --chain-id 6174 \
  --etherscan-api-key $API_KEY
```

---

## 🧪 Testing

### Run Tests

```bash
cd contracts

# All tests
forge test

# Specific test file
forge test --match-path test/consensus/AndeBFTConsensus.t.sol

# With gas report
forge test --gas-report

# With coverage
forge coverage
```

### Test Structure

```
test/
├── consensus/          # Consensus tests
├── token/             # Token tests
├── account/           # Account abstraction tests
├── staking/           # Staking tests
└── integration/       # Integration tests
```

---

## 📖 Additional Resources

### Contract Docs

- **Architecture**: `contracts/src/ARCHITECTURE.md`
- **Recent Changes**: `contracts/src/RECENT_CHANGES.md`
- **Deployment Manual**: `contracts/DEPLOY_MANUAL.md`
- **Security Audit**: `contracts/SECURITY_AUDIT_REPORT.md`

### External Resources

- [OpenZeppelin Contracts](https://docs.openzeppelin.com/contracts/)
- [ERC-4337 Spec](https://eips.ethereum.org/EIPS/eip-4337)
- [Foundry Book](https://book.getfoundry.sh/)

---

## 🔧 Development

### Setup

```bash
cd contracts
forge install
forge build
```

### Adding New Contract

1. Create contract in `src/`
2. Add test in `test/`
3. Update deployment script
4. Document in this file

### Best Practices

- Use OpenZeppelin when possible
- Follow Solidity style guide
- Write comprehensive tests (>90% coverage)
- Document all public functions
- Use NatSpec comments

---

## 📊 Contract Stats

| Category | Contracts | Test Coverage |
|----------|-----------|---------------|
| Consensus | 3 | 95% |
| Token | 2 | 100% |
| Account | 2 | 92% |
| Staking | 2 | 98% |
| **Total** | **9** | **96%** |

---

## 📝 License

MIT License - See LICENSE file for details

---

**Maintained by**: ANDE Labs  
**Last Updated**: 2025-11-16  
**Version**: 1.0.0
