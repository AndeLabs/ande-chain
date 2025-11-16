//! ANDE EVM Factory with Custom Precompiles
//!
//! This module implements a wrapper EVM factory that injects ANDE precompiles
//! while maintaining full compatibility with Reth's standard infrastructure.
//!
//! ## Architecture (Wrapper Pattern)
//!
//! ```text
//! AndeEvmFactory<InnerFactory>
//!     ↓ wraps
//! EthEvmFactory (standard Reth)
//!     ↓ delegates to + injects precompiles
//! Standard EVM with ANDE precompiles
//! ```
//!
//! This wrapper pattern is modular and scalable:
//! - ✅ Compatible with standard PayloadBuilder
//! - ✅ Compatible with standard Executor
//! - ✅ Only customizes what we need (precompiles)
//! - ✅ Easy to add more customizations later

use alloy_evm::{
    eth::EthEvmContext,
    precompiles::PrecompilesMap,
    EthEvmFactory,
    EvmEnv, EvmFactory,
};
use reth_ethereum::evm::{
    primitives::Database,
    revm::{
        context::TxEnv,
        context_interface::result::{EVMError, HaltReason},
        inspector::{Inspector, NoOpInspector},
        interpreter::interpreter::EthInterpreter,
        primitives::hardfork::SpecId,
    },
};
use reth_evm::EthEvm;

/// ANDE EVM Factory - Wrapper Pattern
///
/// Wraps any inner EVM factory (typically `EthEvmFactory`) and injects
/// ANDE custom precompiles.
///
/// ## Generic Parameter
/// - `F`: Inner factory type (usually `EthEvmFactory`)
///
/// ## Usage
/// ```ignore
/// let factory = AndeEvmFactory::new(EthEvmFactory::default());
/// ```
#[derive(Debug, Clone)]
pub struct AndeEvmFactory<F = EthEvmFactory> {
    /// Inner EVM factory (delegates most work to this)
    inner: F,
    /// Spec ID for precompile configuration
    spec_id: SpecId,
}

impl<F> AndeEvmFactory<F> {
    /// Create a new ANDE EVM factory wrapping an inner factory
    pub fn new(inner: F, spec_id: SpecId) -> Self {
        Self { inner, spec_id }
    }

    /// Get reference to inner factory
    pub fn inner(&self) -> &F {
        &self.inner
    }
}

impl Default for AndeEvmFactory<EthEvmFactory> {
    fn default() -> Self {
        Self::new(EthEvmFactory::default(), SpecId::CANCUN)
    }
}

/// Implementation for wrapping EthEvmFactory
///
/// This delegates to the inner factory but uses PrecompilesMap
/// to maintain compatibility with standard Reth infrastructure.
impl EvmFactory for AndeEvmFactory<EthEvmFactory> {
    type Evm<DB: Database, I: Inspector<EthEvmContext<DB>, EthInterpreter>> =
        EthEvm<DB, I, PrecompilesMap>;
    type Tx = TxEnv;
    type Error<DBError: core::error::Error + Send + Sync + 'static> = EVMError<DBError>;
    type HaltReason = HaltReason;
    type Context<DB: Database> = EthEvmContext<DB>;
    type Spec = SpecId;
    type Precompiles = PrecompilesMap;

    fn create_evm<DB: Database>(&self, db: DB, input: EvmEnv) -> Self::Evm<DB, NoOpInspector> {
        // TODO: Here we would inject ANDE precompiles into the PrecompilesMap
        // For now, delegate to inner factory
        // In next phase: extend precompiles map with ANDE custom precompiles
        
        tracing::debug!(
            spec_id = ?self.spec_id,
            "Creating ANDE EVM (wrapper pattern - precompile injection pending)"
        );
        
        self.inner.create_evm(db, input)
    }

    fn create_evm_with_inspector<DB: Database, I: Inspector<Self::Context<DB>, EthInterpreter>>(
        &self,
        db: DB,
        input: EvmEnv,
        inspector: I,
    ) -> Self::Evm<DB, I> {
        self.inner.create_evm_with_inspector(db, input, inspector)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ande_evm_factory_creation() {
        let inner = EthEvmFactory::default();
        let factory = AndeEvmFactory::new(inner, SpecId::CANCUN);
        assert!(matches!(factory.spec_id, SpecId::CANCUN));
    }

    #[test]
    fn test_default_factory() {
        let _factory = AndeEvmFactory::default();
        // Factory created successfully with default inner
    }
}
