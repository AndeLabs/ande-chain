//! ANDE EVM Configuration Factory
//!
//! This module provides factory functions for creating AndeEvmConfig instances.

use alloy_primitives::Address;
use reth_chainspec::ChainSpec;
use std::sync::Arc;

use super::wrapper::AndeEvmConfig;
use crate::evm_config::ANDE_PRECOMPILE_ADDRESS;

pub fn create_ande_evm_config(chain_spec: Arc<ChainSpec>) -> AndeEvmConfig {
    AndeEvmConfig::new(chain_spec)
}

pub const fn ande_precompile_address() -> Address {
    ANDE_PRECOMPILE_ADDRESS
}

#[cfg(test)]
mod tests {
    use super::*;
    use reth_chainspec::{ChainSpecBuilder, MAINNET};

    #[test]
    fn test_ande_precompile_address() {
        assert_eq!(ande_precompile_address(), ANDE_PRECOMPILE_ADDRESS);
    }

    #[test]
    fn test_ande_evm_config_creation() {
        let chain_spec = Arc::new(
            ChainSpecBuilder::default()
                .chain(MAINNET.chain)
                .genesis(Default::default())
                .cancun_activated()
                .build()
        );
        let config = create_ande_evm_config(chain_spec.clone());
        assert_eq!(config.inner().chain_spec().chain.id(), chain_spec.chain.id());
    }
}
