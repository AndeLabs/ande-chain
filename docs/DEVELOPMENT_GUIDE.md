# ANDE Chain Development Guide

> Quick reference for developers working on ANDE Chain

---

## 🚀 Quick Setup

### 1. Install Rust Nightly

```bash
rustup toolchain install nightly-2024-10-18
rustup default nightly-2024-10-18
rustup component add rustfmt clippy
```

### 2. Clone and Build

```bash
git clone https://github.com/AndeLabs/ande-chain.git
cd ande-chain
cargo build --release
```

### 3. Run Node

```bash
./target/release/ande-node --chain specs/genesis.json
```

---

## 📂 Project Structure

### Core Crates

```
ande-chain/crates/
├── ande-reth/         ⭐ CUSTOM RETH IMPLEMENTATION
│   ├── node.rs              # AndeNode type
│   ├── executor.rs          # AndeExecutorBuilder  
│   ├── consensus.rs         # AndeConsensusBuilder
│   └── main.rs              # Binary entry
│
├── ande-evm/          ⭐ EVM CUSTOMIZATIONS
│   ├── evm_config/
│   │   ├── ande_evm_factory.rs          # Wrapper factory
│   │   ├── ande_token_duality_precompile.rs
│   │   └── ande_precompile_provider.rs
│   ├── parallel_executor.rs
│   └── mev_detector.rs
│
├── ande-consensus/    # Consensus contracts client
├── ande-primitives/   # Shared types
└── ande-node/        # Old binary (to be deprecated)
```

### Key Files Reference

| File | Purpose | When to Edit |
|------|---------|--------------|
| `ande-reth/src/node.rs` | Node type definition | Adding new node features |
| `ande-reth/src/executor.rs` | EVM execution logic | Changing EVM behavior |
| `ande-reth/src/consensus.rs` | Consensus builder | Consensus changes |
| `ande-evm/src/evm_config/ande_evm_factory.rs` | EVM factory wrapper | Adding precompiles |
| `specs/genesis.json` | Chain genesis state | Network changes |

---

## 🔨 Common Development Tasks

### Building

```bash
# Fast debug build
cargo build

# Optimized release build  
cargo build --release

# Check types without building
cargo check

# Build specific crate
cargo build -p ande-reth
cargo build -p ande-evm
```

### Testing

```bash
# Run all tests
cargo test

# Run tests for specific crate
cargo test -p ande-evm

# Run specific test
cargo test test_token_duality_precompile

# Run with output
cargo test -- --nocapture

# Run ignored tests (e.g., integration tests)
cargo test -- --ignored
```

### Code Quality

```bash
# Linting
cargo clippy --all-targets --all-features

# Auto-fix lints
cargo clippy --fix

# Format code
cargo fmt

# Check format without changing
cargo fmt -- --check
```

### Debugging

```bash
# Enable debug logs
RUST_LOG=debug cargo run

# Full backtrace on panic
RUST_BACKTRACE=full cargo run

# Specific module logging
RUST_LOG=ande_reth=trace cargo run

# Multiple modules
RUST_LOG=ande_reth=debug,ande_evm=trace cargo run
```

---

## 🧩 Adding New Features

### Adding a New Precompile

**File**: `crates/ande-evm/src/evm_config/ande_evm_factory.rs`

1. Create precompile function:
```rust
// In new file: my_precompile.rs
pub fn my_precompile_handler(
    input: &[u8],
    gas_limit: u64,
    context: &EvmContext,
) -> PrecompileResult {
    // Your logic here
    Ok(PrecompileOutput {
        gas_used: 100,
        bytes: output,
    })
}
```

2. Register in factory:
```rust
// In ande_evm_factory.rs, create_evm method
fn create_evm<DB: Database>(&self, db: DB, input: EvmEnv) -> Self::Evm<DB, NoOpInspector> {
    let mut precompiles = PrecompilesMap::new();
    
    // Add your precompile
    precompiles.insert(
        MY_PRECOMPILE_ADDRESS,
        my_precompile_handler,
    );
    
    // Create EVM with custom precompiles
    EthEvm::new(db, input, precompiles)
}
```

### Modifying Consensus Logic

**File**: `crates/ande-reth/src/consensus.rs`

Change the consensus implementation:
```rust
async fn build_consensus(
    self,
    ctx: &BuilderContext<Node>,
) -> eyre::Result<Self::Consensus> {
    // Option 1: Use custom consensus
    Ok(Arc::new(AndeCustomConsensus::new(ctx.chain_spec())))
    
    // Option 2: Wrap standard consensus with custom logic
    let base = EthBeaconConsensus::new(ctx.chain_spec());
    Ok(Arc::new(AndeConsensusWrapper::new(base)))
}
```

### Adding Custom RPC Methods

**File**: `crates/ande-reth/src/rpc.rs` (create if doesn't exist)

1. Define RPC trait:
```rust
#[rpc(server)]
pub trait AndeApi {
    #[method(name = "ande_getTokenDualityInfo")]
    async fn get_token_duality_info(&self) -> RpcResult<TokenDualityInfo>;
}
```

2. Implement trait:
```rust
impl AndeApiServer for AndeRpc {
    async fn get_token_duality_info(&self) -> RpcResult<TokenDualityInfo> {
        // Your implementation
        Ok(TokenDualityInfo { /* ... */ })
    }
}
```

3. Register in node components.

---

## 🔍 Debugging Common Issues

### Issue: Compilation Errors with ChainSpec

**Symptom**: 
```
error[E0308]: mismatched types
expected `EthEvmConfig<ChainSpec, ...>`
found `EthEvmConfig<<Types as NodeTypes>::ChainSpec, ...>`
```

**Solution**: Use the type-parameterized ChainSpec:
```rust
// ❌ Wrong
type EVM = EthEvmConfig<ChainSpec, AndeEvmFactory>;

// ✅ Correct
type EVM = EthEvmConfig<Types::ChainSpec, AndeEvmFactory>;
```

### Issue: Trait Bound Not Satisfied

**Symptom**:
```
error[E0277]: the trait bound `X: Trait` is not satisfied
```

**Solution**: Check your where clauses:
```rust
impl<Node> ConsensusBuilder<Node> for AndeConsensusBuilder
where
    Node: FullNodeTypes<
        Types: NodeTypes<
            ChainSpec: EthChainSpec + EthereumHardforks,  // ← These bounds
            Primitives = EthPrimitives,
        >,
    >,
```

### Issue: Module Not Found

**Symptom**:
```
error[E0432]: unresolved import `crate::consensus`
```

**Solution**: Declare module in `main.rs`:
```rust
mod node;
mod executor;
mod consensus;  // ← Add this
```

### Issue: Runtime Panic

**Symptom**: Binary compiles but panics on startup.

**Debug steps**:
```bash
# 1. Run with backtrace
RUST_BACKTRACE=full ./target/release/ande-node

# 2. Check for unsafe code
grep -r "unsafe" crates/ande-reth/src/

# 3. Look for Option::unwrap() that might fail
grep -r "unwrap()" crates/ande-reth/src/

# 4. Run with debug logging
RUST_LOG=trace ./target/release/ande-node
```

---

## 📊 Performance Profiling

### Benchmarking

```bash
# Run benchmarks
cargo bench

# Specific benchmark
cargo bench --bench evm_execution
```

### Profiling with perf

```bash
# Build with debug symbols
CARGO_PROFILE_RELEASE_DEBUG=true cargo build --release

# Run with perf
perf record --call-graph dwarf ./target/release/ande-node

# View results
perf report
```

### Memory Profiling

```bash
# Using valgrind
valgrind --tool=massif ./target/release/ande-node

# Using heaptrack
heaptrack ./target/release/ande-node
```

---

## 🧪 Testing Strategy

### Unit Tests

Test individual components:
```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ande_executor_creation() {
        let builder = AndeExecutorBuilder::default();
        assert!(/* your assertion */);
    }
}
```

### Integration Tests

**File**: `tests/integration_test.rs`

```rust
#[tokio::test]
async fn test_full_node_startup() {
    // Setup
    let node = AndeNode::new();
    
    // Execute
    let result = node.start().await;
    
    // Assert
    assert!(result.is_ok());
}
```

### Property Tests

Using proptest:
```rust
use proptest::prelude::*;

proptest! {
    #[test]
    fn test_token_duality_invariants(amount in 0u64..1_000_000) {
        // Property: transfer should preserve total supply
        let before = total_supply();
        transfer(amount);
        let after = total_supply();
        assert_eq!(before, after);
    }
}
```

---

## 📝 Code Style Guide

### Naming Conventions

```rust
// Types: PascalCase
struct AndeEvmFactory { }
enum ConsensusState { }

// Functions: snake_case
fn build_consensus() { }
fn create_evm() { }

// Constants: SCREAMING_SNAKE_CASE
const ANDE_PRECOMPILE_ADDRESS: Address = address!("00...FD");
const MAX_GAS_LIMIT: u64 = 30_000_000;

// Modules: snake_case
mod ande_evm_factory;
mod token_duality_precompile;
```

### Documentation

All public items must have doc comments:
```rust
/// Builds the ANDE EVM with custom precompiles.
///
/// # Arguments
///
/// * `ctx` - Builder context containing chain spec and config
///
/// # Returns
///
/// Returns `EthEvmConfig` with ANDE customizations
///
/// # Errors
///
/// Returns error if EVM initialization fails
async fn build_evm(&self, ctx: &BuilderContext<Node>) -> eyre::Result<Self::EVM> {
    // Implementation
}
```

### Error Handling

```rust
// ✅ Use Result for fallible operations
fn parse_config() -> eyre::Result<Config> {
    let data = std::fs::read_to_string("config.toml")?;
    toml::from_str(&data).map_err(Into::into)
}

// ✅ Use proper error context
fn load_genesis() -> eyre::Result<Genesis> {
    std::fs::read_to_string("genesis.json")
        .wrap_err("Failed to read genesis file")?;
}

// ❌ Don't use unwrap() in production code
let value = map.get("key").unwrap();  // BAD

// ✅ Handle errors properly
let value = map.get("key").ok_or_else(|| eyre!("Key not found"))?;  // GOOD
```

---

## 🔐 Security Guidelines

### Code Review Checklist

Before submitting PR:
- [ ] No `unsafe` code without justification
- [ ] No `.unwrap()` in production paths
- [ ] All inputs validated
- [ ] Gas limits enforced
- [ ] Overflow checks for arithmetic
- [ ] No hardcoded secrets
- [ ] Proper error handling
- [ ] Tests added for new features

### Security-Sensitive Areas

Pay extra attention to:
1. **Precompile implementations** - Direct EVM access
2. **Consensus logic** - Network security
3. **RPC handlers** - External API surface
4. **Balance transfers** - Token duality logic

### Running Security Audit

```bash
# Cargo audit (dependency vulnerabilities)
cargo install cargo-audit
cargo audit

# Clippy security lints
cargo clippy -- -W clippy::all -W clippy::pedantic

# Check for unsafe code
cargo geiger
```

---

## 📚 Further Reading

### Essential Documentation

- [Custom Reth Implementation Guide](CUSTOM_RETH_IMPLEMENTATION.md) - Complete architecture
- [Token Duality Security Audit](SECURITY_AUDIT_PRECOMPILE.md) - Security review
- [Reth Book](https://paradigmxyz.github.io/reth/) - Official Reth docs

### Helpful Resources

- Rust async programming: https://rust-lang.github.io/async-book/
- Revm internals: https://github.com/bluealloy/revm
- Alloy types: https://github.com/alloy-rs/alloy

---

## 🤝 Getting Help

### Before Asking

1. Check this guide
2. Read `docs/CUSTOM_RETH_IMPLEMENTATION.md`
3. Search GitHub issues
4. Check Reth documentation

### Where to Ask

- **GitHub Issues**: Bug reports and feature requests
- **Discord**: General questions and discussion  
- **Code Review**: Pull request comments

---

**Happy coding! 🚀**

Last updated: 2025-11-16
