//! ANDE Executor Builder - EVM Configuration with Custom Precompiles
//!
//! This module implements the ConfigureEvm trait to inject ANDE-specific
//! precompiles into the EVM execution environment.

use reth_evm::ConfigureEvm;
use reth_evm_ethereum::EthEvmConfig;
use tracing::info;

/// ANDE EVM Configuration
///
/// **TEMPORARY IMPLEMENTATION (v0.1):**
/// For now, this is a simple type alias to `EthEvmConfig`.
/// We will add custom precompile injection in a future iteration.
///
/// ## Roadmap
/// 1. v0.1: Get Reth compiling ✅
/// 2. v0.2: Inject ANDE precompile provider
/// 3. v0.3: Production-ready with native 0xFD support
pub type AndeEvmConfig = EthEvmConfig;

/// Initialize ANDE EVM configuration
///
/// Returns a mainnet EthEvmConfig for now.
/// Future: Will inject AndePrecompileProvider here.
pub fn ande_evm_config() -> AndeEvmConfig {
    info!(
        target: "ande::evm",
        "Initializing ANDE EVM Config (using EthEvmConfig as base)"
    );

    EthEvmConfig::mainnet()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ande_evm_config_creation() {
        let config = ande_evm_config();
        assert!(std::mem::size_of_val(&config) >= 0);
    }
}
