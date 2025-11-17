//! Evolve-specific types and integration
//!
//! This crate provides Evolve-specific functionality including:
//! - Custom payload attributes for Evolve
//! - Evolve-specific types and traits
//! - Custom consensus implementation
//! - Custom EVM configuration with ANDE precompiles

/// Evolve-specific types and related definitions.
pub mod types;

/// Configuration for Evolve functionality.
pub mod config;

/// RPC modules for Evolve functionality.
pub mod rpc;

/// Custom consensus implementation for Evolve.
pub mod consensus;

/// Consensus configuration module.
pub mod consensus_config;

/// Custom EVM configuration with Evolve-specific precompiles.
pub mod evm_config;

/// Parallel EVM execution module.
pub mod parallel;

/// Optimized parallel executor for Block-STM
pub mod parallel_executor;

/// MEV detection and integration module.
pub mod mev;

#[cfg(test)]
mod tests;

// Re-export public types
pub use config::{EvolveConfig, DEFAULT_MAX_TXPOOL_BYTES, DEFAULT_MAX_TXPOOL_GAS};
pub use consensus::{EvolveConsensus, EvolveConsensusBuilder};
pub use evm_config::{
    ande_token_duality_precompile, 
    AndeEvmFactory,
    AndePrecompileProvider,
    ANDE_PRECOMPILE_ADDRESS
};
pub use parallel_executor::{ParallelExecutor, TxExecutionResult, optimal_worker_count};
pub use types::{EvolvePayloadAttributes, PayloadAttributesError};
pub use mev::{AndeHandler, AndeMevRedirect, MevDetection, MevRedirectError, MevType};
