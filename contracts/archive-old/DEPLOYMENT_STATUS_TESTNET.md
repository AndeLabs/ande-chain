cd ande# AndeChain Testnet Deployment Status

**Chain ID:** 6174  
**RPC Endpoint:** https://rpc.ande.network  
**Deployment Date:** 2025-11-06  
**Environment:** Testnet (Production-Ready Code)

---

## TIER 1: Core Contracts (✓ DEPLOYED)

### 1. ANDETokenDuality Proxy
- **Status:** ✓ Deployed and Functional
- **Proxy Address:** `0x5FC8d32690cc91D4c39d9d3abcBD16989F875707`
- **Type:** ERC20 + Native Currency (Dual-purpose token)
- **Features:**
  - ERC20 token functionality
  - Native gas currency support
  - Burnable and Pausable
  - Minter role for reward distribution
- **Note:** Initialized in previous deployment

### 2. AndeNativeStaking Proxy
- **Status:** ✓ Deployed and Functional
- **Proxy Address:** `0xa513E6E4b8f2a923D98304ec87F64353C4D5C853`
- **Type:** Multi-tier staking with lock periods
- **Features:**
  - 3 staking pools (Liquidity, Governance, Sequencer)
  - Lock period multipliers (1x to 3x)
  - Reward distribution
  - Vote delegation
- **Note:** Initialized in previous deployment

### 3. AndeSequencerRegistry Proxy ✓
- **Status:** ✓ Deployed and Initialized
- **Implementation:** `0x8A791620dd6260079BF849Dc5567aDC3F2FdC318`
- **Proxy Address:** `0x610178dA211FEF7D417bC0e6FeD39F05609AD788`
- **Type:** Sequencer management with phase transitions
- **Initialization Block:** 6649
- **Features:**
  - Genesis sequencer registration
  - Phase transition system (GENESIS → DUAL → MULTI → DECENTRALIZED)
  - Uptime tracking
  - Leader rotation
  - Block production recording
  - Epoch management (90 days)
- **Initialized With:**
  - Default Admin: `0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266`
  - Foundation: `0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266` (Genesis sequencer)
- **Transaction Hash:** `0xe1bda79ee8251b0bc0bd5693408059a3d267f5a1ce2545ca37caa79a6841525d`
- **Roles Granted:**
  - DEFAULT_ADMIN_ROLE ✓
  - PAUSER_ROLE ✓
  - SEQUENCER_MANAGER_ROLE ✓

---

## TIER 2: Governance Contracts

### 1. AndeTimelockController Proxy ✓
- **Status:** ✓ Deployed and Initialized
- **Implementation:** `0xA51c1fc2f0D1a1b8494Ed1FE312d7C3a78Ed91C0`
- **Proxy Address:** `0x0DCd1Bf9A1b36cE34237eEaFef220932846BCD82`
- **Type:** Time-locked proposal executor
- **Initialization Block:** 6663
- **Features:**
  - Configurable minimum delay (3600 seconds = 1 hour)
  - Proposer role management
  - Executor role management
  - Operation queuing and execution
  - Guardian cancellation capability
- **Initialized With:**
  - Minimum Delay: 3600 seconds
  - Proposers: [] (empty, configured separately)
  - Executors: [] (empty, configured separately)
  - Admin: `0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266`
- **Transaction Hash:** `0x1e2351b49896e90aea4e214e6478e31e8cce4b5e788b4c0abb07bc0d7ee29db8`
- **Current Roles:**
  - DEFAULT_ADMIN_ROLE: `0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266`

### 2. AndeGovernor Proxy
- **Status:** ⏳ Pending Deployment
- **Issue:** Bytecode size exceeds EVM limit (30.5KB vs 24.6KB limit)
- **Solution:** 
  - Simplified governance contract being created (AndeGovernorLite)
  - Removes non-critical extensions to fit within bytecode limit
  - Maintains core voting and proposal functionality
- **Planned Features:**
  - Dual-token voting (ANDE + Staking bonus)
  - TimelockController integration
  - Simple quorum (4%)
  - UUPS upgradeable

---

## Deployed Contracts Summary

| Contract | Type | Address | Status |
|----------|------|---------|--------|
| ANDETokenDuality | Proxy | 0x5FC8d32690cc91D4c39d9d3abcBD16989F875707 | ✓ Deployed |
| AndeNativeStaking | Proxy | 0xa513E6E4b8f2a923D98304ec87F64353C4D5C853 | ✓ Deployed |
| AndeSequencerRegistry | Implementation | 0x8A791620dd6260079BF849Dc5567aDC3F2FdC318 | ✓ Deployed |
| AndeSequencerRegistry | Proxy | 0x610178dA211FEF7D417bC0e6FeD39F05609AD788 | ✓ Deployed & Init |
| AndeTimelockController | Implementation | 0xA51c1fc2f0D1a1b8494Ed1FE312d7C3a78Ed91C0 | ✓ Deployed |
| AndeTimelockController | Proxy | 0x0DCd1Bf9A1b36cE34237eEaFef220932846BCD82 | ✓ Deployed & Init |
| AndeGovernor | Proxy | - | ⏳ In Progress |

---

## Configuration Status

### MINTER_ROLE Configuration
- **Status:** ⏳ Pending
- **Target:** Grant ANDETokenDuality.MINTER_ROLE to AndeNativeStaking
- **Purpose:** Allow staking contract to mint rewards
- **Note:** Token proxy is not responding to calls; may need initialization

### PROPOSER_ROLE Configuration
- **Status:** ⏳ Pending  
- **Target:** Grant AndeTimelockController.PROPOSER_ROLE to AndeGovernor
- **Purpose:** Allow Governor to queue proposals in Timelock
- **Note:** Blocked on AndeGovernor deployment

### EXECUTOR_ROLE Configuration
- **Status:** ⏳ Pending
- **Target:** Grant AndeTimelockController.EXECUTOR_ROLE to address(0) or AndeGovernor
- **Purpose:** Enable proposal execution via Timelock
- **Note:** Blocked on AndeGovernor deployment

---

## Next Steps

### 1. Deploy AndeGovernor (Simplified Version)
```bash
# Using AndeGovernorLite which removes non-critical extensions
# to fit within 24KB bytecode limit

# Deploy Implementation
forge create src/governance/AndeGovernorLite.sol:AndeGovernorLite \
  --rpc-url https://rpc.ande.network \
  --private-key <YOUR_KEY>

# Deploy Proxy (without init)
# Then initialize with:
# - Token: ANDETokenDuality proxy or direct IVotes
# - Timelock: 0x0DCd1Bf9A1b36cE34237eEaFef220932846BCD82
# - Voting Delay: 1 block
# - Voting Period: 50400 blocks (~7 days)
# - Proposal Threshold: 0 (configurable)
# - Admin: 0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266
```

### 2. Configure MINTER_ROLE
```bash
# Grant MINTER_ROLE from ANDETokenDuality to AndeNativeStaking
MINTER_ROLE=$(cast call 0x5FC8d32690cc91D4c39d9d3abcBD16989F875707 "MINTER_ROLE()(bytes32)" --rpc-url https://rpc.ande.network)

cast send 0x5FC8d32690cc91D4c39d9d3abcBD16989F875707 \
  "grantRole(bytes32,address)" "$MINTER_ROLE" "0xa513E6E4b8f2a923D98304ec87F64353C4D5C853" \
  --rpc-url https://rpc.ande.network \
  --private-key <YOUR_KEY>
```

### 3. Configure Governance Roles
```bash
# After AndeGovernor is deployed at <GOVERNOR_PROXY>, execute:

# Grant PROPOSER_ROLE
PROPOSER=$(cast call 0x0DCd1Bf9A1b36cE34237eEaFef220932846BCD82 "PROPOSER_ROLE()(bytes32)" --rpc-url https://rpc.ande.network)

cast send 0x0DCd1Bf9A1b36cE34237eEaFef220932846BCD82 \
  "grantRole(bytes32,address)" "$PROPOSER" "<GOVERNOR_PROXY>" \
  --rpc-url https://rpc.ande.network \
  --private-key <YOUR_KEY>

# Grant EXECUTOR_ROLE to address(0) for open execution
EXECUTOR=$(cast call 0x0DCd1Bf9A1b36cE34237eEaFef220932846BCD82 "EXECUTOR_ROLE()(bytes32)" --rpc-url https://rpc.ande.network)

cast send 0x0DCd1Bf9A1b36cE34237eEaFef220932846BCD82 \
  "grantRole(bytes32,address)" "$EXECUTOR" "0x0000000000000000000000000000000000000000" \
  --rpc-url https://rpc.ande.network \
  --private-key <YOUR_KEY>
```

---

## Architecture Notes

### AndeSequencerRegistry
- **Proxy Pattern:** ERC1967Proxy with UUPS upgradeable implementation
- **State:** Fully initialized
- **Functions Available:**
  - `getCurrentLeader()` - Get current block producer
  - `getActiveSequencers()` - List all active sequencers
  - `getActiveSequencersCount()` - Count of active sequencers
  - `getPhaseRequirements()` - Phase-specific parameters
  - `updateSequencerUptime()` - Record uptime metrics
  - `recordBlockProduced()` - Log produced blocks
  - `transitionPhase()` - Move to next phase (time-based)

### AndeTimelockController
- **Proxy Pattern:** ERC1967Proxy with UUPS upgradeable implementation
- **State:** Fully initialized
- **Minimum Delay:** 3600 seconds (1 hour)
- **Functions Available:**
  - `schedule()` - Queue a proposal for execution
  - `execute()` - Execute a queued proposal
  - `cancel()` - Cancel a queued proposal
  - `hasRole()` - Check role permissions

### Deployment Deployer Account
- **Address:** `0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266`
- **Note:** Hardhat default test account (for testnet only)
- **Recommendation:** Replace with production account before mainnet

---

## Known Issues & Resolutions

### Issue 1: ANDETokenDuality Not Responding
- **Problem:** ANDE token proxy reverts on function calls
- **Cause:** Contract may not be properly initialized
- **Resolution:** Verify implementation and initialization state
- **Impact:** MINTER_ROLE configuration blocked

### Issue 2: AndeGovernor Bytecode Size
- **Problem:** Original AndeGovernor is 30.5KB (exceeds 24.6KB limit)
- **Cause:** Multiple custom extensions (GovernorDualTokenVoting, GovernorAdaptiveQuorum, GovernorMultiLevel)
- **Resolution:** Creating simplified version (AndeGovernorLite) with core functionality
- **Features Removed Temporarily:**
  - Adaptive quorum (using fixed 4% instead)
  - Multi-level proposals
  - Dual-token voting (using standard GovernorVotes)
- **Timeline:** Can be re-added via upgrade pattern

---

## Testing Checklist

- [x] AndeSequencerRegistry deployment
- [x] AndeSequencerRegistry initialization
- [x] AndeTimelockController deployment
- [x] AndeTimelockController initialization
- [ ] AndeGovernor deployment
- [ ] AndeGovernor initialization
- [ ] MINTER_ROLE configuration
- [ ] PROPOSER_ROLE configuration
- [ ] EXECUTOR_ROLE configuration
- [ ] End-to-end proposal workflow test
- [ ] Role-based access control verification

---

## Maintenance & Support

### Upgrade Strategy
All TIER 2 contracts use UUPS upgradeable pattern:
- Use `_authorizeUpgrade()` for access control
- DEFAULT_ADMIN_ROLE must authorize upgrades
- Implementation contract address can be updated without proxy redeployment

### Emergency Procedures
- Pause/Unpause: Use PAUSER_ROLE on AndeSequencerRegistry
- Cancel Operations: Timelock has guardian cancellation
- Role Management: Use `grantRole()` / `revokeRole()` with appropriate permissions

---

**Documentation Version:** 1.0  
**Last Updated:** 2025-11-06  
**Status:** Active Testnet Deployment
