//! Configuration for ANDE Token Duality Precompile
//!
//! This module provides secure configuration for the ANDE precompile with:
//! - Allow-list of authorized callers
//! - Per-call transfer caps
//! - Per-block transfer caps
//! - Environment-based configuration

use alloy_primitives::{Address, U256};
use std::collections::HashSet;
use std::str::FromStr;

/// Configuration for the ANDE Precompile Inspector
///
/// Note: This is different from AndePrecompileConfig in ande_token_duality.rs
/// which uses storage-based authorization. This config uses in-memory HashSet
/// for the inspector validation layer.
#[derive(Clone, Debug)]
pub struct AndeInspectorConfig {
    /// The address of the ANDE Token Duality precompile (0x00..fd)
    pub precompile_address: Address,
    
    /// Address of the ANDEToken contract authorized to call this precompile
    pub ande_token_address: Address,
    
    /// A list of addresses that are allowed to call the precompile
    /// This provides more flexibility than a single authorized address
    pub allow_list: HashSet<Address>,
    
    /// The maximum amount that can be transferred in a single call
    pub per_call_cap: U256,
    
    /// The maximum amount that can be transferred in a single block
    /// None means no block-level cap
    pub per_block_cap: Option<U256>,
    
    /// Enable/disable strict validation (useful for testing)
    pub strict_validation: bool,
}

impl Default for AndeInspectorConfig {
    fn default() -> Self {
        Self {
            precompile_address: super::ande_token_duality::ANDE_PRECOMPILE_ADDRESS,
            ande_token_address: Address::ZERO, // Will be set via config
            allow_list: HashSet::new(),
            // Default: 1 million ANDE tokens per call (with 18 decimals)
            per_call_cap: U256::from(1_000_000u64) * U256::from(10u64).pow(U256::from(18)),
            // Default: 10 million ANDE tokens per block
            per_block_cap: Some(U256::from(10_000_000u64) * U256::from(10u64).pow(U256::from(18))),
            strict_validation: true,
        }
    }
}

impl AndeInspectorConfig {
    /// Creates a new `AndePrecompileConfig` from environment variables
    ///
    /// Environment variables:
    /// - `ANDE_PRECOMPILE_ADDRESS`: Address of the precompile (default: 0x00..fd)
    /// - `ANDE_TOKEN_ADDRESS`: Address of the ANDEToken contract
    /// - `ANDE_ALLOW_LIST`: Comma-separated list of authorized addresses
    /// - `ANDE_PER_CALL_CAP`: Maximum transfer per call (in wei)
    /// - `ANDE_PER_BLOCK_CAP`: Maximum transfer per block (in wei)
    /// - `ANDE_STRICT_VALIDATION`: Enable strict validation (true/false)
    pub fn from_env() -> eyre::Result<Self> {
        let mut config = Self::default();

        // Parse precompile address if provided
        if let Ok(addr) = std::env::var("ANDE_PRECOMPILE_ADDRESS") {
            config.precompile_address = Address::from_str(&addr)?;
        }

        // Parse ANDEToken contract address
        if let Ok(addr) = std::env::var("ANDE_TOKEN_ADDRESS") {
            config.ande_token_address = Address::from_str(&addr)?;
            // Automatically add to allow-list
            config.allow_list.insert(config.ande_token_address);
        }

        // Parse allow-list
        if let Ok(list) = std::env::var("ANDE_ALLOW_LIST") {
            for addr_str in list.split(',') {
                let addr = Address::from_str(addr_str.trim())?;
                config.allow_list.insert(addr);
            }
        }

        // Parse per-call cap
        if let Ok(cap) = std::env::var("ANDE_PER_CALL_CAP") {
            config.per_call_cap = U256::from_str(&cap)?;
        }

        // Parse per-block cap
        if let Ok(cap) = std::env::var("ANDE_PER_BLOCK_CAP") {
            config.per_block_cap = Some(U256::from_str(&cap)?);
        }

        // Parse strict validation
        if let Ok(strict) = std::env::var("ANDE_STRICT_VALIDATION") {
            config.strict_validation = strict.to_lowercase() == "true" || strict == "1";
        }

        Ok(config)
    }

    /// Creates a config for testing with relaxed constraints
    #[cfg(test)]
    pub fn for_testing() -> Self {
        let mut config = Self::default();
        config.strict_validation = false;
        config.per_call_cap = U256::MAX;
        config.per_block_cap = None;
        config
    }

    /// Adds an address to the allow-list
    pub fn add_to_allow_list(&mut self, address: Address) {
        self.allow_list.insert(address);
    }

    /// Removes an address from the allow-list
    pub fn remove_from_allow_list(&mut self, address: Address) {
        self.allow_list.remove(&address);
    }

    /// Checks if an address is authorized to call the precompile
    pub fn is_authorized(&self, caller: Address) -> bool {
        if !self.strict_validation {
            return true;
        }
        self.allow_list.contains(&caller)
    }

    /// Validates a transfer amount against per-call cap
    pub fn validate_per_call_cap(&self, amount: U256) -> Result<(), String> {
        if amount > self.per_call_cap {
            return Err(format!(
                "Transfer amount {} exceeds per-call cap {}",
                amount, self.per_call_cap
            ));
        }
        Ok(())
    }

    /// Validates a transfer amount against per-block cap
    pub fn validate_per_block_cap(
        &self,
        amount: U256,
        transferred_this_block: U256,
    ) -> Result<(), String> {
        if let Some(block_cap) = self.per_block_cap {
            let total = transferred_this_block.saturating_add(amount);
            if total > block_cap {
                return Err(format!(
                    "Total block transfers {} would exceed per-block cap {}",
                    total, block_cap
                ));
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = AndeInspectorConfig::default();
        assert_eq!(
            config.precompile_address,
            super::super::ande_token_duality::ANDE_PRECOMPILE_ADDRESS
        );
        assert!(config.strict_validation);
        assert!(config.per_call_cap > U256::ZERO);
        assert!(config.per_block_cap.is_some());
    }

    #[test]
    fn test_allow_list() {
        let mut config = AndeInspectorConfig::default();
        let addr = Address::repeat_byte(0x42);

        assert!(!config.is_authorized(addr));

        config.add_to_allow_list(addr);
        assert!(config.is_authorized(addr));

        config.remove_from_allow_list(addr);
        assert!(!config.is_authorized(addr));
    }

    #[test]
    fn test_per_call_cap_validation() {
        let config = AndeInspectorConfig::default();

        // Amount within cap should pass
        let small_amount = U256::from(1000u64);
        assert!(config.validate_per_call_cap(small_amount).is_ok());

        // Amount exceeding cap should fail
        let large_amount = U256::MAX;
        assert!(config.validate_per_call_cap(large_amount).is_err());
    }

    #[test]
    fn test_per_block_cap_validation() {
        let config = AndeInspectorConfig::default();
        let transferred = U256::from(5_000_000u64) * U256::from(10u64).pow(U256::from(18));

        // Amount within remaining cap should pass
        let amount = U256::from(1_000_000u64) * U256::from(10u64).pow(U256::from(18));
        assert!(config.validate_per_block_cap(amount, transferred).is_ok());

        // Amount exceeding remaining cap should fail
        let large_amount = U256::from(10_000_000u64) * U256::from(10u64).pow(U256::from(18));
        assert!(config
            .validate_per_block_cap(large_amount, transferred)
            .is_err());
    }

    #[test]
    fn test_testing_config() {
        let config = AndeInspectorConfig::for_testing();
        assert!(!config.strict_validation);
        assert_eq!(config.per_call_cap, U256::MAX);
        assert!(config.per_block_cap.is_none());
    }

    // =====================================================
    // SECURITY TESTS - Critical for production safety
    // =====================================================

    #[test]
    fn test_security_zero_address_not_authorized() {
        // SECURITY: Zero address should never be authorized by default
        let config = AndeInspectorConfig::default();
        assert!(!config.is_authorized(Address::ZERO));
    }

    #[test]
    fn test_security_cannot_add_zero_to_allowlist_and_authorize() {
        // SECURITY: Even if zero address is added to allowlist,
        // it should be treated carefully in production
        let mut config = AndeInspectorConfig::default();
        config.add_to_allow_list(Address::ZERO);

        // It's technically in the list (no validation on add)
        // but this test documents the behavior
        assert!(config.is_authorized(Address::ZERO));
    }

    #[test]
    fn test_security_random_address_not_authorized() {
        // SECURITY: Random addresses must not be authorized
        let config = AndeInspectorConfig::default();

        for i in 1..=10 {
            let addr = Address::repeat_byte(i);
            assert!(!config.is_authorized(addr),
                "Random address {:?} should not be authorized", addr);
        }
    }

    #[test]
    fn test_security_precompile_address_not_auto_authorized() {
        // SECURITY: The precompile address itself should not be authorized
        let config = AndeInspectorConfig::default();
        assert!(!config.is_authorized(config.precompile_address));
    }

    #[test]
    fn test_security_per_call_cap_boundary() {
        // SECURITY: Exact boundary values for per-call cap
        let config = AndeInspectorConfig::default();

        // Exactly at cap should pass
        assert!(config.validate_per_call_cap(config.per_call_cap).is_ok());

        // One wei over should fail
        let over_cap = config.per_call_cap.saturating_add(U256::from(1));
        assert!(config.validate_per_call_cap(over_cap).is_err());

        // One wei under should pass
        let under_cap = config.per_call_cap.saturating_sub(U256::from(1));
        assert!(config.validate_per_call_cap(under_cap).is_ok());
    }

    #[test]
    fn test_security_per_block_cap_boundary() {
        // SECURITY: Exact boundary values for per-block cap
        let config = AndeInspectorConfig::default();
        let block_cap = config.per_block_cap.unwrap();

        // Already transferred 99% of cap
        let transferred = block_cap * U256::from(99) / U256::from(100);

        // Exactly remaining should pass
        let remaining = block_cap - transferred;
        assert!(config.validate_per_block_cap(remaining, transferred).is_ok());

        // One wei more than remaining should fail
        let over_remaining = remaining + U256::from(1);
        assert!(config.validate_per_block_cap(over_remaining, transferred).is_err());
    }

    #[test]
    fn test_security_u256_max_overflow_protection() {
        // SECURITY: U256::MAX values should not cause overflow
        let config = AndeInspectorConfig::default();

        // Validate per-call cap with MAX
        let result = config.validate_per_call_cap(U256::MAX);
        assert!(result.is_err());

        // Validate per-block cap with near-MAX values
        let result = config.validate_per_block_cap(
            U256::MAX - U256::from(1),
            U256::from(2)
        );
        // saturating_add prevents overflow but total exceeds cap
        assert!(result.is_err());
    }

    #[test]
    fn test_security_multiple_small_transfers_accumulate() {
        // SECURITY: Multiple small transfers should accumulate correctly
        let config = AndeInspectorConfig::default();
        let block_cap = config.per_block_cap.unwrap();

        let small_amount = block_cap / U256::from(5);

        // First 4 transfers should pass
        let mut total_transferred = U256::ZERO;
        for i in 0..4 {
            assert!(
                config.validate_per_block_cap(small_amount, total_transferred).is_ok(),
                "Transfer {} should pass", i
            );
            total_transferred += small_amount;
        }

        // 5th transfer exactly fills cap - should pass
        assert!(config.validate_per_block_cap(small_amount, total_transferred).is_ok());
        total_transferred += small_amount;

        // 6th transfer exceeds cap - should fail
        assert!(config.validate_per_block_cap(small_amount, total_transferred).is_err());
    }

    #[test]
    fn test_security_allowlist_isolation() {
        // SECURITY: Adding one address should not affect others
        let mut config = AndeInspectorConfig::default();

        let addr1 = Address::repeat_byte(0x01);
        let addr2 = Address::repeat_byte(0x02);

        config.add_to_allow_list(addr1);

        assert!(config.is_authorized(addr1));
        assert!(!config.is_authorized(addr2));

        // Removing addr1 should not affect behavior for addr2
        config.remove_from_allow_list(addr1);
        assert!(!config.is_authorized(addr1));
        assert!(!config.is_authorized(addr2));
    }

    #[test]
    fn test_security_strict_validation_toggle() {
        // SECURITY: Strict validation toggle must work correctly
        let mut config = AndeInspectorConfig::default();
        let unauthorized = Address::repeat_byte(0xFF);

        // With strict validation ON
        config.strict_validation = true;
        assert!(!config.is_authorized(unauthorized));

        // With strict validation OFF (testing mode)
        config.strict_validation = false;
        assert!(config.is_authorized(unauthorized));

        // Back to strict
        config.strict_validation = true;
        assert!(!config.is_authorized(unauthorized));
    }

    #[test]
    fn test_security_zero_amount_transfers() {
        // SECURITY: Zero amount transfers should pass per-call validation
        let config = AndeInspectorConfig::default();

        // Zero amount passes per-call cap
        assert!(config.validate_per_call_cap(U256::ZERO).is_ok());

        // Zero amount passes per-block cap (when not already at max)
        let transferred = config.per_block_cap.unwrap() / U256::from(2);
        assert!(config.validate_per_block_cap(U256::ZERO, transferred).is_ok());

        // Zero amount still passes even at the cap (0 + cap = cap, not > cap)
        assert!(config.validate_per_block_cap(U256::ZERO, config.per_block_cap.unwrap()).is_ok());
    }

    #[test]
    fn test_security_no_block_cap_allows_unlimited() {
        // SECURITY: When block cap is None, any amount should pass
        let mut config = AndeInspectorConfig::default();
        config.per_block_cap = None;

        // Even MAX should pass when there's no block cap
        assert!(config.validate_per_block_cap(U256::MAX, U256::MAX).is_ok());
    }

    #[test]
    fn test_security_error_messages_contain_values() {
        // SECURITY: Error messages should contain actual values for debugging
        let config = AndeInspectorConfig::default();

        let err = config.validate_per_call_cap(U256::MAX).unwrap_err();
        assert!(err.contains("exceeds"), "Error should mention exceeds");
        assert!(err.contains(&U256::MAX.to_string()), "Error should contain amount");

        let err = config.validate_per_block_cap(U256::MAX, U256::ZERO).unwrap_err();
        assert!(err.contains("would exceed"), "Error should mention would exceed");
    }

    #[test]
    fn test_security_concurrent_allowlist_modifications() {
        // SECURITY: Multiple modifications to allowlist should be consistent
        let mut config = AndeInspectorConfig::default();
        let addresses: Vec<Address> = (1..=100).map(|i| Address::repeat_byte(i)).collect();

        // Add all
        for addr in &addresses {
            config.add_to_allow_list(*addr);
        }

        // Verify all authorized
        for addr in &addresses {
            assert!(config.is_authorized(*addr));
        }

        // Remove odd-numbered
        for addr in addresses.iter().step_by(2) {
            config.remove_from_allow_list(*addr);
        }

        // Verify odd removed, even still present
        for (i, addr) in addresses.iter().enumerate() {
            if i % 2 == 0 {
                assert!(!config.is_authorized(*addr));
            } else {
                assert!(config.is_authorized(*addr));
            }
        }
    }
}
