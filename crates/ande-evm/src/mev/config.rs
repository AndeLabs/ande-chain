//! MEV Configuration from Environment Variables
//!
//! Provides configuration for MEV redistribution policy loaded from environment.

use alloy_primitives::{Address, U256};
use std::env;
use thiserror::Error;

use super::AndeMevRedirect;

/// MEV configuration loaded from environment variables
#[derive(Debug, Clone, Copy)]
pub struct MevConfig {
    /// MEV distribution sink address
    pub mev_sink: Address,
    /// Minimum profit threshold to classify as MEV
    pub min_threshold: U256,
    /// Whether MEV redistribution is enabled
    pub enabled: bool,
}

impl MevConfig {
    /// Load MEV configuration from environment variables
    ///
    /// Environment variables:
    /// - `ANDE_MEV_ENABLED`: Enable MEV redistribution (default: false)
    /// - `ANDE_MEV_SINK`: MEV distribution contract address (required if enabled)
    /// - `ANDE_MEV_MIN_THRESHOLD`: Minimum profit in wei (default: 0.001 ETH)
    ///
    /// # Returns
    ///
    /// - `Ok(Some(config))` if MEV is enabled and configured correctly
    /// - `Ok(None)` if MEV is disabled
    /// - `Err(error)` if configuration is invalid
    pub fn from_env() -> Result<Option<Self>, MevConfigError> {
        // Check if MEV is enabled
        let enabled = env::var("ANDE_MEV_ENABLED")
            .ok()
            .and_then(|v| v.parse::<bool>().ok())
            .unwrap_or(false);

        if !enabled {
            return Ok(None);
        }

        // MEV sink address is required when enabled
        let mev_sink = env::var("ANDE_MEV_SINK")
            .map_err(|_| MevConfigError::MissingSinkAddress)?
            .parse::<Address>()
            .map_err(|_| MevConfigError::InvalidSinkAddress)?;

        // SECURITY (M-2): Validate sink is not zero address
        if mev_sink.is_zero() {
            return Err(MevConfigError::ZeroSinkAddress);
        }

        // Min threshold is optional, defaults to 0.001 ETH
        let min_threshold = env::var("ANDE_MEV_MIN_THRESHOLD")
            .ok()
            .and_then(|v| v.parse::<u128>().ok())
            .map(U256::from)
            .unwrap_or(AndeMevRedirect::DEFAULT_MIN_MEV_THRESHOLD);

        Ok(Some(Self {
            mev_sink,
            min_threshold,
            enabled: true,
        }))
    }

    /// Convert configuration to AndeMevRedirect
    pub fn to_redirect(self) -> AndeMevRedirect {
        AndeMevRedirect::new(self.mev_sink, self.min_threshold)
    }

    /// Create a new MEV config with default threshold
    pub fn new(mev_sink: Address) -> Self {
        Self {
            mev_sink,
            min_threshold: AndeMevRedirect::DEFAULT_MIN_MEV_THRESHOLD,
            enabled: true,
        }
    }

    /// Create a new MEV config with custom threshold
    pub fn with_threshold(mev_sink: Address, min_threshold: U256) -> Self {
        Self {
            mev_sink,
            min_threshold,
            enabled: true,
        }
    }
}

/// Errors that can occur when loading MEV configuration
#[derive(Debug, Error)]
pub enum MevConfigError {
    /// MEV is enabled but sink address is not configured
    #[error("MEV enabled but ANDE_MEV_SINK not set")]
    MissingSinkAddress,

    /// Invalid MEV sink address format
    #[error("Invalid ANDE_MEV_SINK address format")]
    InvalidSinkAddress,

    /// MEV sink address cannot be zero (M-2 Security Fix)
    #[error("MEV sink address cannot be zero address")]
    ZeroSinkAddress,
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloy_primitives::address;
    use std::sync::Mutex;

    // Global mutex to serialize environment variable tests
    // Prevents race conditions when tests run in parallel
    static ENV_LOCK: Mutex<()> = Mutex::new(());

    #[test]
    fn test_mev_config_disabled_by_default() {
        let _lock = ENV_LOCK.lock().unwrap();

        // Clear env vars
        unsafe {
            env::remove_var("ANDE_MEV_ENABLED");
            env::remove_var("ANDE_MEV_SINK");
        }

        let config = MevConfig::from_env().unwrap();
        assert!(config.is_none());
    }

    #[test]
    fn test_mev_config_enabled_requires_sink() {
        let _lock = ENV_LOCK.lock().unwrap();

        unsafe {
            env::set_var("ANDE_MEV_ENABLED", "true");
            env::remove_var("ANDE_MEV_SINK");
        }

        let result = MevConfig::from_env();
        assert!(result.is_err());

        // Cleanup
        unsafe {
            env::remove_var("ANDE_MEV_ENABLED");
        }
    }

    #[test]
    fn test_mev_config_valid() {
        let _lock = ENV_LOCK.lock().unwrap();

        let sink = address!("0x1234567890123456789012345678901234567890");

        unsafe {
            env::set_var("ANDE_MEV_ENABLED", "true");
            env::set_var("ANDE_MEV_SINK", sink.to_string());
            env::remove_var("ANDE_MEV_MIN_THRESHOLD");  // Clear to get default
        }

        let config = MevConfig::from_env().unwrap().expect("config should be present");
        assert!(config.enabled);
        assert_eq!(config.mev_sink, sink);
        assert_eq!(config.min_threshold, AndeMevRedirect::DEFAULT_MIN_MEV_THRESHOLD);

        // Cleanup
        unsafe {
            env::remove_var("ANDE_MEV_ENABLED");
            env::remove_var("ANDE_MEV_SINK");
        }
    }

    #[test]
    fn test_mev_config_custom_threshold() {
        let _lock = ENV_LOCK.lock().unwrap();

        let sink = address!("0x1234567890123456789012345678901234567890");
        let threshold = U256::from(5_000_000_000_000_000u64); // 0.005 ETH

        unsafe {
            env::set_var("ANDE_MEV_ENABLED", "true");
            env::set_var("ANDE_MEV_SINK", sink.to_string());
            env::set_var("ANDE_MEV_MIN_THRESHOLD", "5000000000000000");
        }

        let config = MevConfig::from_env().unwrap().expect("config should be present");
        assert_eq!(config.min_threshold, threshold);

        // Cleanup
        unsafe {
            env::remove_var("ANDE_MEV_ENABLED");
            env::remove_var("ANDE_MEV_SINK");
            env::remove_var("ANDE_MEV_MIN_THRESHOLD");
        }
    }

    #[test]
    fn test_to_redirect() {
        let sink = address!("0x1234567890123456789012345678901234567890");
        let config = MevConfig::new(sink);

        let redirect = config.to_redirect();
        assert_eq!(redirect.mev_sink(), sink);
        assert_eq!(redirect.min_threshold(), AndeMevRedirect::DEFAULT_MIN_MEV_THRESHOLD);
    }

    // M-2 SECURITY FIX TEST: Zero Address Validation

    #[test]
    fn test_mev_config_rejects_zero_address() {
        let _lock = ENV_LOCK.lock().unwrap();

        unsafe {
            env::set_var("ANDE_MEV_ENABLED", "true");
            env::set_var("ANDE_MEV_SINK", "0x0000000000000000000000000000000000000000");
        }

        let result = MevConfig::from_env();
        assert!(result.is_err());

        match result.unwrap_err() {
            MevConfigError::ZeroSinkAddress => {
                // Expected error
            }
            other => panic!("Expected ZeroSinkAddress error, got {:?}", other),
        }

        // Cleanup
        unsafe {
            env::remove_var("ANDE_MEV_ENABLED");
            env::remove_var("ANDE_MEV_SINK");
        }
    }
}
