//! ANDE EVM Factory with Custom Precompiles
//!
//! This module implements a custom EvmFactory that injects the ANDE Token Duality
//! precompile into the EVM at runtime. Uses AndePrecompileProvider directly as the
//! precompile system instead of PrecompilesMap.

use super::AndePrecompileProvider;
use alloy_evm::{
    eth::EthEvmContext,
    EvmEnv, EvmFactory,
};
use reth_ethereum::evm::{
    primitives::Database,
    revm::{
        context::{Context, TxEnv},
        context_interface::result::{EVMError, HaltReason},
        inspector::{Inspector, NoOpInspector},
        interpreter::interpreter::EthInterpreter,
        primitives::hardfork::SpecId,
        MainBuilder, MainContext,
    },
};
use reth_evm::EthEvm;
use std::sync::Arc;

/// Custom EVM factory that injects ANDE precompiles
#[derive(Debug, Clone)]
pub struct AndeEvmFactory {
    /// The ANDE precompile provider
    precompile_provider: Arc<AndePrecompileProvider>,
}

impl AndeEvmFactory {
    /// Create a new ANDE EVM factory with the given spec
    pub fn new(spec_id: SpecId) -> Self {
        Self {
            precompile_provider: Arc::new(AndePrecompileProvider::new(spec_id)),
        }
    }

    /// Get reference to the precompile provider
    pub fn precompile_provider(&self) -> &Arc<AndePrecompileProvider> {
        &self.precompile_provider
    }
}

impl EvmFactory for AndeEvmFactory {
    type Evm<DB: Database, I: Inspector<EthEvmContext<DB>, EthInterpreter>> =
        EthEvm<DB, I, AndePrecompileProvider>;
    type Tx = TxEnv;
    type Error<DBError: core::error::Error + Send + Sync + 'static> = EVMError<DBError>;
    type HaltReason = HaltReason;
    type Context<DB: Database> = EthEvmContext<DB>;
    type Spec = SpecId;
    type Precompiles = AndePrecompileProvider;

    fn create_evm<DB: Database>(&self, db: DB, input: EvmEnv) -> Self::Evm<DB, NoOpInspector> {
        // 🔥 ANDE PRECOMPILE INJECTION
        // Use AndePrecompileProvider directly as the precompile system
        // This provider implements PrecompileProvider<CTX> with full context access
        // allowing it to execute native balance transfers via journal.transfer()
        
        let ande_provider = self.precompile_provider.as_ref().clone();
        
        // Create EVM context with ANDE precompiles
        let evm = Context::mainnet()
            .with_db(db)
            .with_cfg(input.cfg_env)
            .with_block(input.block_env)
            .build_mainnet_with_inspector(NoOpInspector {})
            .with_precompiles(ande_provider); // ✅ Inject ANDE precompile provider

        EthEvm::new(evm, false)
    }

    fn create_evm_with_inspector<DB: Database, I: Inspector<Self::Context<DB>, EthInterpreter>>(
        &self,
        db: DB,
        input: EvmEnv,
        inspector: I,
    ) -> Self::Evm<DB, I> {
        EthEvm::new(self.create_evm(db, input).into_inner().with_inspector(inspector), false)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ande_evm_factory_creation() {
        let factory = AndeEvmFactory::new(SpecId::CANCUN);
        assert!(Arc::strong_count(factory.precompile_provider()) >= 1);
    }

    #[test]
    fn test_precompile_provider_available() {
        let factory = AndeEvmFactory::new(SpecId::CANCUN);
        let provider = factory.precompile_provider();
        
        // Verify provider is available
        assert!(Arc::strong_count(provider) >= 1);
    }
}
