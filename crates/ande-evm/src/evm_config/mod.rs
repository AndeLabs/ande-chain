//! Evolve-specific EVM configuration with custom precompiles
//!
//! This module provides the ANDE Token Duality precompile that will be
//! injected into the EVM at runtime during block execution.

pub mod precompile;
pub mod precompile_config;
pub mod precompile_inspector;
pub mod ande_precompile_provider;
pub mod factory;
pub mod wrapper;
pub mod injection;
pub mod executor_factory;

#[cfg(test)]
mod integration_test;

#[cfg(test)]
mod e2e_test;

pub use precompile::{
    ande_token_duality_precompile, ANDE_PRECOMPILE_ADDRESS, ANDE_TOKEN_ADDRESS,
};
pub use precompile_config::AndePrecompileConfig;
pub use precompile_inspector::AndePrecompileInspector;
pub use ande_precompile_provider::AndePrecompileProvider;
pub use wrapper::AndeEvmConfig;
pub use factory::create_ande_evm_config;
pub use injection::{create_ande_precompile_provider, ande_precompile_address};
pub use executor_factory::AndeBlockExecutorFactory;
