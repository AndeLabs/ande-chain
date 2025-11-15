//! Consensus configuration for AndeChain PoS integration

use alloy_primitives::Address;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Configuration for AndeChain consensus integration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsensusConfig {
    /// Enable consensus contract integration
    #[serde(default = "default_enabled")]
    pub enabled: bool,

    /// AndeConsensusV2 contract address
    pub consensus_address: Address,

    /// AndeNativeStaking contract address
    pub staking_address: Address,

    /// RPC endpoint for contract calls
    #[serde(default = "default_rpc_url")]
    pub rpc_url: String,

    /// Private key file path for signing
    pub private_key_file: Option<PathBuf>,

    /// Private key (direct, not recommended for production)
    pub private_key: Option<String>,

    /// Enable block attestation (proposeBlock calls)
    #[serde(default = "default_attestation_enabled")]
    pub attestation_enabled: bool,

    /// Validator sync interval (in seconds)
    #[serde(default = "default_sync_interval")]
    pub validator_sync_interval_secs: u64,

    /// Enable automatic phase transition
    #[serde(default)]
    pub auto_phase_transition: bool,
}

impl ConsensusConfig {
    /// Create config from environment variables
    pub fn from_env() -> eyre::Result<Self> {
        let consensus_address = std::env::var("ANDE_CONSENSUS_ADDRESS")?
            .parse()
            .map_err(|e| eyre::eyre!("Invalid consensus address: {}", e))?;

        let staking_address = std::env::var("ANDE_STAKING_ADDRESS")?
            .parse()
            .map_err(|e| eyre::eyre!("Invalid staking address: {}", e))?;

        let rpc_url = std::env::var("ANDE_RPC_URL").unwrap_or_else(|_| default_rpc_url());

        let private_key_file = std::env::var("SEQUENCER_PRIVATE_KEY_FILE")
            .ok()
            .map(PathBuf::from);

        let private_key = std::env::var("SEQUENCER_PRIVATE_KEY").ok();

        let enabled = std::env::var("ANDE_CONSENSUS_ENABLED")
            .unwrap_or_else(|_| "true".to_string())
            .parse()
            .unwrap_or(true);

        let attestation_enabled = std::env::var("ANDE_ATTESTATION_ENABLED")
            .unwrap_or_else(|_| "true".to_string())
            .parse()
            .unwrap_or(true);

        Ok(Self {
            enabled,
            consensus_address,
            staking_address,
            rpc_url,
            private_key_file,
            private_key,
            attestation_enabled,
            validator_sync_interval_secs: default_sync_interval(),
            auto_phase_transition: false,
        })
    }

    /// Get private key file path if configured
    pub fn get_private_key_file(&self) -> Option<&PathBuf> {
        self.private_key_file.as_ref()
    }

    /// Get private key string if configured (not recommended for production)
    pub fn get_private_key(&self) -> Option<&String> {
        self.private_key.as_ref()
    }
}

impl Default for ConsensusConfig {
    fn default() -> Self {
        Self {
            enabled: default_enabled(),
            consensus_address: Address::ZERO,
            staking_address: Address::ZERO,
            rpc_url: default_rpc_url(),
            private_key_file: None,
            private_key: None,
            attestation_enabled: default_attestation_enabled(),
            validator_sync_interval_secs: default_sync_interval(),
            auto_phase_transition: false,
        }
    }
}

fn default_enabled() -> bool {
    true
}

fn default_rpc_url() -> String {
    "http://localhost:8545".to_string()
}

fn default_attestation_enabled() -> bool {
    true
}

fn default_sync_interval() -> u64 {
    300 // 5 minutes
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = ConsensusConfig::default();
        assert!(config.enabled);
        assert!(config.attestation_enabled);
        assert_eq!(config.rpc_url, "http://localhost:8545");
        assert_eq!(config.validator_sync_interval_secs, 300);
    }

    #[test]
    fn test_get_private_key() {
        let mut config = ConsensusConfig::default();
        config.private_key = Some(
            "0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80".to_string(),
        );

        let key = config.get_private_key();
        assert!(key.is_some());
    }
}
