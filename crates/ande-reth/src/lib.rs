//! ANDE Reth - Custom Reth Node Library
//!
//! Production-ready Ethereum client for ANDE Chain.
//!
//! ## Current Implementation (Phase 1)
//!
//! Uses standard Ethereum components for stability:
//! - EthereumNode as base
//! - Standard EVM execution
//! - Standard transaction pool
//! - Compatible with Evolve sequencer
//!
//! ## Custom Features (Ready for Phase 2)
//!
//! All ANDE-specific features are implemented in `ande-evm` crate:
//! - Token Duality Precompile (0xFD)
//! - Parallel EVM Execution (Block-STM)
//! - MEV Detection and Protection
//!
//! ## Main Components
//!
//! - `AndeNode`: Node type (delegates to EthereumNode)
//! - `AndeConfig`: Feature configuration

#![cfg_attr(not(test), warn(unused_crate_dependencies))]

/// Node implementation
pub mod node;

/// Executor configuration
pub mod executor;

/// Re-export main node type
pub use node::AndeNode;

/// Re-export executor config
pub use executor::AndeConfig;
