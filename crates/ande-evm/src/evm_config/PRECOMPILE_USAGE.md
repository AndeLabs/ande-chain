# ANDE Token Duality Precompile - Usage Guide

## Overview

The ANDE Token Duality Precompile enables ANDE tokens to function as both:
1. **Native gas token** - for paying transaction fees
2. **ERC-20 token** - for DeFi applications

This guide explains how to configure and use the enhanced security features.

## Architecture

The precompile system consists of three main components:

1. **`precompile.rs`** - Core precompile logic for token transfers
2. **`precompile_config.rs`** - Configuration with security parameters
3. **`precompile_inspector.rs`** - Runtime validation with full EVM context access

## Security Features

### 1. Allow-List Validation

Only addresses in the allow-list can call the precompile:

```rust
use evolve::evm_config::{AndePrecompileConfig, AndePrecompileInspector};

let mut config = AndePrecompileConfig::default();

// Add ANDEToken contract to allow-list
config.add_to_allow_list(ande_token_address);

// Add other authorized addresses (e.g., governance)
config.add_to_allow_list(governance_address);

let inspector = AndePrecompileInspector::new(config);
```

### 2. Per-Call Transfer Caps

Limit the maximum amount that can be transferred in a single transaction:

```rust
use alloy_primitives::U256;

let mut config = AndePrecompileConfig::default();

// Set max 1M ANDE per call (18 decimals)
config.per_call_cap = U256::from(1_000_000u64) * U256::from(10u64).pow(U256::from(18));
```

### 3. Per-Block Transfer Caps

Limit the total amount that can be transferred within a single block:

```rust
// Set max 10M ANDE per block
config.per_block_cap = Some(U256::from(10_000_000u64) * U256::from(10u64).pow(U256::from(18)));

// Disable block cap
config.per_block_cap = None;
```

## Configuration Methods

### Method 1: Environment Variables

Set configuration via environment variables:

```bash
# Precompile address (default: 0x00..fd)
export ANDE_PRECOMPILE_ADDRESS="0x00000000000000000000000000000000000000fd"

# ANDEToken contract address (automatically added to allow-list)
export ANDE_TOKEN_ADDRESS="0x1234567890123456789012345678901234567890"

# Additional authorized addresses (comma-separated)
export ANDE_ALLOW_LIST="0xabcd...,0xef01..."

# Per-call cap in wei (1M ANDE with 18 decimals)
export ANDE_PER_CALL_CAP="1000000000000000000000000"

# Per-block cap in wei (10M ANDE with 18 decimals)
export ANDE_PER_BLOCK_CAP="10000000000000000000000000"

# Enable strict validation (true/false)
export ANDE_STRICT_VALIDATION="true"
```

Then create the config:

```rust
let config = AndePrecompileConfig::from_env()?;
let inspector = AndePrecompileInspector::new(config);
```

### Method 2: Programmatic Configuration

```rust
use evolve::evm_config::{AndePrecompileConfig, AndePrecompileInspector};
use alloy_primitives::{Address, U256};
use std::collections::HashSet;

let mut allow_list = HashSet::new();
allow_list.insert(ande_token_address);
allow_list.insert(governance_address);

let config = AndePrecompileConfig {
    precompile_address: ANDE_PRECOMPILE_ADDRESS,
    ande_token_address,
    allow_list,
    per_call_cap: U256::from(1_000_000u64) * U256::from(10u64).pow(U256::from(18)),
    per_block_cap: Some(U256::from(10_000_000u64) * U256::from(10u64).pow(U256::from(18))),
    strict_validation: true,
};

let inspector = AndePrecompileInspector::new(config);
```

### Method 3: Testing Configuration

For tests, use relaxed validation:

```rust
#[cfg(test)]
let config = AndePrecompileConfig::for_testing();
```

This disables strict validation and removes caps.

## Integration with EVM

### Adding the Inspector to the EVM

The inspector should be added to the EVM during initialization:

```rust
use revm::{Evm, inspector::InspectorStack};

let mut inspector_stack = InspectorStack::new();

// Add ANDE precompile inspector
let ande_inspector = AndePrecompileInspector::from_env()?;
inspector_stack.push(ande_inspector);

// Build EVM with inspector
let mut evm = Evm::builder()
    .with_inspector(inspector_stack)
    .build();
```

### Precompile Input Format

The precompile expects exactly 96 bytes of input:

```
Bytes 0-31:   from address (32 bytes, address in last 20 bytes)
Bytes 32-63:  to address (32 bytes, address in last 20 bytes)  
Bytes 64-95:  value amount (32 bytes, uint256)
```

Example Solidity call:

```solidity
// In ANDEToken contract
function transfer(address to, uint256 value) external returns (bool) {
    bytes memory input = abi.encode(msg.sender, to, value);
    
    (bool success, ) = ANDE_PRECOMPILE_ADDRESS.call(input);
    require(success, "Precompile call failed");
    
    return true;
}
```

## Error Handling

The inspector will revert with descriptive messages:

- `"Unauthorized caller: 0x..."` - Caller not in allow-list
- `"Invalid input length: X (expected 96)"` - Wrong input size
- `"Transfer to zero address"` - Attempted transfer to 0x0
- `"Transfer amount X exceeds per-call cap Y"` - Exceeds per-call limit
- `"Total block transfers X would exceed per-block cap Y"` - Exceeds block limit

## Security Best Practices

1. **Always use allow-list** - Never disable authorization in production
2. **Set reasonable caps** - Prevent accidental large transfers
3. **Monitor block transfers** - Track usage patterns
4. **Regular audits** - Review allow-list and caps periodically
5. **Test thoroughly** - Use testing config for comprehensive tests

## Example: Complete Setup

```rust
use evolve::evm_config::{
    AndePrecompileConfig, 
    AndePrecompileInspector,
    ANDE_PRECOMPILE_ADDRESS,
};
use alloy_primitives::{Address, U256};

// 1. Create configuration
let mut config = AndePrecompileConfig::default();

// 2. Set ANDEToken address
let ande_token = Address::from_str("0x1234...")?;
config.ande_token_address = ande_token;
config.add_to_allow_list(ande_token);

// 3. Configure security limits
config.per_call_cap = U256::from(1_000_000u64) * U256::from(10u64).pow(U256::from(18));
config.per_block_cap = Some(U256::from(10_000_000u64) * U256::from(10u64).pow(U256::from(18)));

// 4. Create inspector
let inspector = AndePrecompileInspector::new(config);

// 5. Add to EVM (implementation-specific)
// evm.with_inspector(inspector);
```

## Monitoring and Debugging

### Enable Logging

```rust
// Log all precompile calls
tracing::info!(
    target: "ande_precompile",
    caller = ?inputs.caller,
    value = ?value,
    "Precompile called"
);
```

### Track Block Transfers

The inspector automatically tracks transfers per block and resets on new blocks.

### Testing

Run tests with:

```bash
cargo test -p evolve --lib evm_config::precompile
cargo test -p evolve --lib evm_config::precompile_config
cargo test -p evolve --lib evm_config::precompile_inspector
```

## Upgrading from Previous Version

If you're upgrading from the basic precompile implementation:

1. **Install new modules** - Add `precompile_config` and `precompile_inspector`
2. **Create configuration** - Use `AndePrecompileConfig::from_env()`
3. **Add inspector to EVM** - Integrate `AndePrecompileInspector`
4. **Set environment variables** - Configure allow-list and caps
5. **Test thoroughly** - Verify all security features work

The core `precompile.rs` remains backward compatible - no changes needed to existing precompile logic.
