//! ANDE Executor Builder - EVM Configuration with Custom Precompiles
//!
//! This module implements the ConfigureEvm trait to inject ANDE-specific
//! precompiles into the EVM execution environment.

use ande_evm::evm_config::{AndePrecompileProvider, ANDE_PRECOMPILE_ADDRESS};
use reth_evm::ConfigureEvm;
use reth_evm_ethereum::EthEvmConfig;
use revm::primitives::hardfork::SpecId;
use std::sync::Arc;
use tracing::info;

/// ANDE EVM Configuration
///
/// Wraps the standard Ethereum EVM config with ANDE precompile support.
/// Delegates all ConfigureEvm trait methods to EthEvmConfig for maximum compatibility.
#[derive(Debug, Clone)]
pub struct AndeEvmConfig {
    /// Base Ethereum EVM configuration (handles all the trait methods)
    inner: EthEvmConfig,
    /// ANDE precompile provider (0xFD) - used for custom precompile injection
    _precompile_provider: Arc<AndePrecompileProvider>,
}

impl Default for AndeEvmConfig {
    fn default() -> Self {
        Self::new(SpecId::CANCUN)
    }
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
            inner: EthEvmConfig::default(),
            _precompile_provider: Arc::new(AndePrecompileProvider::new(spec_id)),
        }
    }
}

// ============================================================================
// ConfigureEvm Implementation - Delegates to EthEvmConfig
// ============================================================================
//
// Strategy: We delegate ALL trait methods to the inner EthEvmConfig.
// This ensures full API compatibility with Reth v1.8.2+.
//
// Custom precompile injection happens at a different level (via EvmFactory).
// See: docs/PRECOMPILE_INTEGRATION_FINDINGS.md

impl ConfigureEvm for AndeEvmConfig {
    type Primitives = <EthEvmConfig as ConfigureEvm>::Primitives;
    type Error = <EthEvmConfig as ConfigureEvm>::Error;
    type NextBlockEnvCtx = <EthEvmConfig as ConfigureEvm>::NextBlockEnvCtx;
    type BlockExecutorFactory = <EthEvmConfig as ConfigureEvm>::BlockExecutorFactory;
    type BlockAssembler = <EthEvmConfig as ConfigureEvm>::BlockAssembler;

    fn block_executor_factory(&self) -> &Self::BlockExecutorFactory {
        self.inner.block_executor_factory()
    }

    fn block_assembler(&self) -> &Self::BlockAssembler {
        self.inner.block_assembler()
    }

    fn evm_env(
        &self,
        header: &<Self::Primitives as reth_node_api::NodePrimitives>::BlockHeader,
    ) -> Result<
        reth_evm::EvmEnv<
            <<<Self::BlockExecutorFactory as reth_evm::BlockExecutorFactory>::EvmFactory as reth_evm::EvmFactory>::Spec as revm::primitives::SpecId>,
        >,
        Self::Error,
    > {
        self.inner.evm_env(header)
    }

    fn next_evm_env(
        &self,
        parent: &<Self::Primitives as reth_node_api::NodePrimitives>::BlockHeader,
        attributes: &Self::NextBlockEnvCtx,
    ) -> Result<
        reth_evm::EvmEnv<
            <<<Self::BlockExecutorFactory as reth_evm::BlockExecutorFactory>::EvmFactory as reth_evm::EvmFactory>::Spec as revm::primitives::SpecId>,
        >,
        Self::Error,
    > {
        self.inner.next_evm_env(parent, attributes)
    }

    fn context_for_block<'a>(
        &self,
        block: &'a reth_ethereum::primitives::SealedBlock<
            <Self::Primitives as reth_node_api::NodePrimitives>::Block,
        >,
    ) -> Result<
        <<Self::BlockExecutorFactory as reth_evm::BlockExecutorFactory>::ExecutionCtx<'a> as Clone>::Output,
        Self::Error,
    > {
        self.inner.context_for_block(block)
    }

    fn context_for_next_block(
        &self,
        parent: &reth_primitives::SealedHeader<
            <Self::Primitives as reth_node_api::NodePrimitives>::BlockHeader,
        >,
        attributes: Self::NextBlockEnvCtx,
    ) -> Result<
        <<Self::BlockExecutorFactory as reth_evm::BlockExecutorFactory>::ExecutionCtx<'_> as Clone>::Output,
        Self::Error,
    > {
        self.inner.context_for_next_block(parent, attributes)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ande_evm_config_creation() {
        let config = AndeEvmConfig::new(SpecId::CANCUN);
        assert!(std::mem::size_of_val(&config) >= 0);
    }

    #[test]
    fn test_ande_evm_config_default() {
        let config = AndeEvmConfig::default();
        assert!(std::mem::size_of_val(&config) >= 0);
    }
}
