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
}
