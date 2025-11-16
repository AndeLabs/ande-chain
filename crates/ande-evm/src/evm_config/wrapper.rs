//! ANDE EVM Configuration
//!
//! For Phase 1, we use EthEvmConfig directly as our EVM configuration.
//! The ANDE precompile provider is available for future integration
//! at the payload builder or execution layer.
//!
//! Future Phase: Create a custom EvmFactory that wraps the EVM handler
//! and injects AndePrecompileProvider before transaction execution.

use crate::evm_config::AndePrecompileProvider;
use reth_chainspec::{ChainSpec, EthereumHardforks};
use reth_evm_ethereum::EthEvmConfig;
use revm::primitives::hardfork::SpecId;
use std::sync::Arc;
use tracing::info;

/// ANDE Chain EVM Configuration
///
/// Wraps EthEvmConfig and maintains a reference to our custom precompile provider.
/// The precompile provider can be accessed for injection at the payload builder level.
///
/// Note: For Phase 1, this is functionally equivalent to EthEvmConfig.
/// Phase 2 will implement custom EvmFactory for precompile injection.
#[derive(Debug, Clone)]
pub struct AndeEvmConfig {
    /// Inner Ethereum EVM config
    inner: EthEvmConfig,
    /// ANDE precompile provider (for future use)
    #[allow(dead_code)]
    precompile_provider: Arc<AndePrecompileProvider>,
}

impl AndeEvmConfig {
    /// Create a new ANDE EVM configuration
    pub fn new(chain_spec: Arc<ChainSpec>) -> Self {
        // Determine the spec ID from the chain spec
        let spec_id = Self::spec_id_from_chain(&chain_spec);

        info!(
            chain_id = chain_spec.chain.id(),
            ?spec_id,
            "✅ Initializing ANDE EVM config (Phase 1: using EthEvmConfig)"
        );
        info!("   📝 Precompile provider ready for Phase 2 integration");

        Self {
            inner: EthEvmConfig::new(chain_spec.clone()),
            precompile_provider: Arc::new(AndePrecompileProvider::new(spec_id)),
        }
    }

    /// Get the spec ID from chain spec hardforks
    fn spec_id_from_chain(chain_spec: &ChainSpec) -> SpecId {
        // Check for latest hardforks first
        if chain_spec.is_prague_active_at_timestamp(0) {
            SpecId::PRAGUE
        } else if chain_spec.is_cancun_active_at_timestamp(0) {
            SpecId::CANCUN
        } else if chain_spec.is_shanghai_active_at_timestamp(0) {
            SpecId::SHANGHAI
        } else {
            // Default to Cancun
            SpecId::CANCUN
        }
    }

    /// Get reference to the precompile provider
    pub fn precompile_provider(&self) -> &Arc<AndePrecompileProvider> {
        &self.precompile_provider
    }

    /// Get reference to inner EthEvmConfig
    pub fn inner(&self) -> &EthEvmConfig {
        &self.inner
    }
}

// Delegate all ConfigureEvm methods to inner EthEvmConfig
impl std::ops::Deref for AndeEvmConfig {
    type Target = EthEvmConfig;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use reth_chainspec::{ChainSpecBuilder, MAINNET};

    #[test]
    fn test_ande_evm_config_creation() {
        let chain_spec = Arc::new(
            ChainSpecBuilder::default()
                .chain(MAINNET.chain)
                .genesis(Default::default())
                .cancun_activated()
                .build(),
        );

        let config = AndeEvmConfig::new(chain_spec.clone());
        assert_eq!(config.inner().chain_spec().chain.id(), chain_spec.chain.id());
    }

    #[test]
    fn test_spec_id_detection() {
        let chain_spec = Arc::new(
            ChainSpecBuilder::default()
                .chain(MAINNET.chain)
                .genesis(Default::default())
                .cancun_activated()
                .build(),
        );

        let spec_id = AndeEvmConfig::spec_id_from_chain(&chain_spec);
        assert_eq!(spec_id, SpecId::CANCUN);
    }

    #[test]
    fn test_precompile_provider_access() {
        let chain_spec = Arc::new(
            ChainSpecBuilder::default()
                .chain(MAINNET.chain)
                .genesis(Default::default())
                .cancun_activated()
                .build(),
        );

        let config = AndeEvmConfig::new(chain_spec);
        let provider = config.precompile_provider();

        assert_eq!(provider.spec_id(), SpecId::CANCUN);
    }
}
