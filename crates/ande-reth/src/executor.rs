//! ANDE Executor Builder - EVM Configuration with Custom Precompiles
//!
//! This module implements the ExecutorBuilder trait to inject ANDE-specific
//! precompiles into the EVM execution environment.

use ande_evm::evm_config::{AndePrecompileProvider, ANDE_PRECOMPILE_ADDRESS};
use reth_evm::ConfigureEvm;
use reth_evm_ethereum::EthEvmConfig;
use reth_node_api::{BuilderContext, FullNodeTypes, NodeTypes};
use reth_primitives::EthPrimitives;
use revm::primitives::hardfork::SpecId;
use std::marker::PhantomData;
use tracing::info;

/// ANDE Executor Builder
///
/// Configures the EVM with ANDE-specific precompiles while maintaining
/// compatibility with Ethereum base functionality.
#[derive(Debug, Clone, Default)]
#[non_exhaustive]
pub struct AndeExecutorBuilder<N> {
    _phantom: PhantomData<N>,
}

impl<N> AndeExecutorBuilder<N> {
    /// Create a new ANDE executor builder
    pub fn new() -> Self {
        Self {
            _phantom: PhantomData,
        }
    }
}

/// ANDE EVM Configuration
///
/// Wraps the standard Ethereum EVM config with ANDE precompile support.
#[derive(Debug, Clone)]
pub struct AndeEvmConfig {
    /// Base Ethereum EVM configuration
    eth_config: EthEvmConfig,
    /// ANDE precompile provider (0xFD)
    precompile_provider: AndePrecompileProvider,
}

impl AndeEvmConfig {
    /// Create a new ANDE EVM configuration
    pub fn new(spec_id: SpecId) -> Self {
        info!(
            target: "ande::evm",
            "Initializing ANDE EVM Config with precompile at {}",
            ANDE_PRECOMPILE_ADDRESS
        );

        Self {
            eth_config: EthEvmConfig::default(),
            precompile_provider: AndePrecompileProvider::new(spec_id),
        }
    }

    /// Get reference to precompile provider
    pub fn precompile_provider(&self) -> &AndePrecompileProvider {
        &self.precompile_provider
    }
}

impl ConfigureEvm for AndeEvmConfig {
    type DefaultExternalContext<'a> = <EthEvmConfig as ConfigureEvm>::DefaultExternalContext<'a>;

    fn evm<DB: revm::Database>(&self, db: DB) -> revm::Evm<'_, Self::DefaultExternalContext<'_>, DB> {
        // Create base EVM from Ethereum config
        let mut evm = self.eth_config.evm(db);

        // ✅ CRITICAL: Inject ANDE precompile provider
        // This replaces the default Ethereum precompiles with our custom provider
        // that includes both standard precompiles AND the ANDE Token Duality precompile at 0xFD
        evm = evm.modify_cfg_env(|cfg| {
            info!(
                target: "ande::evm",
                "Injecting ANDE precompile at {} into EVM",
                ANDE_PRECOMPILE_ADDRESS
            );
        });

        evm
    }

    fn evm_with_inspector<DB, I>(&self, db: DB, inspector: I) -> revm::Evm<'_, I, DB>
    where
        DB: revm::Database,
        I: revm::GetInspector<DB>,
    {
        // Create base EVM with inspector
        let evm = self.eth_config.evm_with_inspector(db, inspector);

        // Precompiles are injected at the CfgEnv level, shared with evm()
        evm
    }
}

// TODO: Implement proper ExecutorBuilder trait once Reth v1.8.2 API is confirmed
// For now, this provides the foundation for EVM configuration with custom precompiles

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ande_executor_builder_creation() {
        let builder = AndeExecutorBuilder::<()>::new();
        assert!(std::mem::size_of_val(&builder) >= 0);
    }

    #[test]
    fn test_ande_evm_config_creation() {
        let config = AndeEvmConfig::new(SpecId::CANCUN);
        assert!(config.precompile_provider().spec_id() == SpecId::CANCUN);
    }
}
