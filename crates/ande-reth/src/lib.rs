//! ANDE Reth - Custom Reth Fork for ANDE Chain
//!
//! This is NOT a wrapper around standard Reth/EthereumNode.
//! This is our CUSTOM FORK based on Reth v1.8.2 (git: 9c30bf7).
//!
//! ## Architecture
//!
//! ```text
//! AndeNode (Custom Node Type)
//!     ↓
//! AndeExecutorBuilder (Custom Executor)
//!     ↓
//! AndeEvmFactory (Custom EVM Factory)
//!     ↓
//! AndePrecompileProvider (Custom Precompiles)
//!     ↓
//! Token Duality Precompile @ 0xFD
//! ```
//!
//! ## Active Features ✅
//!
//! - **Token Duality Precompile (0xFD)**: Native ANDE token as ERC20
//! - **Custom EVM Execution**: AndeEvmFactory with full context access
//! - **Custom Executor**: AndeExecutorBuilder integrating ANDE features
//!
//! ## Planned Features ⏳
//!
//! - **Parallel EVM Execution**: Block-STM algorithm
//! - **MEV Detection**: Fair MEV distribution
//! - **Enhanced Consensus**: Custom validator selection
//!
//! ## Components
//!
//! - `AndeNode`: Custom node type (NOT EthereumNode)
//! - `AndeExecutorBuilder`: Custom executor with ANDE EVM
//! - Integration with `ande-evm` crate for precompiles

#![cfg_attr(not(test), warn(unused_crate_dependencies))]

/// ANDE custom node implementation
pub mod node;

/// ANDE custom executor builder
pub mod executor;

/// ANDE custom consensus builder
pub mod consensus;

/// Re-export main node type
pub use node::AndeNode;

/// Re-export executor builder
pub use executor::AndeExecutorBuilder;

/// Re-export consensus builder
pub use consensus::AndeConsensusBuilder;
