//! Optimized REVM configuration for ANDE Chain
//!
//! This module provides performance-optimized EVM configurations based on:
//! - Reth v1.8.2 best practices
//! - REVM 29.0.1 optimization techniques
//! - Production deployment requirements

use alloy_primitives::U256;
use revm::primitives::hardfork::SpecId;
use revm::context::CfgEnv;  // Reth 1.8.2 / REVM 29 correct path

/// Default maximum bytes for txpool selection (1.85 MiB)
pub const DEFAULT_MAX_TXPOOL_BYTES: u64 = 1_939_865;

/// Default maximum gas for txpool selection (30M gas)
pub const DEFAULT_MAX_TXPOOL_GAS: u64 = 30_000_000;

/// Get the current block gas limit (configured globally)
pub const fn current_block_gas_limit() -> u64 {
    DEFAULT_MAX_TXPOOL_GAS
}

/// Evolve-specific configuration
#[derive(Debug, Clone)]
pub struct EvolveConfig {
    /// Maximum bytes for transaction pool selection
    pub max_txpool_bytes: u64,
    /// Maximum gas for transaction pool selection
    pub max_txpool_gas: u64,
}

impl Default for EvolveConfig {
    fn default() -> Self {
        Self {
            max_txpool_bytes: DEFAULT_MAX_TXPOOL_BYTES,
            max_txpool_gas: DEFAULT_MAX_TXPOOL_GAS,
        }
    }
}

/// Optimized EVM configuration for production use
#[derive(Debug, Clone)]
pub struct AndeEvmConfig {
    /// Chain ID (6174 for ANDE)
    pub chain_id: u64,
    /// Specification ID
    pub spec_id: SpecId,
    /// Gas limit for blocks
    pub block_gas_limit: u64,
    /// Base fee configuration
    pub base_fee_config: BaseFeeConfig,
    /// Performance optimizations enabled
    pub optimizations: PerformanceOptimizations,
    /// Cached CfgEnv for performance
    cached_cfg: Option<CfgEnv>,
}

#[derive(Debug, Clone)]
pub struct BaseFeeConfig {
    /// Minimum base fee (7 wei as per EIP-1559)
    pub min_base_fee: U256,
    /// EIP-1559 elasticity multiplier
    pub elasticity_multiplier: u64,
    /// EIP-1559 denominator
    pub base_fee_max_change_denominator: u64,
}

#[derive(Debug, Clone)]
pub struct PerformanceOptimizations {
    /// Enable aggressive inlining for hot paths
    pub aggressive_inlining: bool,
    /// Disable balance checks for known safe operations
    pub skip_balance_checks_for_system: bool,
    /// Enable parallel precompile execution
    pub parallel_precompiles: bool,
    /// Cache compiled bytecode
    pub bytecode_caching: bool,
}

impl Default for AndeEvmConfig {
    fn default() -> Self {
        Self::production()
    }
}

impl AndeEvmConfig {
    /// Production-optimized configuration
    #[inline]
    pub fn production() -> Self {
        Self {
            chain_id: 6174,
            spec_id: SpecId::CANCUN,
            block_gas_limit: 30_000_000,
            base_fee_config: BaseFeeConfig {
                min_base_fee: U256::from_limbs([7, 0, 0, 0]),
                elasticity_multiplier: 2,
                base_fee_max_change_denominator: 8,
            },
            optimizations: PerformanceOptimizations {
                aggressive_inlining: true,
                skip_balance_checks_for_system: true,
                parallel_precompiles: true,
                bytecode_caching: true,
            },
            cached_cfg: None,
        }
    }

    /// Development configuration (less optimized, more checks)
    #[inline]
    pub fn development() -> Self {
        Self {
            chain_id: 6174,
            spec_id: SpecId::CANCUN,
            block_gas_limit: 30_000_000,
            base_fee_config: BaseFeeConfig {
                min_base_fee: U256::from_limbs([7, 0, 0, 0]),
                elasticity_multiplier: 2,
                base_fee_max_change_denominator: 8,
            },
            optimizations: PerformanceOptimizations {
                aggressive_inlining: false,
                skip_balance_checks_for_system: false,
                parallel_precompiles: false,
                bytecode_caching: true,
            },
            cached_cfg: None,
        }
    }

    /// Create REVM CfgEnv from AndeEvmConfig
    ///
    /// Uses REVM 29 builder pattern following Reth 1.8.2 best practices
    #[inline(always)]
    pub fn to_cfg_env(&self) -> CfgEnv {
        // REVM 29 uses builder pattern: CfgEnv::new().with_chain_id().with_spec()
        let mut cfg = CfgEnv::new()
            .with_chain_id(self.chain_id)
            .with_spec(self.spec_id);

        // Configure contract code size limit (24KB - EIP-170)
        cfg.limit_contract_initcode_size = Some(0x6000);

        // Keep security checks enabled for production
        cfg.disable_nonce_check = false;

        cfg
    }

    /// Get chain ID
    #[inline(always)]
    pub const fn chain_id(&self) -> u64 {
        self.chain_id
    }

    /// Get spec ID
    #[inline(always)]
    pub const fn spec_id(&self) -> SpecId {
        self.spec_id
    }

    /// Check if optimizations are enabled
    #[inline(always)]
    pub const fn is_optimized(&self) -> bool {
        self.optimizations.aggressive_inlining
    }
}

/// Get a mutable reference to build and cache the CfgEnv
impl AndeEvmConfig {
    /// Get or build cached CfgEnv for maximum performance
    pub fn get_cfg_env(&mut self) -> &CfgEnv {
        if self.cached_cfg.is_none() {
            self.cached_cfg = Some(self.to_cfg_env());
        }
        self.cached_cfg.as_ref().unwrap()
    }

    /// Invalidate the cached configuration (e.g., after spec changes)
    pub fn invalidate_cache(&mut self) {
        self.cached_cfg = None;
    }
}

/// EVM configuration builder for flexible setup
pub struct AndeEvmConfigBuilder {
    config: AndeEvmConfig,
}

impl AndeEvmConfigBuilder {
    /// Start with production defaults
    pub fn production() -> Self {
        Self {
            config: AndeEvmConfig::production(),
        }
    }

    /// Start with development defaults
    pub fn development() -> Self {
        Self {
            config: AndeEvmConfig::development(),
        }
    }

    /// Set chain ID
    pub fn chain_id(mut self, chain_id: u64) -> Self {
        self.config.chain_id = chain_id;
        self
    }

    /// Set spec ID
    pub fn spec_id(mut self, spec_id: SpecId) -> Self {
        self.config.spec_id = spec_id;
        self
    }

    /// Set block gas limit
    pub fn block_gas_limit(mut self, limit: u64) -> Self {
        self.config.block_gas_limit = limit;
        self
    }

    /// Enable all performance optimizations
    pub fn max_performance(mut self) -> Self {
        self.config.optimizations = PerformanceOptimizations {
            aggressive_inlining: true,
            skip_balance_checks_for_system: true,
            parallel_precompiles: true,
            bytecode_caching: true,
        };
        self
    }

    /// Disable all optimizations (for testing)
    pub fn safe_mode(mut self) -> Self {
        self.config.optimizations = PerformanceOptimizations {
            aggressive_inlining: false,
            skip_balance_checks_for_system: false,
            parallel_precompiles: false,
            bytecode_caching: false,
        };
        self
    }

    /// Build the configuration
    pub fn build(self) -> AndeEvmConfig {
        self.config
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_production_config() {
        let config = AndeEvmConfig::production();
        assert_eq!(config.chain_id, 6174);
        assert_eq!(config.spec_id, SpecId::CANCUN);
        assert!(config.is_optimized());
    }

    #[test]
    fn test_development_config() {
        let config = AndeEvmConfig::development();
        assert_eq!(config.chain_id, 6174);
        assert!(!config.is_optimized());
    }

    #[test]
    fn test_builder() {
        let config = AndeEvmConfigBuilder::production()
            .chain_id(12345)
            .block_gas_limit(50_000_000)
            .build();
        
        assert_eq!(config.chain_id, 12345);
        assert_eq!(config.block_gas_limit, 50_000_000);
    }

    #[test]
    fn test_cfg_env_conversion() {
        let config = AndeEvmConfig::production();
        let cfg_env = config.to_cfg_env();

        assert_eq!(cfg_env.chain_id, 6174);
        assert_eq!(cfg_env.spec, SpecId::CANCUN);
        assert!(!cfg_env.disable_nonce_check);
    }
}
