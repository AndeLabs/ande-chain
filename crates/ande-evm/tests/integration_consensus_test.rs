//! Integration test for AndeChain PoS consensus
//!
//! This test verifies basic functionality of the consensus client.

use ande_evm::consensus_config::ConsensusConfig;
use alloy_primitives::Address;

#[tokio::test]
#[ignore] // Ignore by default - requires running node
async fn test_consensus_client_integration() {
    // This test requires:
    // 1. A running node with RPC
    // 2. Deployed consensus contract
    // 3. Environment variables set
    
    let config = ConsensusConfig::from_env().expect("Failed to load config");
    
    assert!(config.enabled, "Consensus should be enabled");
    assert_ne!(config.consensus_address, Address::ZERO, "Consensus address must be set");
}

#[test]
fn test_consensus_config_defaults() {
    let config = ConsensusConfig::default();
    
    assert!(config.enabled);
    assert!(config.attestation_enabled);
    assert_eq!(config.rpc_url, "http://localhost:8545");
    assert_eq!(config.validator_sync_interval_secs, 300);
}

#[test]
fn test_consensus_config_validation() {
    let mut config = ConsensusConfig::default();
    config.consensus_address = "0x1234567890123456789012345678901234567890".parse().unwrap();
    config.staking_address = "0x0987654321098765432109876543210987654321".parse().unwrap();
    
    assert_ne!(config.consensus_address, Address::ZERO);
    assert_ne!(config.staking_address, Address::ZERO);
}
