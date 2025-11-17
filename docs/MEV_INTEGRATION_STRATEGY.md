# MEV Integration Strategy for ANDE Chain

## Executive Summary

This document outlines ANDE Chain's MEV (Maximal Extractable Value) integration strategy, comparing different implementation approaches and documenting our final decision.

## Background

MEV refers to the profit that can be extracted from transaction ordering and inclusion. ANDE Chain aims to redistribute MEV profits fairly:
- 80% to ANDE stakers
- 20% to protocol treasury

## Implementation Approaches Analyzed

### Approach 1: EVM Handler Wrapper (evstack pattern)

**Pattern**: Wrap the entire EVM with a custom handler that intercepts `reward_beneficiary()`

**Pros**:
- Maximum control over execution
- Can intercept every transaction
- Follows established pattern (evstack)

**Cons**:
- High complexity (requires deep revm internals knowledge)
- Tight coupling with revm implementation details
- Difficult to maintain across reth/revm upgrades
- Requires custom `EvEvm` wrapper type
- Type system complexity (generics, trait bounds)

**Verdict**: ❌ Too complex for ANDE's needs, hard to maintain

### Approach 2: Smart Contract Distribution (Recommended)

**Pattern**: Redirect block rewards/base fees to a MEV distribution smart contract

**Pros**:
- ✅ Clean separation of concerns
- ✅ Easy to audit and verify
- ✅ No coupling with reth/revm internals  
- ✅ On-chain transparency
- ✅ Flexible distribution logic (can be upgraded)
- ✅ Standard pattern used by major chains (Optimism, Arbitrum)

**Cons**:
- Requires deploying distribution contract
- Slightly delayed distribution (next block)

**Verdict**: ✅ **SELECTED APPROACH**

### Approach 3: Post-Execution Hook

**Pattern**: Intercept after block execution, modify state before finalization

**Pros**:
- Simpler than full EVM wrapper
- Can redistribute in same block

**Cons**:
- Still requires deep executor integration
- State modification after execution is risky
- Hard to audit (off-chain logic)

**Verdict**: ❌ Complexity without benefits

## Selected Implementation: Smart Contract Distribution

### Architecture

```
┌─────────────────┐
│   New Block     │
│  (Coinbase fee) │
└────────┬────────┘
         │
         ▼
┌─────────────────────────┐
│ MEV Distribution        │
│ Contract (0x...)        │
│                         │
│ - Accumulates base fees │
│ - Tracks contributions  │
│ - Enables withdrawals   │
└──────────┬──────────────┘
           │
     ┌─────┴─────┐
     │           │
     ▼           ▼
┌─────────┐ ┌────────────┐
│ Stakers │ │  Treasury  │
│  (80%)  │ │   (20%)    │
└─────────┘ └────────────┘
```

### Configuration

MEV distribution is configured via chainspec and environment variables:

```toml
# genesis.json
{
  "config": {
    "mevDistribution": {
      "enabled": true,
      "contractAddress": "0x...",
      "stakersShare": 80,  // percentage
      "treasuryShare": 20  // percentage
    }
  }
}
```

```bash
# Environment variables (optional override)
export ANDE_MEV_ENABLED=true
export ANDE_MEV_SINK=0x0000000000000000000000000000000000000042
export ANDE_MEV_MIN_THRESHOLD=1000000000000000  # 0.001 ETH
```

### Smart Contract Interface

```solidity
// SPDX-License-Identifier: MIT
pragma solidity ^0.8.0;

interface IAndeMevDistribution {
    /// @notice Receive MEV profits from block production
    receive() external payable;
    
    /// @notice Distribute accumulated MEV to stakers and treasury
    function distribute() external;
    
    /// @notice Withdraw MEV rewards for a staker
    /// @param staker Address of the staker
    function withdraw(address staker) external;
    
    /// @notice Get pending MEV rewards for a staker
    /// @param staker Address of the staker
    /// @return amount Pending reward amount
    function pendingRewards(address staker) external view returns (uint256 amount);
    
    /// @notice Total MEV accumulated this epoch
    function totalMev() external view returns (uint256);
}
```

### Node Integration

The node simply sets the coinbase/fee_recipient to the MEV distribution contract:

```rust
// In executor/block building
let mev_sink = chain_spec.mev_distribution_contract();
block_env.coinbase = mev_sink;  // All fees go to contract
```

### Distribution Logic (Smart Contract)

```solidity
contract AndeMevDistribution {
    uint256 public constant STAKERS_SHARE = 80;
    uint256 public constant TREASURY_SHARE = 20;
    
    address public treasury;
    mapping(address => uint256) public stakerWeights;
    mapping(address => uint256) public pendingRewards;
    
    receive() external payable {
        // Accumulate MEV from block rewards
        emit MevReceived(msg.value, block.number);
    }
    
    function distribute() external {
        uint256 totalMev = address(this).balance;
        
        uint256 stakersAmount = (totalMev * STAKERS_SHARE) / 100;
        uint256 treasuryAmount = (totalMev * TREASURY_SHARE) / 100;
        
        // Distribute to treasury
        payable(treasury).transfer(treasuryAmount);
        
        // Calculate per-staker rewards based on weight
        uint256 totalWeight = getTotalStakerWeight();
        for (address staker : stakers) {
            uint256 share = (stakersAmount * stakerWeights[staker]) / totalWeight;
            pendingRewards[staker] += share;
        }
        
        emit MevDistributed(stakersAmount, treasuryAmount);
    }
}
```

## Implementation Phases

### Phase 1: Infrastructure Setup ✅ COMPLETED
- ✅ Created `AndeMevRedirect` detection structure
- ✅ Created `MevConfig` for environment configuration
- ✅ Created `AndeHandler` (kept for future flexibility)
- ✅ Documented MEV infrastructure

### Phase 2: Smart Contract Development (CURRENT)
- [ ] Write `AndeMevDistribution.sol` contract
- [ ] Comprehensive testing (unit + integration)
- [ ] Gas optimization
- [ ] Security audit
- [ ] Deploy to testnet

### Phase 3: Node Integration
- [ ] Update genesis.json with MEV contract address
- [ ] Configure block building to use MEV contract as coinbase
- [ ] Add MEV metrics/logging
- [ ] Document operator configuration

### Phase 4: Production Deployment
- [ ] Deploy MEV contract to mainnet
- [ ] Update chain spec
- [ ] Coordinate with validators
- [ ] Monitor MEV distribution

## Code Organization

```
ande-chain/
├── crates/
│   ├── ande-evm/
│   │   └── src/
│   │       └── mev/
│   │           ├── redirect.rs      # MEV detection (kept for flexibility)
│   │           ├── handler.rs       # Handler wrapper (kept for flexibility)
│   │           └── config.rs        # Environment configuration
│   └── ande-contracts/             # NEW: Smart contracts
│       └── src/
│           ├── AndeMevDistribution.sol
│           └── test/
│               └── MevDistribution.t.sol
└── docs/
    ├── MEV_INTEGRATION_STRATEGY.md  # This document
    └── MEV_HANDLER_ANALYSIS.md      # Technical analysis
```

## Benefits of This Approach

1. **Simplicity**: No deep executor integration needed
2. **Transparency**: All MEV distribution logic is on-chain and auditable
3. **Flexibility**: Distribution logic can be upgraded without node changes
4. **Standard**: Follows industry best practices (Optimism, Arbitrum, etc.)
5. **Security**: Smart contract security is well-understood domain
6. **Maintainability**: No coupling with reth/revm internals

## Migration Path

Existing MEV infrastructure (AndeHandler, AndeMevRedirect) is kept but not actively used. This provides:
- Option to add additional MEV detection in the future
- Research capabilities for MEV analysis
- Flexibility if requirements change

## References

- Optimism MEV approach: https://community.optimism.io/docs/protocol/sequencing/
- Arbitrum sequencer revenue: https://docs.arbitrum.io/dao-constitution
- MEV-Boost: https://boost.flashbots.net/
- evstack MEV implementation: `ev-reth-official/crates/ev-revm/`

## Conclusion

ANDE Chain will implement MEV redistribution via smart contract, following industry best practices. This approach provides the optimal balance of:
- Implementation simplicity
- On-chain transparency  
- Security auditability
- Operational flexibility

The existing MEV detection infrastructure remains available for future enhancements or research purposes.
