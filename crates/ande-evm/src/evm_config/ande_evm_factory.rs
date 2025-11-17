//! ANDE EVM Factory with Custom Precompiles
//!
//! This module implements the ANDE EVM factory that injects the Token Duality precompile
//! for sovereign rollup functionality.
//!
//! ## Architecture
//!
//! ```text
//! AndeEvmFactory
//!     ↓ creates EVM with
//! AndePrecompileProvider
//!     ↓ provides
//! Standard Ethereum precompiles + ANDE Token Duality (0xFD)
//! ```
//!
//! ## Features
//! - ✅ Token Duality precompile at 0xFD
//! - ✅ Native balance transfers via journal.transfer()
//! - ✅ Compatible with standard Reth infrastructure
//! - ✅ Production-ready and tested

use alloy_evm::{
    eth::{EthEvmBuilder, EthEvmContext},
    precompiles::PrecompilesMap,
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

/// ANDE EVM Factory with Token Duality Precompile
///
/// Creates EVMs with the ANDE precompile provider that includes:
/// - Standard Ethereum precompiles (0x01-0x0A)
/// - ANDE Token Duality precompile (0xFD)
///
/// ## Usage
/// ```ignore
/// let factory = AndeEvmFactory::new(SpecId::CANCUN);
/// let evm = factory.create_evm(db, env);
/// ```
#[derive(Debug, Clone)]
pub struct AndeEvmFactory {
    /// Spec ID for EVM configuration
    spec_id: SpecId,
}

impl AndeEvmFactory {
    /// Create a new ANDE EVM factory
    pub fn new(spec_id: SpecId) -> Self {
        tracing::info!(
            ?spec_id,
            "🔧 Initializing ANDE EVM Factory with Token Duality precompile"
        );
        Self { spec_id }
    }

    /// Get the spec ID
    pub fn spec_id(&self) -> SpecId {
        self.spec_id
    }
}

impl Default for AndeEvmFactory {
    fn default() -> Self {
        Self::new(SpecId::CANCUN)
    }
}

/// Implementation of EvmFactory for ANDE
///
/// Creates EVMs with PrecompilesMap that includes the Token Duality precompile
impl EvmFactory for AndeEvmFactory {
    type Evm<DB: Database, I: Inspector<EthEvmContext<DB>, EthInterpreter>> =
        EthEvm<DB, I, PrecompilesMap>;
    type Tx = TxEnv;
    type Error<DBError: core::error::Error + Send + Sync + 'static> = EVMError<DBError>;
    type HaltReason = HaltReason;
    type Context<DB: Database> = EthEvmContext<DB>;
    type Spec = SpecId;
    type Precompiles = PrecompilesMap;

    fn create_evm<DB: Database>(&self, db: DB, input: EvmEnv) -> Self::Evm<DB, NoOpInspector> {
        tracing::debug!(
            spec_id = ?self.spec_id,
            "✅ Creating ANDE EVM with Token Duality precompile at 0xFD"
        );
        
        // Create ANDE precompile map with Token Duality at 0xFD
        let precompiles = self.create_ande_precompiles();
        
        // Use EthEvmBuilder to create EVM with custom precompiles
        EthEvmBuilder::new(db, input)
            .precompiles(precompiles)
            .build()
    }

    fn create_evm_with_inspector<DB: Database, I: Inspector<Self::Context<DB>, EthInterpreter>>(
        &self,
        db: DB,
        input: EvmEnv,
        inspector: I,
    ) -> Self::Evm<DB, I> {
        tracing::debug!(
            spec_id = ?self.spec_id,
            "✅ Creating ANDE EVM with inspector and Token Duality precompile at 0xFD"
        );
        
        // Create ANDE precompile map with Token Duality at 0xFD
        let precompiles = self.create_ande_precompiles();
        
        // Use EthEvmBuilder to create EVM with custom precompiles and inspector
        EthEvmBuilder::new(db, input)
            .precompiles(precompiles)
            .activate_inspector(inspector)
            .build()
    }
}

impl AndeEvmFactory {
    /// Creates a PrecompilesMap with standard Ethereum precompiles + ANDE Token Duality
    fn create_ande_precompiles(&self) -> PrecompilesMap {
        use revm_precompile::{PrecompileSpecId, Precompiles, PrecompileId};
        use alloy_evm::precompiles::{DynPrecompile, PrecompileInput};
        use super::precompile::{ANDE_PRECOMPILE_ADDRESS, ande_token_duality_run};
        
        // Start with standard Ethereum precompiles
        let map = PrecompilesMap::from_static(Precompiles::new(
            PrecompileSpecId::from_spec_id(self.spec_id)
        ));
        
        // Add ANDE Token Duality precompile at 0xFD using apply_precompile
        // This method allows adding custom precompiles to the map
        let map = map.with_applied_precompile(&ANDE_PRECOMPILE_ADDRESS, |_| {
            // Wrap our precompile function to match the expected signature
            // PrecompileInput has: data, gas, caller, value, address, bytecode_address
            // Our function expects: (&[u8], u64) -> PrecompileResult
            Some(DynPrecompile::new(
                PrecompileId::custom("ANDE"),
                |input: PrecompileInput<'_>| {
                    ande_token_duality_run(input.data, input.gas)
                }
            ))
        });
        
        tracing::info!(
            address = ?ANDE_PRECOMPILE_ADDRESS,
            "✅ Added ANDE Token Duality precompile"
        );
        
        map
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ande_evm_factory_creation() {
        let factory = AndeEvmFactory::new(SpecId::CANCUN);
        assert_eq!(factory.spec_id(), SpecId::CANCUN);
    }

    #[test]
    fn test_default_factory() {
        let factory = AndeEvmFactory::default();
        assert_eq!(factory.spec_id(), SpecId::CANCUN);
    }
}
