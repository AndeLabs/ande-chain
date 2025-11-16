//! ANDE Executor - Feature Configuration
//!
//! This module provides configuration for ANDE Chain custom features:
//! - Token Duality Precompile (0xFD)
//! - Parallel EVM Execution (Block-STM)
//! - MEV Detection System
//!
//! Note: In production v1.0.0, we use EthereumNode directly.
//! Custom EVM configuration will be added in future versions via hooks.

use ande_evm::evm_config::AndePrecompileProvider;
use revm::primitives::hardfork::SpecId;
use tracing::info;

/// ANDE Chain feature configuration
#[derive(Debug, Clone)]
pub struct AndeConfig {
    spec_id: SpecId,
    parallel_enabled: bool,
    mev_detection_enabled: bool,
}

impl Default for AndeConfig {
    fn default() -> Self {
        Self::new()
    }
}

impl AndeConfig {
    /// Create new ANDE configuration from environment
    pub fn new() -> Self {
        // Get spec ID from environment or default to CANCUN
        let spec_id = std::env::var("ANDE_SPEC_ID")
            .ok()
            .and_then(|s| match s.to_uppercase().as_str() {
                "SHANGHAI" => Some(SpecId::SHANGHAI),
                "CANCUN" => Some(SpecId::CANCUN),
                "PRAGUE" => Some(SpecId::PRAGUE),
                _ => None,
            })
            .unwrap_or(SpecId::CANCUN);

        let parallel_enabled = std::env::var("ANDE_ENABLE_PARALLEL_EVM")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(true);

        let mev_detection_enabled = std::env::var("ANDE_ENABLE_MEV_DETECTION")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(true);

        info!(
            spec = ?spec_id,
            parallel = parallel_enabled,
            mev_detection = mev_detection_enabled,
            "Initializing ANDE Chain configuration"
        );

        Self {
            spec_id,
            parallel_enabled,
            mev_detection_enabled,
        }
    }

    /// Get the spec ID
    pub const fn spec_id(&self) -> SpecId {
        self.spec_id
    }

    /// Check if parallel execution is enabled
    pub const fn is_parallel_enabled(&self) -> bool {
        self.parallel_enabled
    }

    /// Check if MEV detection is enabled
    pub const fn is_mev_detection_enabled(&self) -> bool {
        self.mev_detection_enabled
    }

    /// Create precompile provider
    pub fn create_precompile_provider(&self) -> AndePrecompileProvider {
        info!(
            spec = ?self.spec_id,
            "Creating ANDE precompile provider with Token Duality at 0xFD"
        );
        AndePrecompileProvider::new(self.spec_id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use reth_chainspec::MAINNET;

    #[test]
    fn test_ande_evm_config_creation() {
        let config = AndeEvmConfig::new(MAINNET.clone());
        assert_eq!(config.spec_id(), SpecId::CANCUN);
        assert!(config.is_parallel_enabled());
        assert!(config.is_mev_detection_enabled());
    }

    #[test]
    fn test_precompile_provider_creation() {
        let config = AndeEvmConfig::new(MAINNET.clone());
        let provider = config.create_precompile_provider();
        assert_eq!(provider.spec_id(), SpecId::CANCUN);
    }
}
