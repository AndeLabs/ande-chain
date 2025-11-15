//! Parallel EVM Configuration
//!
//! Configuration options for parallel transaction execution in AndeChain.

use std::num::NonZeroUsize;
use serde::{Deserialize, Serialize};

/// Configuration for parallel execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParallelConfig {
    /// Number of worker threads for parallel execution
    pub concurrency_level: NonZeroUsize,
    /// Enable lazy updates for ANDE precompile and beneficiary
    pub enable_lazy_updates: bool,
    /// Maximum number of retries for failed transactions
    pub max_retries: usize,
    /// Minimum number of transactions required to use parallel execution
    pub min_transactions_for_parallel: usize,
    /// Force fallback to sequential execution
    pub force_sequential: bool,
    /// Enable advanced dependency analysis
    pub enable_advanced_dependency_analysis: bool,
    /// Maximum number of dependent transactions per group
    pub max_dependency_depth: usize,
    /// Enable performance monitoring
    pub enable_monitoring: bool,
}

impl Default for ParallelConfig {
    fn default() -> Self {
        Self {
            concurrency_level: NonZeroUsize::new(8).unwrap(),
            enable_lazy_updates: true,
            max_retries: 3,
            min_transactions_for_parallel: 4,
            force_sequential: false,
            enable_advanced_dependency_analysis: false, // Phase 1: keep simple
            max_dependency_depth: 10,
            enable_monitoring: true,
        }
    }
}

impl ParallelConfig {
    /// Create a new configuration with default values
    pub fn new() -> Self {
        Self::default()
    }

    /// Create configuration optimized for high throughput
    pub fn high_throughput() -> Self {
        Self {
            concurrency_level: NonZeroUsize::new(16).unwrap(),
            enable_lazy_updates: true,
            max_retries: 5,
            min_transactions_for_parallel: 2,
            force_sequential: false,
            enable_advanced_dependency_analysis: true,
            max_dependency_depth: 20,
            enable_monitoring: true,
        }
    }

    /// Create configuration for low latency
    pub fn low_latency() -> Self {
        Self {
            concurrency_level: NonZeroUsize::new(4).unwrap(),
            enable_lazy_updates: true,
            max_retries: 2,
            min_transactions_for_parallel: 3,
            force_sequential: false,
            enable_advanced_dependency_analysis: false,
            max_dependency_depth: 5,
            enable_monitoring: true,
        }
    }

    /// Create configuration for testing
    pub fn testing() -> Self {
        Self {
            concurrency_level: NonZeroUsize::new(2).unwrap(),
            enable_lazy_updates: false, // Simplified for testing
            max_retries: 1,
            min_transactions_for_parallel: 2,
            force_sequential: false,
            enable_advanced_dependency_analysis: false,
            max_dependency_depth: 3,
            enable_monitoring: false,
        }
    }

    /// Create configuration with sequential execution forced
    pub fn sequential_only() -> Self {
        Self {
            concurrency_level: NonZeroUsize::new(1).unwrap(),
            enable_lazy_updates: false,
            max_retries: 1,
            min_transactions_for_parallel: usize::MAX, // Never use parallel
            force_sequential: true,
            enable_advanced_dependency_analysis: false,
            max_dependency_depth: 1,
            enable_monitoring: false,
        }
    }

    /// Validate configuration
    pub fn validate(&self) -> Result<(), String> {
        if self.concurrency_level.get() == 0 {
            return Err("Concurrency level must be at least 1".to_string());
        }

        if self.min_transactions_for_parallel == 0 {
            return Err("Minimum transactions for parallel must be at least 1".to_string());
        }

        if self.max_retries == 0 {
            return Err("Max retries must be at least 1".to_string());
        }

        if self.max_dependency_depth == 0 {
            return Err("Max dependency depth must be at least 1".to_string());
        }

        Ok(())
    }

    /// Get configuration as environment-friendly format
    pub fn to_env_format(&self) -> Vec<(&'static str, String)> {
        vec![
            ("ANDE_PARALLEL_CONCURRENCY_LEVEL", self.concurrency_level.get().to_string()),
            ("ANDE_PARALLEL_ENABLE_LAZY_UPDATES", self.enable_lazy_updates.to_string()),
            ("ANDE_PARALLEL_MAX_RETRIES", self.max_retries.to_string()),
            ("ANDE_PARALLEL_MIN_TRANSACTIONS", self.min_transactions_for_parallel.to_string()),
            ("ANDE_PARALLEL_FORCE_SEQUENTIAL", self.force_sequential.to_string()),
            ("ANDE_PARALLEL_ENABLE_ADVANCED_ANALYSIS", self.enable_advanced_dependency_analysis.to_string()),
            ("ANDE_PARALLEL_MAX_DEPENDENCY_DEPTH", self.max_dependency_depth.to_string()),
            ("ANDE_PARALLEL_ENABLE_MONITORING", self.enable_monitoring.to_string()),
        ]
    }

    /// Create configuration from environment variables
    pub fn from_env() -> Result<Self, String> {
        let concurrency_level = std::env::var("ANDE_PARALLEL_CONCURRENCY_LEVEL")
            .ok()
            .and_then(|s| s.parse::<usize>().ok())
            .and_then(|n| NonZeroUsize::new(n))
            .unwrap_or_else(|| NonZeroUsize::new(8).unwrap());

        let enable_lazy_updates = std::env::var("ANDE_PARALLEL_ENABLE_LAZY_UPDATES")
            .ok()
            .and_then(|s| s.parse::<bool>().ok())
            .unwrap_or(true);

        let max_retries = std::env::var("ANDE_PARALLEL_MAX_RETRIES")
            .ok()
            .and_then(|s| s.parse::<usize>().ok())
            .unwrap_or(3);

        let min_transactions_for_parallel = std::env::var("ANDE_PARALLEL_MIN_TRANSACTIONS")
            .ok()
            .and_then(|s| s.parse::<usize>().ok())
            .unwrap_or(4);

        let force_sequential = std::env::var("ANDE_PARALLEL_FORCE_SEQUENTIAL")
            .ok()
            .and_then(|s| s.parse::<bool>().ok())
            .unwrap_or(false);

        let enable_advanced_dependency_analysis = std::env::var("ANDE_PARALLEL_ENABLE_ADVANCED_ANALYSIS")
            .ok()
            .and_then(|s| s.parse::<bool>().ok())
            .unwrap_or(false);

        let max_dependency_depth = std::env::var("ANDE_PARALLEL_MAX_DEPENDENCY_DEPTH")
            .ok()
            .and_then(|s| s.parse::<usize>().ok())
            .unwrap_or(10);

        let enable_monitoring = std::env::var("ANDE_PARALLEL_ENABLE_MONITORING")
            .ok()
            .and_then(|s| s.parse::<bool>().ok())
            .unwrap_or(true);

        let config = Self {
            concurrency_level,
            enable_lazy_updates,
            max_retries,
            min_transactions_for_parallel,
            force_sequential,
            enable_advanced_dependency_analysis,
            max_dependency_depth,
            enable_monitoring,
        };

        config.validate()?;
        Ok(config)
    }

    /// Get human-readable description
    pub fn description(&self) -> String {
        format!(
            "ParallelConfig(concurrency={}, lazy={}, min_tx={}, retries={})",
            self.concurrency_level.get(),
            self.enable_lazy_updates,
            self.min_transactions_for_parallel,
            self.max_retries
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = ParallelConfig::default();
        assert_eq!(config.concurrency_level.get(), 8);
        assert!(config.enable_lazy_updates);
        assert_eq!(config.max_retries, 3);
        assert_eq!(config.min_transactions_for_parallel, 4);
        assert!(!config.force_sequential);
    }

    #[test]
    fn test_high_throughput_config() {
        let config = ParallelConfig::high_throughput();
        assert_eq!(config.concurrency_level.get(), 16);
        assert!(config.enable_lazy_updates);
        assert_eq!(config.min_transactions_for_parallel, 2);
        assert!(config.enable_advanced_dependency_analysis);
    }

    #[test]
    fn test_low_latency_config() {
        let config = ParallelConfig::low_latency();
        assert_eq!(config.concurrency_level.get(), 4);
        assert_eq!(config.min_transactions_for_parallel, 3);
        assert!(!config.enable_advanced_dependency_analysis);
    }

    #[test]
    fn test_config_validation() {
        let config = ParallelConfig::default();

        // Valid config should pass
        assert!(config.validate().is_ok());

        // Note: Can't test invalid concurrency_level because NonZeroUsize type prevents it
        // Instead, test other validation cases

        // Invalid min_transactions_for_parallel
        let mut invalid_config = ParallelConfig::default();
        invalid_config.min_transactions_for_parallel = 0;
        assert!(invalid_config.validate().is_err());

        // Invalid max_retries
        let mut invalid_config2 = ParallelConfig::default();
        invalid_config2.max_retries = 0;
        assert!(invalid_config2.validate().is_err());
    }

    #[test]
    fn test_should_use_parallel() {
        let config = ParallelConfig::default();

        // Should use parallel with enough transactions
        assert!(config.min_transactions_for_parallel <= 10);

        // Sequential only config
        let sequential_config = ParallelConfig::sequential_only();
        assert!(sequential_config.force_sequential);
        assert_eq!(sequential_config.min_transactions_for_parallel, usize::MAX);
    }
}