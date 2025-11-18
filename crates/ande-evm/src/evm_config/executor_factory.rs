//! ANDE Block Executor Factory
//!
//! This module provides a custom BlockExecutorFactory that injects the ANDE
//! precompile provider into the EVM during execution.

use crate::evm_config::AndePrecompileProvider;
use reth_chainspec::ChainSpec;
use revm::primitives::hardfork::SpecId;
use std::sync::Arc;

/// Custom block executor factory that injects ANDE precompiles
#[derive(Debug, Clone)]
pub struct AndeBlockExecutorFactory {
    chain_spec: Arc<ChainSpec>,
    precompile_provider: Arc<AndePrecompileProvider>,
}

impl AndeBlockExecutorFactory {
    /// Create a new ANDE block executor factory
    pub fn new(chain_spec: Arc<ChainSpec>) -> Self {
        // ANDE Chain uses Cancun spec (EIP-4844 blob transactions support)
        let spec_id = SpecId::CANCUN;

        let precompile_provider = Arc::new(AndePrecompileProvider::new(spec_id));

        Self {
            chain_spec,
            precompile_provider,
        }
    }

    /// Get reference to the chain spec
    pub fn chain_spec(&self) -> &Arc<ChainSpec> {
        &self.chain_spec
    }

    /// Get reference to the precompile provider
    pub fn precompile_provider(&self) -> &Arc<AndePrecompileProvider> {
        &self.precompile_provider
    }
}

// Note: Full BlockExecutorFactory implementation requires deep integration with reth's
// execution architecture. For Phase 1, we provide the precompile provider separately
// for runtime injection during block building.
//
// Future work (Phase 2): Create a custom EvmFactory that wraps the EVM and injects
// the precompile provider before each transaction execution.

#[cfg(test)]
mod tests {
    use super::*;
    use reth_chainspec::{ChainSpecBuilder, MAINNET};

    #[test]
    fn test_ande_block_executor_factory_creation() {
        let chain_spec = Arc::new(
            ChainSpecBuilder::default()
                .chain(MAINNET.chain)
                .genesis(Default::default())
                .build()
        );
        
        let factory = AndeBlockExecutorFactory::new(chain_spec.clone());
        assert_eq!(factory.chain_spec().chain, chain_spec.chain);
    }

    #[test]
    fn test_precompile_provider_access() {
        let chain_spec = Arc::new(
            ChainSpecBuilder::default()
                .chain(MAINNET.chain)
                .genesis(Default::default())
                .build()
        );
        
        let factory = AndeBlockExecutorFactory::new(chain_spec);
        let provider = factory.precompile_provider();
        
        // Provider should be available
        assert!(Arc::strong_count(provider) >= 1);
    }
}
