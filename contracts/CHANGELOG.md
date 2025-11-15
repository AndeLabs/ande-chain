# Changelog - AndeChain Smart Contracts

All notable changes to the AndeChain smart contract system.

## [1.1.0] - 2025-10-14 - DeFi Primitives Complete

### Added - Major DeFi Ecosystem Expansion

#### 🏦 AndeLend Protocol
- **AndeLend.sol**: Lending and borrowing protocol inspired by Aave/Compound
  - Collateralized lending with 80% LTV
  - Dynamic interest rate model (2% base, scales to 60% at high utilization)
  - Health factor system for liquidation (threshold: 1.0)
  - Liquidation engine with 5% bonus for liquidators
  - Multi-asset support with individual markets
- **AToken.sol**: Interest-bearing ERC20 tokens
  - 1:1 exchange with underlying initially
  - Accrues interest through protocol
  - Fully transferable and composable

#### 🌾 AndeYieldVault (ERC4626)
- **AndeYieldVault.sol**: Auto-compound yield optimization vaults
  - ERC4626 standard compliance for maximum composability
  - Auto-harvest and reinvest ANDE rewards
  - Stake LP tokens in gauges, compound rewards back to LP
  - Performance fee: 10% (adjustable, max 20%)
  - Withdrawal fee: 0.1% (adjustable, max 1%)
  - Real-time APY calculation
  - Gas-efficient batch operations

#### 🚀 AndeLaunchpad
- **AndeLaunchpad.sol**: IDO (Initial DEX Offering) platform
  - Tiered allocation system via ANDE staking:
    - Bronze: 100 ANDE → 1x allocation
    - Silver: 500 ANDE → 5x allocation
    - Gold: 1,000 ANDE → 15x allocation
    - Platinum: 5,000 ANDE → 50x allocation
  - Merkle tree whitelist for gas-efficient verification
  - Flexible vesting schedules (cliff + linear unlock)
  - TGE (Token Generation Event) initial unlock
  - Automatic liquidity addition (minimum 50%)
  - Soft cap / hard cap mechanism
  - Refund system if soft cap not met
  - Platform fee: 2% (adjustable)

#### 💧 AndeSwapV3 (Concentrated Liquidity)
- **AndeSwapV3Pool.sol**: Uniswap V3-style concentrated liquidity
  - Price range selection for LPs (tick-based)
  - Capital efficiency up to 4000x vs V2
  - Multiple fee tiers:
    - 0.05% for stablecoins
    - 0.3% for standard pairs
    - 1% for volatile/exotic pairs
  - Range orders (limit order functionality)
  - Active liquidity management
  - Built-in TWAP oracle
  - Position NFTs (unique ranges)

### Documentation
- Added **DEFI_PRIMITIVES.md**: Comprehensive guide to new DeFi stack
  - Architecture diagrams for each protocol
  - Usage examples and code snippets
  - Security considerations
  - Integration patterns
  - Comparison with competitors
  - Detailed math explanations
- Updated **CHANGELOG.md**: This file

### Infrastructure
- Created `/src/lending/` directory for lending protocol
- Created `/src/vaults/yield/` directory for yield vaults
- Created `/src/launchpad/` directory for IDO platform
- Created `/src/dex/v3/` directory for concentrated liquidity

## [Unreleased]

### Planned Features
- Gas optimization improvements
- Additional token templates
- Enhanced liquidity locking mechanisms
- Multi-signature support for factory
- Governance integration

---

**Full Changelog**: https://github.com/ande-labs/andechain/compare/v0.9.0...v1.0.0
