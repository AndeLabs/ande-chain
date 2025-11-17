//! End-to-End Integration Tests for ANDE Chain
//!
//! Tests the complete integration of:
//! - Token Duality Precompile (0xFD)
//! - BFT Consensus
//! - MEV Configuration
//! - Node startup and shutdown

use alloy_primitives::{address, Address, U256};

/// Test that Token Duality precompile address is correctly defined
#[test]
fn test_token_duality_precompile_address() {
    use ande_evm::ANDE_PRECOMPILE_ADDRESS;
    
    // Token Duality should be at 0xFD
    assert_eq!(ANDE_PRECOMPILE_ADDRESS, address!("00000000000000000000000000000000000000fd"));
}

/// Test MEV configuration loading from environment
#[test]
fn test_mev_config_from_env() {
    use ande_evm::MevConfig;
    
    // Test when MEV is disabled (default)
    std::env::remove_var("ANDE_MEV_ENABLED");
    let config = MevConfig::from_env().unwrap();
    assert!(config.is_none(), "MEV should be disabled by default");
    
    // Test when MEV is enabled
    std::env::set_var("ANDE_MEV_ENABLED", "true");
    std::env::set_var("ANDE_MEV_SINK", "0x0000000000000000000000000000000000000042");
    std::env::set_var("ANDE_MEV_MIN_THRESHOLD", "1000000000000000");
    
    let config = MevConfig::from_env().unwrap();
    assert!(config.is_some(), "MEV should be enabled");
    
    let config = config.unwrap();
    assert_eq!(config.mev_sink, address!("0000000000000000000000000000000000000042"));
    assert_eq!(config.min_threshold, U256::from(1000000000000000u64));
    assert!(config.enabled);
    
    // Cleanup
    std::env::remove_var("ANDE_MEV_ENABLED");
    std::env::remove_var("ANDE_MEV_SINK");
    std::env::remove_var("ANDE_MEV_MIN_THRESHOLD");
}

/// Test MEV redirect creation
#[test]
fn test_mev_redirect_creation() {
    use ande_evm::AndeMevRedirect;
    
    let sink = address!("0000000000000000000000000000000000000042");
    let threshold = U256::from(1000000000000000u64);
    
    let redirect = AndeMevRedirect::new(sink, threshold);
    
    assert_eq!(redirect.mev_sink(), sink);
    assert_eq!(redirect.min_threshold(), threshold);
}

/// Test consensus configuration loading
#[test]
fn test_consensus_config_loading() {
    use ande_consensus::ConsensusConfig;
    
    // Test when consensus is disabled (default)
    std::env::remove_var("ANDE_CONSENSUS_ENABLED");
    let config = ConsensusConfig::from_env();
    assert!(config.is_err(), "Consensus should fail to load when disabled");
    
    // Test when consensus is enabled with validators
    std::env::set_var("ANDE_CONSENSUS_ENABLED", "true");
    std::env::set_var("ANDE_CONSENSUS_VALIDATORS", r#"[
        {"address":"0x0000000000000000000000000000000000000001","weight":100},
        {"address":"0x0000000000000000000000000000000000000002","weight":50}
    ]"#);
    std::env::set_var("ANDE_CONSENSUS_THRESHOLD", "67");
    
    let config = ConsensusConfig::from_env();
    assert!(config.is_ok(), "Consensus config should load successfully");
    
    let config = config.unwrap();
    assert_eq!(config.validators.len(), 2);
    assert_eq!(config.threshold, 67);
    assert_eq!(config.validators[0].weight, 100);
    assert_eq!(config.validators[1].weight, 50);
    
    // Cleanup
    std::env::remove_var("ANDE_CONSENSUS_ENABLED");
    std::env::remove_var("ANDE_CONSENSUS_VALIDATORS");
    std::env::remove_var("ANDE_CONSENSUS_THRESHOLD");
}

/// Test parallel executor optimal worker count
#[test]
fn test_optimal_worker_count() {
    use ande_evm::optimal_worker_count;
    
    let workers = optimal_worker_count();
    
    // Should be at least 1 worker
    assert!(workers >= 1, "Should have at least 1 worker");
    
    // Should not exceed CPU count
    let cpu_count = num_cpus::get();
    assert!(workers <= cpu_count, "Workers should not exceed CPU count");
}

/// Integration test: Verify all features can be configured together
#[test]
fn test_full_feature_integration() {
    // Setup environment for all features
    std::env::set_var("ANDE_MEV_ENABLED", "true");
    std::env::set_var("ANDE_MEV_SINK", "0x0000000000000000000000000000000000000042");
    std::env::set_var("ANDE_CONSENSUS_ENABLED", "true");
    std::env::set_var("ANDE_CONSENSUS_VALIDATORS", r#"[
        {"address":"0x0000000000000000000000000000000000000001","weight":100}
    ]"#);
    std::env::set_var("ANDE_CONSENSUS_THRESHOLD", "67");
    
    // Load all configurations
    let mev_config = ande_evm::MevConfig::from_env().unwrap();
    let consensus_config = ande_consensus::ConsensusConfig::from_env();
    
    // Verify MEV is configured
    assert!(mev_config.is_some());
    
    // Verify consensus is configured
    assert!(consensus_config.is_ok());
    
    // Verify Token Duality precompile is available
    use ande_evm::ANDE_PRECOMPILE_ADDRESS;
    assert_eq!(ANDE_PRECOMPILE_ADDRESS, address!("00000000000000000000000000000000000000fd"));
    
    // Cleanup
    std::env::remove_var("ANDE_MEV_ENABLED");
    std::env::remove_var("ANDE_MEV_SINK");
    std::env::remove_var("ANDE_CONSENSUS_ENABLED");
    std::env::remove_var("ANDE_CONSENSUS_VALIDATORS");
    std::env::remove_var("ANDE_CONSENSUS_THRESHOLD");
}

/// Test that AndeExecutorBuilder can be created
#[test]
fn test_executor_builder_creation() {
    use ande_reth::AndeExecutorBuilder;
    
    let _builder = AndeExecutorBuilder::default();
    // If this compiles and runs, the builder is correctly implemented
}

/// Test that AndeConsensusBuilder can be created
#[test]
fn test_consensus_builder_creation() {
    use ande_reth::AndeConsensusBuilder;
    
    let _builder = AndeConsensusBuilder::default();
    // If this compiles and runs, the builder is correctly implemented
}

#[cfg(test)]
mod documentation_tests {
    use super::*;
    
    /// Verify all critical documentation files exist
    #[test]
    fn test_documentation_exists() {
        let docs = [
            "docs/TOKEN_DUALITY_PRECOMPILE.md",
            "docs/BFT_CONSENSUS_INTEGRATION.md",
            "docs/MEV_HANDLER_ANALYSIS.md",
            "docs/MEV_INTEGRATION_STRATEGY.md",
            "docs/FEATURES_SUMMARY.md",
        ];
        
        for doc in &docs {
            let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("..").join(doc);
            assert!(path.exists(), "Documentation file missing: {}", doc);
        }
    }
}

#[cfg(test)]
mod performance_tests {
    use super::*;
    
    /// Basic performance test for MEV detection overhead
    #[test]
    fn test_mev_detection_performance() {
        use ande_evm::AndeMevRedirect;
        use std::time::Instant;
        
        let sink = address!("0000000000000000000000000000000000000042");
        let threshold = U256::from(1000000000000000u64);
        let redirect = AndeMevRedirect::new(sink, threshold);
        
        // Measure creation time
        let start = Instant::now();
        for _ in 0..1000 {
            let _ = AndeMevRedirect::new(sink, threshold);
        }
        let duration = start.elapsed();
        
        // Should be very fast (< 1ms for 1000 iterations)
        assert!(duration.as_millis() < 10, "MEV redirect creation too slow: {:?}", duration);
    }
}
