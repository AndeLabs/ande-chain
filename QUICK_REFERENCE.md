# ANDE Chain - Quick Reference

> Cheat sheet for common commands and operations

---

## 🚀 Essential Commands

### Build & Run

```bash
# Build release
cargo build --release

# Run node
./target/release/ande-node --chain specs/genesis.json

# Build + Run in one command
cargo run --release -- --chain specs/genesis.json
```

### Development

```bash
# Fast compilation check
cargo check

# Run tests
cargo test

# Format code
cargo fmt

# Lint code
cargo clippy

# Clean build
cargo clean && cargo build --release
```

---

## 📂 Critical Files

### Where to Make Changes

| Task | File | Line/Function |
|------|------|---------------|
| Add precompile | `ande-evm/src/evm_config/ande_evm_factory.rs` | `create_evm()` ~75 |
| Change consensus | `ande-reth/src/consensus.rs` | `build_consensus()` ~40 |
| Modify executor | `ande-reth/src/executor.rs` | `build_evm()` ~53 |
| Update node config | `ande-reth/src/node.rs` | `components<N>()` ~70 |
| Chain parameters | `specs/genesis.json` | Root object |

### Import Statements Reference

```rust
// In executor.rs
use alloy_evm::EthEvmFactory;  // ← NOT from reth_ethereum::evm
use reth_ethereum::evm::revm::primitives::hardfork::SpecId;

// In consensus.rs
use reth_chainspec::EthChainSpec;  // ← NOT just ChainSpec
use reth_ethereum_consensus::EthBeaconConsensus;

// In node.rs
use reth_ethereum::node::builder::components::ComponentsBuilder;
```

---

## 🔧 Type Signatures Reference

### Executor Builder

```rust
impl<Types, Node> ExecutorBuilder<Node> for AndeExecutorBuilder
where
    Types: NodeTypes<
        ChainSpec: Hardforks + EthExecutorSpec + EthereumHardforks,
        Primitives = EthPrimitives,
    >,
    Node: FullNodeTypes<Types = Types>,
{
    type EVM = EthEvmConfig<Types::ChainSpec, AndeEvmFactory>;
    //                      ^^^^^^^^^^^^^^^^ Use Types::ChainSpec
}
```

### Consensus Builder

```rust
impl<Node> ConsensusBuilder<Node> for AndeConsensusBuilder
where
    Node: FullNodeTypes<
        Types: NodeTypes<
            ChainSpec: EthChainSpec + EthereumHardforks,
            Primitives = EthPrimitives,
        >,
    >,
{
    type Consensus = Arc<EthBeaconConsensus<<Node::Types as NodeTypes>::ChainSpec>>;
    //               ^^^ Don't forget Arc!
}
```

---

## 🐛 Common Error Fixes

### Error: "mismatched types" with ChainSpec

```rust
// ❌ Wrong
type EVM = EthEvmConfig<ChainSpec, AndeEvmFactory>;

// ✅ Correct
type EVM = EthEvmConfig<Types::ChainSpec, AndeEvmFactory>;
```

### Error: "trait bound not satisfied"

Check your where clause has all required traits:
```rust
where
    Node: FullNodeTypes<
        Types: NodeTypes<
            ChainSpec: EthChainSpec + EthereumHardforks,  // ← Both required
            Primitives = EthPrimitives,
        >,
    >,
```

### Error: "unresolved import"

Add module declaration in `main.rs`:
```rust
mod node;
mod executor;
mod consensus;  // ← Make sure this exists
```

### Error: "attempted to zero-initialize"

Use `Option` for optional values:
```rust
// ❌ Wrong
engine: Arc<RwLock<ConsensusEngine>>,

// ✅ Correct
engine: Option<Arc<RwLock<ConsensusEngine>>>,
```

---

## 📊 Project Status Check

```bash
# Check compilation
cargo check 2>&1 | grep -E '(error|warning)' | wc -l

# Count errors only
cargo check 2>&1 | grep '^error' | wc -l

# Test coverage
cargo test 2>&1 | grep -E 'test result'

# Binary size
ls -lh target/release/ande-node

# Dependencies tree
cargo tree -p ande-reth | head -20
```

---

## 🔍 Debugging Commands

```bash
# Full backtrace
RUST_BACKTRACE=full ./target/release/ande-node

# Debug logging
RUST_LOG=debug ./target/release/ande-node

# Trace specific module
RUST_LOG=ande_reth=trace ./target/release/ande-node

# Multiple modules
RUST_LOG=ande_reth=debug,ande_evm=trace ./target/release/ande-node

# Check for panics in code
grep -r "unwrap()" crates/ande-reth/src/

# Find unsafe blocks
grep -r "unsafe" crates/ande-reth/src/
```

---

## 📦 Dependency Management

```bash
# Update dependencies
cargo update

# Check for outdated deps
cargo outdated

# Audit security
cargo audit

# Show dependency tree
cargo tree

# Why is X included?
cargo tree -i <package-name>
```

---

## 🧪 Testing Shortcuts

```bash
# Run all tests
cargo test

# Run specific test
cargo test test_ande_evm_factory

# Run tests in specific file
cargo test --test integration_test

# Show test output
cargo test -- --nocapture

# Run ignored tests
cargo test -- --ignored

# Test specific crate
cargo test -p ande-evm
```

---

## 🎯 Git Workflow

```bash
# Create feature branch
git checkout -b feature/my-feature

# Commit with descriptive message
git commit -m "feat(executor): add custom precompile injection"

# Push branch
git push origin feature/my-feature

# Rebase on main
git fetch origin
git rebase origin/main

# Squash commits
git rebase -i HEAD~3
```

### Commit Message Format

```
<type>(<scope>): <subject>

<body>

<footer>
```

Types: `feat`, `fix`, `docs`, `style`, `refactor`, `test`, `chore`

Examples:
- `feat(evm): implement token duality precompile`
- `fix(consensus): resolve panic on disabled mode`
- `docs(readme): update quick start guide`

---

## 🚨 Emergency Fixes

### Build is Stuck

```bash
# Kill all cargo processes
pkill -9 cargo

# Clean and rebuild
cargo clean
cargo build --release
```

### Tests Failing

```bash
# Update test snapshots
cargo test -- --test-threads=1

# Run single test in isolation
cargo test test_name -- --exact --nocapture
```

### Can't Find Module

```bash
# Check module is declared
grep -r "mod consensus" crates/ande-reth/src/

# Check module is exported
grep -r "pub use.*Consensus" crates/ande-reth/src/
```

---

## 📈 Performance Check

```bash
# Compilation time
time cargo build --release

# Binary size
du -h target/release/ande-node

# Startup time
time timeout 5 ./target/release/ande-node || true

# Memory usage
ps aux | grep ande-node
```

---

## 🔐 Security Check

```bash
# Audit dependencies
cargo audit

# Check for common issues
cargo clippy -- -W clippy::all

# Find unsafe code
rg "unsafe" crates/

# Find unwrap calls
rg "\.unwrap\(\)" crates/
```

---

## 📝 Documentation

```bash
# Generate docs
cargo doc --open

# Generate docs for specific crate
cargo doc -p ande-reth --open

# Check doc comments
cargo rustdoc -- -D warnings
```

---

## 🎨 Code Quality

```bash
# Format all code
cargo fmt

# Check formatting
cargo fmt -- --check

# Run clippy
cargo clippy --all-targets --all-features

# Auto-fix clippy warnings
cargo clippy --fix
```

---

## 🌐 Network Testing

```bash
# Check if port is available
lsof -i :8545

# Test RPC endpoint
curl -X POST http://localhost:8545 \
  -H "Content-Type: application/json" \
  -d '{"jsonrpc":"2.0","method":"eth_chainId","params":[],"id":1}'

# Monitor network traffic
tcpdump -i any -n port 8545
```

---

## 💾 Database Operations

```bash
# Check database size
du -sh ~/.ande/db

# Backup database
tar -czf ande-db-backup.tar.gz ~/.ande/db

# Clear database
rm -rf ~/.ande/db
```

---

## 📋 Environment Setup

```bash
# Set Rust version
rustup default nightly-2024-10-18

# Add components
rustup component add rustfmt clippy

# Check Rust version
rustc --version
cargo --version

# Verify installation
cargo check
```

---

## 🔗 Useful Aliases

Add to `~/.bashrc` or `~/.zshrc`:

```bash
# ANDE Chain aliases
alias ab='cargo build --release'
alias at='cargo test'
alias ac='cargo check'
alias af='cargo fmt'
alias al='cargo clippy'
alias ar='./target/release/ande-node'
alias ard='RUST_LOG=debug ./target/release/ande-node'

# Quick rebuild
alias are='cargo clean && cargo build --release'

# Test shortcuts
alias atv='cargo test -- --nocapture'
alias ati='cargo test -- --ignored'
```

---

## 📞 Getting Help

1. **Check logs**: `RUST_LOG=trace cargo run`
2. **Read error**: Full error message with `--verbose`
3. **Search code**: `rg "pattern" crates/`
4. **Check docs**: `cargo doc --open`
5. **Ask on Discord**: [ANDE Community](https://discord.gg/ande)

---

**Last Updated**: 2025-11-16  
**Version**: 1.0.0

For detailed guides, see:
- [Custom Reth Implementation](docs/CUSTOM_RETH_IMPLEMENTATION.md)
- [Development Guide](docs/DEVELOPMENT_GUIDE.md)
