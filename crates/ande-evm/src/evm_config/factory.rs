//! ANDE EVM Configuration Factory
//!
//! This module provides factory functions for creating AndeEvmConfig instances.

use alloy_primitives::Address;
use reth_chainspec::ChainSpec;
use reth_evm_ethereum::EthEvmConfig;
use std::sync::Arc;

use super::wrapper::AndeEvmConfig;
use crate::evm_config::ANDE_PRECOMPILE_ADDRESS;

pub fn create_ande_evm_config(chain_spec: Arc<ChainSpec>) -> AndeEvmConfig {
    EthEvmConfig::new(chain_spec)
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
                .build()
        );
        let config = create_ande_evm_config(chain_spec);
        assert_eq!(config.chain_spec().chain, MAINNET.chain);
    }
}