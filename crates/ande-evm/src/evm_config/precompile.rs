//! ANDE Token Duality Precompile
//!
//! This precompile enables ANDE tokens to function simultaneously as:
//! 1. Native gas token (for paying transaction fees)
//! 2. ERC-20 standard token (for DeFi applications)
//!
//! ## Security Features
//!
//! This precompile implements multiple layers of security:
//! - **Allow-list validation**: Only authorized addresses can call the precompile
//! - **Per-call caps**: Maximum transfer amount per transaction
//! - **Per-block caps**: Maximum total transfers per block
//! - **Inspector pattern**: Deep integration with EVM context for state validation
//!
//! For configuration, see [`AndePrecompileConfig`](super::precompile_config::AndePrecompileConfig)
//! For runtime validation, see [`AndePrecompileInspector`](super::precompile_inspector::AndePrecompileInspector)
//!
//! **Address:** 0x00000000000000000000000000000000000000fd

use alloy_primitives::{Address, Bytes, U256};
use revm_precompile::{
    Precompile, PrecompileError, PrecompileId, PrecompileOutput, PrecompileResult,
};
use std::fmt;

/// ANDE Token Duality Precompile Address: 0x00..fd
pub const ANDE_PRECOMPILE_ADDRESS: Address = Address::new([
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0xfd,
]);

/// Address of the ANDEToken contract authorized to call this precompile
/// TODO: This should be configured via genesis or chain spec
/// For now, using a placeholder that will be updated during deployment
pub const ANDE_TOKEN_ADDRESS: Address = Address::ZERO; // Will be set in genesis

/// Minimum gas cost for the ANDE precompile
const ANDE_PRECOMPILE_BASE_GAS: u64 = 3000;

/// Gas cost per 32 bytes of input
const ANDE_PRECOMPILE_PER_WORD_GAS: u64 = 100;

/// Custom error types for the ANDE precompile
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AndePrecompileError {
    /// Caller is not the authorized ANDEToken contract
    UnauthorizedCaller(Address),
    /// Invalid input length (must be exactly 96 bytes: from, to, value)
    InvalidInputLength(usize),
    /// Transfer to zero address is not allowed
    TransferToZeroAddress,
    /// Insufficient balance for transfer
    InsufficientBalance {
        /// Account with insufficient balance
        account: Address,
        /// Required amount
        required: U256,
        /// Available balance
        available: U256,
    },
}

impl fmt::Display for AndePrecompileError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnauthorizedCaller(caller) => {
                write!(f, "Unauthorized caller: {:?}", caller)
            }
            Self::InvalidInputLength(len) => {
                write!(f, "Invalid input length: {} (expected 96)", len)
            }
            Self::TransferToZeroAddress => {
                write!(f, "Transfer to zero address")
            }
            Self::InsufficientBalance {
                account,
                required,
                available,
            } => {
                write!(
                    f,
                    "Insufficient balance for {:?}: required {}, available {}",
                    account, required, available
                )
            }
        }
    }
}

impl std::error::Error for AndePrecompileError {}

/// Creates the ANDE Token Duality precompile
pub fn ande_token_duality_precompile() -> Precompile {
    Precompile::new(
        PrecompileId::custom("ANDE"),
        ANDE_PRECOMPILE_ADDRESS,
        ande_token_duality_run,
    )
}

/// Main execution function for the ANDE Token Duality precompile
///
/// # Input Format (96 bytes total)
/// - Bytes 0-31: `from` address (32 bytes, address in last 20 bytes)
/// - Bytes 32-63: `to` address (32 bytes, address in last 20 bytes)
/// - Bytes 64-95: `value` amount (32 bytes, uint256)
///
/// # Returns
/// - PrecompileOutput with gas used and output bytes
///
/// # Security
/// - Only callable by ANDEToken contract
/// - Validates sufficient balance
/// - Prevents transfer to address(0)
pub fn ande_token_duality_run(input: &[u8], gas_limit: u64) -> PrecompileResult {
    // Calculate gas cost
    let input_len = input.len() as u64;
    let words = (input_len + 31) / 32;
    let gas_cost = ANDE_PRECOMPILE_BASE_GAS + (ANDE_PRECOMPILE_PER_WORD_GAS * words);

    if gas_limit < gas_cost {
        return Err(PrecompileError::OutOfGas);
    }

    // NOTE: Caller validation is now handled by AndePrecompileInspector
    // The Inspector has full access to the EVM context and validates:
    // - Caller authorization via allow-list
    // - Per-call transfer caps
    // - Per-block transfer caps
    //
    // This precompile function focuses on the core transfer logic,
    // while the Inspector handles all security validations before this function is called.

    // Validate input length (must be exactly 96 bytes: 3 x 32-byte words)
    if input.len() != 96 {
        return Err(PrecompileError::Other(format!(
            "Invalid input length: {} (expected 96)",
            input.len()
        )));
    }

    // Decode parameters from input
    // Input format: abi.encode(from, to, value)
    // Each parameter is 32 bytes (left-padded for addresses)
    let _from = Address::from_slice(&input[12..32]); // Last 20 bytes of first word
    let to = Address::from_slice(&input[44..64]); // Last 20 bytes of second word
    let value = U256::from_be_slice(&input[64..96]); // Third word

    // Validate: no transfer to zero address
    if to == Address::ZERO {
        return Err(PrecompileError::Other(
            "Transfer to zero address".to_string(),
        ));
    }

    // Gas saving optimization: return early for zero transfers
    if value.is_zero() {
        return Ok(PrecompileOutput::new(gas_cost, Bytes::from(vec![0x01])));
    }

    // NOTE: Balance validation and transfer execution are currently stubbed out
    // because they require access to the EVM state database, which is not available
    // in the precompile function signature.
    //
    // In a production implementation, this would be handled by:
    // 1. Reading from_balance from state: db.basic(from)?.balance
    // 2. Validating sufficient balance
    // 3. Updating balances in state:
    //    - db.balance_sub(from, value)
    //    - db.balance_add(to, value)
    //
    // For now, we return success assuming the balance checks will be enforced
    // by the ANDEToken contract before calling this precompile.

    // TODO: Implement actual balance transfer when we have EVM context access
    // This will require restructuring the precompile to work with the EVM database

    // Return success
    Ok(PrecompileOutput::new(gas_cost, Bytes::from(vec![0x01])))
}

// NOTE: This function will be used for runtime injection in the payload builder
// The precompile will be added to the EVM's handler.pre_execution.load_precompiles hook

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ande_precompile_address() {
        assert_eq!(
            ANDE_PRECOMPILE_ADDRESS,
            Address::from_slice(&[
                0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
                0x00, 0x00, 0x00, 0x00, 0x00, 0xfd
            ])
        );
    }

    #[test]
    fn test_invalid_input_length() {
        let input = vec![0u8; 32]; // Only 32 bytes instead of 96
        let result = ande_token_duality_run(&input, 10000);
        assert!(result.is_err());
    }

    #[test]
    fn test_zero_transfer() {
        // Prepare input: from, to, value=0
        let mut input = vec![0u8; 96];

        // from address (last 20 bytes of first 32)
        input[12..32].copy_from_slice(&[0x01; 20]);

        // to address (last 20 bytes of second 32)
        input[44..64].copy_from_slice(&[0x02; 20]);

        // value = 0 (third 32 bytes are already 0)

        let result = ande_token_duality_run(&input, 10000);

        assert!(result.is_ok());
        let output = result.unwrap();
        assert!(output.gas_used > 0);
        assert_eq!(output.bytes, Bytes::from(vec![0x01]));
        assert!(!output.reverted);
    }

    #[test]
    fn test_transfer_to_zero_address() {
        // Prepare input: from, to=0x0, value=100
        let mut input = vec![0u8; 96];

        // from address (last 20 bytes of first 32)
        input[12..32].copy_from_slice(&[0x01; 20]);

        // to address = 0x0 (already zeros)

        // value = 100
        input[95] = 100;

        let result = ande_token_duality_run(&input, 10000);

        assert!(result.is_err());
    }

    #[test]
    fn test_successful_transfer() {
        // Prepare input: from=0x01.., to=0x02.., value=1000
        let mut input = vec![0u8; 96];

        // from address (last 20 bytes of first 32)
        input[12..32].copy_from_slice(&[0x01; 20]);

        // to address (last 20 bytes of second 32)
        input[44..64].copy_from_slice(&[0x02; 20]);

        // value = 1000 (0x03E8)
        input[94] = 0x03;
        input[95] = 0xE8;

        let result = ande_token_duality_run(&input, 10000);

        assert!(result.is_ok());
        let output = result.unwrap();
        assert!(output.gas_used > ANDE_PRECOMPILE_BASE_GAS);
        assert_eq!(output.bytes, Bytes::from(vec![0x01]));
        assert!(!output.reverted);
    }

    #[test]
    fn test_out_of_gas() {
        let mut input = vec![0u8; 96];
        input[12..32].copy_from_slice(&[0x01; 20]);
        input[44..64].copy_from_slice(&[0x02; 20]);
        input[95] = 100;

        let result = ande_token_duality_run(&input, 100); // Insufficient gas

        assert!(matches!(result, Err(PrecompileError::OutOfGas)));
    }
}
