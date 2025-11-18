//! ANDE EVM Configuration with Custom Precompiles
//!
//! This module provides the ANDE Token Duality precompile that enables
//! ANDE tokens to function as both native gas and ERC-20-like tokens.

pub mod precompile_config;
pub mod precompile_inspector;
pub mod ande_precompile_provider;
pub mod ande_evm_factory;
pub mod ande_token_duality;
pub mod factory;
pub mod wrapper;
pub mod injection;
pub mod executor_factory;

#[cfg(test)]
mod integration_test;

#[cfg(test)]
mod e2e_test;

// Primary exports from ande_token_duality (the production implementation)
pub use ande_token_duality::{
    AndeTokenDualityPrecompile,
    AndePrecompileConfig as TokenDualityConfig,
    ANDE_PRECOMPILE_ADDRESS,
    ANDE_TOKEN_ADDRESS,
};

// Security and configuration
pub use precompile_config::AndeInspectorConfig;
pub use precompile_inspector::AndePrecompileInspector;

// EVM integration
pub use ande_precompile_provider::AndePrecompileProvider;
pub use ande_evm_factory::AndeEvmFactory;
pub use wrapper::AndeEvmConfig;
pub use factory::create_ande_evm_config;
pub use injection::{create_ande_precompile_provider, ande_precompile_address};
pub use executor_factory::AndeBlockExecutorFactory;
