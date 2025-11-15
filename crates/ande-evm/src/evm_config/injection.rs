//! ANDE Precompile Injection
//!
//! This module provides runtime injection of ANDE precompiles into the EVM.
//! Since we can't easily override the EVM factory, we inject precompiles after
//! the EVM is created but before execution begins.

use super::ande_precompile_provider::AndePrecompileProvider;
use super::ANDE_PRECOMPILE_ADDRESS;
use revm::primitives::hardfork::SpecId;

/// Inject ANDE precompile into an existing precompile provider
///
/// This function can be called after creating an EVM to add the ANDE precompile
/// to the existing precompile set.
///
/// # Example
/// ```ignore
/// let mut evm = evm_config.evm_for_block(...);
/// inject_ande_precompile(&mut evm.handler.pre_execution().precompiles, spec_id);
/// ```
pub fn create_ande_precompile_provider(spec_id: SpecId) -> AndePrecompileProvider {
    AndePrecompileProvider::new(spec_id)
}

/// Get the ANDE precompile address
pub const fn ande_precompile_address() -> alloy_primitives::Address {
    ANDE_PRECOMPILE_ADDRESS
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_ande_precompile_provider() {
        let _provider = create_ande_precompile_provider(SpecId::CANCUN);
        // Provider created successfully
    }

    #[test]
    fn test_ande_precompile_address() {
        let addr = ande_precompile_address();
        assert_eq!(addr, ANDE_PRECOMPILE_ADDRESS);
    }
}
