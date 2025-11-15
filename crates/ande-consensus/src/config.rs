//! Configuration for consensus module

use alloy_primitives::Address;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::time::Duration;

/// Complete configuration for the consensus engine
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsensusConfig {
    /// Ethereum RPC endpoint URL
    pub rpc_url: String,

    /// WebSocket RPC URL (for event subscriptions)
    pub ws_url: String,

    /// Address of AndeConsensus contract
    pub consensus_contract: Address,

    /// Address of AndeSequencerCoordinator contract
    pub coordinator_contract: Address,

    /// Address of AndeSequencerRegistry contract
    pub registry_contract: Address,

    /// This node's sequencer address
    pub sequencer_address: Address,

    /// Path to private key file
    pub private_key_path: PathBuf,

    /// Chain ID
    pub chain_id: u64,

    /// Number of blocks per rotation
    pub blocks_per_rotation: u64,

    /// Timeout threshold (blocks without production before force rotation)
    pub timeout_blocks: u64,

    /// Block time (expected time between blocks)
    pub block_time: Duration,

    /// Sync interval (how often to sync validator set)
    pub sync_interval: Duration,

    /// Metrics server address
    pub metrics_addr: String,

    /// Enable strict validation
    pub strict_validation: bool,

    /// Maximum attestation age (blocks)
    pub max_attestation_age: u64,

    /// Whether to auto-unjail after jail period
    pub auto_unjail: bool,

    /// Data directory for state persistence
    pub data_dir: PathBuf,

    /// Log level
    pub log_level: String,
}

impl ConsensusConfig {
    /// Load configuration from environment variables
    ///
    /// # Errors
    ///
    /// Returns error if required environment variables are missing
    pub fn from_env() -> Result<Self, String> {
        Ok(Self {
            rpc_url: std::env::var("CONSENSUS_RPC_URL")
                .unwrap_or_else(|_| "http://localhost:8545".to_string()),

            ws_url: std::env::var("CONSENSUS_WS_URL")
                .unwrap_or_else(|_| "ws://localhost:8546".to_string()),

            consensus_contract: std::env::var("CONSENSUS_CONTRACT")
                .map_err(|_| "CONSENSUS_CONTRACT not set")?
                .parse()
                .map_err(|e| format!("Invalid CONSENSUS_CONTRACT address: {e}"))?,

            coordinator_contract: std::env::var("COORDINATOR_CONTRACT")
                .map_err(|_| "COORDINATOR_CONTRACT not set")?
                .parse()
                .map_err(|e| format!("Invalid COORDINATOR_CONTRACT address: {e}"))?,

            registry_contract: std::env::var("REGISTRY_CONTRACT")
                .map_err(|_| "REGISTRY_CONTRACT not set")?
                .parse()
                .map_err(|e| format!("Invalid REGISTRY_CONTRACT address: {e}"))?,

            sequencer_address: std::env::var("SEQUENCER_ADDRESS")
                .map_err(|_| "SEQUENCER_ADDRESS not set")?
                .parse()
                .map_err(|e| format!("Invalid SEQUENCER_ADDRESS: {e}"))?,

            private_key_path: std::env::var("PRIVATE_KEY_PATH")
                .map(PathBuf::from)
                .unwrap_or_else(|_| PathBuf::from("./sequencer.key")),

            chain_id: std::env::var("CHAIN_ID")
                .unwrap_or_else(|_| "6174".to_string())
                .parse()
                .map_err(|e| format!("Invalid CHAIN_ID: {e}"))?,

            blocks_per_rotation: std::env::var("BLOCKS_PER_ROTATION")
                .unwrap_or_else(|_| "100".to_string())
                .parse()
                .map_err(|e| format!("Invalid BLOCKS_PER_ROTATION: {e}"))?,

            timeout_blocks: std::env::var("TIMEOUT_BLOCKS")
                .unwrap_or_else(|_| "10".to_string())
                .parse()
                .map_err(|e| format!("Invalid TIMEOUT_BLOCKS: {e}"))?,

            block_time: Duration::from_secs(
                std::env::var("BLOCK_TIME_SECS")
                    .unwrap_or_else(|_| "2".to_string())
                    .parse()
                    .map_err(|e| format!("Invalid BLOCK_TIME_SECS: {e}"))?,
            ),

            sync_interval: Duration::from_secs(
                std::env::var("SYNC_INTERVAL_SECS")
                    .unwrap_or_else(|_| "30".to_string())
                    .parse()
                    .map_err(|e| format!("Invalid SYNC_INTERVAL_SECS: {e}"))?,
            ),

            metrics_addr: std::env::var("METRICS_ADDR")
                .unwrap_or_else(|_| "0.0.0.0:9090".to_string()),

            strict_validation: std::env::var("STRICT_VALIDATION")
                .unwrap_or_else(|_| "true".to_string())
                .parse()
                .map_err(|e| format!("Invalid STRICT_VALIDATION: {e}"))?,

            max_attestation_age: std::env::var("MAX_ATTESTATION_AGE")
                .unwrap_or_else(|_| "100".to_string())
                .parse()
                .map_err(|e| format!("Invalid MAX_ATTESTATION_AGE: {e}"))?,

            auto_unjail: std::env::var("AUTO_UNJAIL")
                .unwrap_or_else(|_| "true".to_string())
                .parse()
                .map_err(|e| format!("Invalid AUTO_UNJAIL: {e}"))?,

            data_dir: std::env::var("CONSENSUS_DATA_DIR")
                .map(PathBuf::from)
                .unwrap_or_else(|_| PathBuf::from("./consensus-data")),

            log_level: std::env::var("LOG_LEVEL")
                .unwrap_or_else(|_| "info".to_string()),
        })
    }

    /// Validate configuration
    ///
    /// # Errors
    ///
    /// Returns error if configuration is invalid
    pub fn validate(&self) -> Result<(), String> {
        if self.blocks_per_rotation == 0 {
            return Err("blocks_per_rotation must be > 0".to_string());
        }

        if self.timeout_blocks == 0 {
            return Err("timeout_blocks must be > 0".to_string());
        }

        if self.timeout_blocks >= self.blocks_per_rotation {
            return Err(
                "timeout_blocks must be < blocks_per_rotation".to_string()
            );
        }

        if self.max_attestation_age == 0 {
            return Err("max_attestation_age must be > 0".to_string());
        }

        Ok(())
    }
}

impl Default for ConsensusConfig {
    fn default() -> Self {
        Self {
            rpc_url: "http://localhost:8545".to_string(),
            ws_url: "ws://localhost:8546".to_string(),
            consensus_contract: Address::ZERO,
            coordinator_contract: Address::ZERO,
            registry_contract: Address::ZERO,
            sequencer_address: Address::ZERO,
            private_key_path: PathBuf::from("./sequencer.key"),
            chain_id: 6174,
            blocks_per_rotation: 100,
            timeout_blocks: 10,
            block_time: Duration::from_secs(2),
            sync_interval: Duration::from_secs(30),
            metrics_addr: "0.0.0.0:9090".to_string(),
            strict_validation: true,
            max_attestation_age: 100,
            auto_unjail: true,
            data_dir: PathBuf::from("./consensus-data"),
            log_level: "info".to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config_is_valid() {
        let config = ConsensusConfig::default();
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_invalid_blocks_per_rotation() {
        let mut config = ConsensusConfig::default();
        config.blocks_per_rotation = 0;
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_timeout_must_be_less_than_rotation() {
        let mut config = ConsensusConfig::default();
        config.timeout_blocks = config.blocks_per_rotation;
        assert!(config.validate().is_err());
    }
}
