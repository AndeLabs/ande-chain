//! MEV Types and Configuration

use alloy_primitives::{Address, U256, B256};
use serde::{Deserialize, Serialize};
use std::time::Duration;

/// Configuration for MEV detection and integration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MevConfig {
    /// Enable MEV detection
    pub enable_detection: bool,
    /// Enable MEV auction integration
    pub enable_auction: bool,
    /// Enable MEV distribution
    pub enable_distribution: bool,
    /// MEV distributor contract address
    pub distributor_address: Option<Address>,
    /// MEV auction manager contract address
    pub auction_address: Option<Address>,
    /// Minimum MEV value to report (in wei)
    pub min_mev_value: U256,
    /// RPC endpoint for contract calls
    pub rpc_endpoint: String,
    /// Deposit interval for MEV distribution
    pub deposit_interval: Duration,
    /// Maximum MEV buffer before forcing deposit
    pub max_mev_buffer: U256,
}

impl Default for MevConfig {
    fn default() -> Self {
        Self {
            enable_detection: true,
            enable_auction: true,
            enable_distribution: true,
            distributor_address: None,
            auction_address: None,
            min_mev_value: U256::from(100_000_000_000_000_000u64), // 0.1 ANDE
            rpc_endpoint: "http://localhost:8545".to_string(),
            deposit_interval: Duration::from_secs(3600), // 1 hour
            max_mev_buffer: U256::from(1000) * U256::from(10u64.pow(18)), // 1000 ANDE
        }
    }
}

impl MevConfig {
    /// Validate configuration
    pub fn validate(&self) -> Result<(), String> {
        if self.enable_distribution && self.distributor_address.is_none() {
            return Err("MEV distribution enabled but no distributor address provided".to_string());
        }
        
        if self.enable_auction && self.auction_address.is_none() {
            return Err("MEV auction enabled but no auction address provided".to_string());
        }
        
        Ok(())
    }
}

/// MEV metrics for monitoring
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct MevMetrics {
    /// Total MEV captured (in wei)
    pub total_mev_captured: U256,
    /// Number of MEV opportunities detected
    pub opportunities_detected: u64,
    /// Number of bundles executed
    pub bundles_executed: u64,
    /// Total MEV distributed to stakers
    pub total_distributed: U256,
    /// Current epoch number
    pub current_epoch: u64,
    /// MEV captured in current epoch
    pub epoch_mev: U256,
    /// Average MEV per block
    pub avg_mev_per_block: U256,
    /// Number of failed bundle submissions
    pub failed_submissions: u64,
}

impl MevMetrics {
    /// Create new metrics instance
    pub fn new() -> Self {
        Self::default()
    }
    
    /// Record MEV opportunity
    pub fn record_opportunity(&mut self, value: U256) {
        self.opportunities_detected += 1;
        self.total_mev_captured += value;
        self.epoch_mev += value;
    }
    
    /// Record bundle execution
    pub fn record_bundle_execution(&mut self, value: U256) {
        self.bundles_executed += 1;
        self.total_mev_captured += value;
        self.epoch_mev += value;
    }
    
    /// Record failed submission
    pub fn record_failed_submission(&mut self) {
        self.failed_submissions += 1;
    }
    
    /// Start new epoch
    pub fn new_epoch(&mut self) {
        self.current_epoch += 1;
        self.epoch_mev = U256::ZERO;
    }
    
    /// Calculate average MEV per block
    pub fn calculate_avg_mev(&mut self, total_blocks: u64) {
        if total_blocks > 0 {
            self.avg_mev_per_block = self.total_mev_captured / U256::from(total_blocks);
        }
    }
}

/// Bundle information for auction
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BundleInfo {
    /// Bundle hash
    pub hash: B256,
    /// Searcher address
    pub searcher: Address,
    /// Bid amount
    pub bid_amount: U256,
    /// Target block number
    pub target_block: u64,
    /// Bundle transactions
    pub transactions: Vec<B256>,
    /// Estimated MEV value
    pub estimated_mev: U256,
}

/// Epoch information from distributor
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EpochInfo {
    /// Epoch number
    pub epoch_number: u64,
    /// Total MEV captured in epoch
    pub total_mev: U256,
    /// Stakers reward amount
    pub stakers_reward: U256,
    /// Protocol fee amount
    pub protocol_fee: U256,
    /// Treasury amount
    pub treasury_amount: U256,
    /// Whether epoch is settled
    pub settled: bool,
    /// Epoch end timestamp
    pub timestamp: u64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mev_config_default() {
        let config = MevConfig::default();
        assert!(config.enable_detection);
        assert!(config.enable_auction);
        assert!(config.enable_distribution);
    }

    #[test]
    fn test_mev_config_validation() {
        let mut config = MevConfig::default();
        
        // Should fail without distributor address
        assert!(config.validate().is_err());
        
        // Should pass with addresses
        config.distributor_address = Some(Address::ZERO);
        config.auction_address = Some(Address::ZERO);
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_mev_metrics_recording() {
        let mut metrics = MevMetrics::new();
        
        let value = U256::from(1_000_000_000_000_000_000u64); // 1 ANDE
        metrics.record_opportunity(value);
        
        assert_eq!(metrics.opportunities_detected, 1);
        assert_eq!(metrics.total_mev_captured, value);
        assert_eq!(metrics.epoch_mev, value);
    }

    #[test]
    fn test_mev_metrics_epoch() {
        let mut metrics = MevMetrics::new();
        
        let value = U256::from(1_000_000_000_000_000_000u64);
        metrics.record_opportunity(value);
        
        assert_eq!(metrics.current_epoch, 0);
        assert_eq!(metrics.epoch_mev, value);
        
        metrics.new_epoch();
        
        assert_eq!(metrics.current_epoch, 1);
        assert_eq!(metrics.epoch_mev, U256::ZERO);
        assert_eq!(metrics.total_mev_captured, value); // Total should persist
    }
}
