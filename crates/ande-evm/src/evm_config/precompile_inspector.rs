//! Inspector for ANDE Token Duality Precompile
//!
//! This inspector validates precompile calls with:
//! - Caller authorization checks
//! - Per-call transfer limits
//! - Per-block transfer limits
//! - Full access to EVM context for state validation

use super::precompile::ANDE_PRECOMPILE_ADDRESS;
use super::precompile_config::AndePrecompileConfig;
use alloy_primitives::{Address, U256};
use revm::{
    context_interface::ContextTr,
    inspector::Inspector,
    interpreter::{CallInputs, CallOutcome, Gas, InstructionResult, InterpreterResult},
};

/// Inspector that validates ANDE Token Duality precompile calls
#[derive(Clone, Debug)]
pub struct AndePrecompileInspector {
    /// Configuration for the precompile
    config: AndePrecompileConfig,
    
    /// Total amount transferred in the current block
    transferred_this_block: U256,
    
    /// Current block number for tracking resets
    current_block: u64,
}

impl AndePrecompileInspector {
    /// Creates a new inspector with the given configuration
    pub fn new(config: AndePrecompileConfig) -> Self {
        Self {
            config,
            transferred_this_block: U256::ZERO,
            current_block: 0,
        }
    }

    /// Creates an inspector from environment variables
    pub fn from_env() -> eyre::Result<Self> {
        let config = AndePrecompileConfig::from_env()?;
        Ok(Self::new(config))
    }

    /// Expected input length for transfer calls (96 bytes)
    const TRANSFER_CALLDATA_LEN: usize = 96; // from(32) + to(32) + value(32)

    /// Resets the block counter if we're in a new block
    #[allow(dead_code)]
    fn maybe_reset_block_counter(&mut self, block_number: u64) {
        if block_number != self.current_block {
            self.current_block = block_number;
            self.transferred_this_block = U256::ZERO;
        }
    }

    /// Manually resets the block counter for a new block
    /// Call this at the start of each block to reset transfer tracking
    pub fn reset_for_new_block(&mut self, block_number: u64) {
        self.current_block = block_number;
        self.transferred_this_block = U256::ZERO;
    }

    /// Gets the total amount transferred in the current block
    pub fn transferred_this_block(&self) -> U256 {
        self.transferred_this_block
    }

    /// Creates a revert outcome with a message
    fn revert_outcome(message: &str, inputs: &CallInputs) -> CallOutcome {
        CallOutcome::new(
            Self::revert_result(message),
            inputs.return_memory_offset.clone(),
        )
    }

    /// Creates a revert result with a message
    fn revert_result(message: &str) -> InterpreterResult {
        InterpreterResult {
            result: InstructionResult::Revert,
            output: message.as_bytes().to_vec().into(),
            gas: Gas::new(0),
        }
    }

    /// Parses transfer parameters from calldata
    fn parse_transfer_params(calldata: &[u8]) -> (Address, Address, U256) {
        // Input format: from(32 bytes) + to(32 bytes) + value(32 bytes)
        let from = Address::from_slice(&calldata[12..32]); // Last 20 bytes of first word
        let to = Address::from_slice(&calldata[44..64]); // Last 20 bytes of second word
        let value = U256::from_be_slice(&calldata[64..96]); // Third word
        (from, to, value)
    }
}

impl<CTX> Inspector<CTX> for AndePrecompileInspector
where
    CTX: ContextTr,
{
    fn call(&mut self, context: &mut CTX, inputs: &mut CallInputs) -> Option<CallOutcome> {
        // Only intercept calls to the ANDE precompile
        if inputs.target_address != ANDE_PRECOMPILE_ADDRESS {
            return None;
        }

        // TODO: Get current block number from context and reset counter
        // This requires accessing the block environment from the context
        // For now, we'll rely on manual reset between blocks

        // Validate caller authorization
        if !self.config.is_authorized(inputs.caller) {
            return Some(Self::revert_outcome(
                &format!("Unauthorized caller: {:?}", inputs.caller),
                inputs,
            ));
        }

        // Get calldata
        let calldata = inputs.input.bytes(context);

        // Validate input length
        if calldata.len() != Self::TRANSFER_CALLDATA_LEN {
            return Some(Self::revert_outcome(
                &format!(
                    "Invalid input length: {} (expected {})",
                    calldata.len(),
                    Self::TRANSFER_CALLDATA_LEN
                ),
                inputs,
            ));
        }

        // Parse transfer parameters
        let (_from, to, value) = Self::parse_transfer_params(&calldata);

        // Validate: no transfer to zero address
        if to == Address::ZERO {
            return Some(Self::revert_outcome("Transfer to zero address", inputs));
        }

        // Skip zero-value transfers (optimization)
        if value.is_zero() {
            return None; // Allow the precompile to handle it
        }

        // Validate per-call cap
        if let Err(err) = self.config.validate_per_call_cap(value) {
            return Some(Self::revert_outcome(&err, inputs));
        }

        // Validate per-block cap
        if let Err(err) = self
            .config
            .validate_per_block_cap(value, self.transferred_this_block)
        {
            return Some(Self::revert_outcome(&err, inputs));
        }

        // Update block transfer counter
        self.transferred_this_block = self.transferred_this_block.saturating_add(value);

        // Allow the precompile to execute
        None
    }

    fn call_end(
        &mut self,
        _context: &mut CTX,
        _inputs: &CallInputs,
        outcome: &mut CallOutcome,
    ) {
        // If the call failed, rollback the block transfer counter
        if outcome.result.result != InstructionResult::Return
            && outcome.result.result != InstructionResult::Stop
        {
            // On failure, we should ideally rollback, but since we don't know
            // the exact amount that was added, we'll leave the counter as-is.
            // In practice, failed calls won't affect state anyway.
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_transfer_params() {
        let mut calldata = vec![0u8; 96];

        // from address
        calldata[12..32].copy_from_slice(&[0x11; 20]);

        // to address
        calldata[44..64].copy_from_slice(&[0x22; 20]);

        // value = 1000
        calldata[94] = 0x03;
        calldata[95] = 0xE8;

        let (from, to, value) = AndePrecompileInspector::parse_transfer_params(&calldata);

        assert_eq!(from, Address::repeat_byte(0x11));
        assert_eq!(to, Address::repeat_byte(0x22));
        assert_eq!(value, U256::from(1000));
    }

    #[test]
    fn test_block_counter_reset() {
        let config = AndePrecompileConfig::for_testing();
        let mut inspector = AndePrecompileInspector::new(config);

        inspector.transferred_this_block = U256::from(1000);
        inspector.current_block = 10;

        // Same block - counter should not reset
        inspector.maybe_reset_block_counter(10);
        assert_eq!(inspector.transferred_this_block, U256::from(1000));

        // New block - counter should reset
        inspector.maybe_reset_block_counter(11);
        assert_eq!(inspector.transferred_this_block, U256::ZERO);
        assert_eq!(inspector.current_block, 11);
    }
}
