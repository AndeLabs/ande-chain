//! ANDE Reth - Production Ethereum Client with Token Duality
//!
//! This crate implements a full Reth node with ANDE-specific components:
//! - Custom EVM configuration with native precompiles (AndeEvmConfig)
//! - Token Duality pattern (ANDE as native currency + ERC-20)
//! - Production-ready, modular architecture

#![warn(missing_docs, unreachable_pub, unused_crate_dependencies)]
#![deny(unused_must_use, rust_2018_idioms)]

/// Node-specific components and configuration
pub mod node;

/// EVM configuration with ANDE precompiles
pub mod executor;

/// Re-export main node type
pub use node::AndeNode;

/// Re-export EVM config
pub use executor::AndeEvmConfig;
