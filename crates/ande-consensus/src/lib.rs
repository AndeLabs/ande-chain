//! # Ande Consensus
//!
//! Consensus mechanism implementation.

#![warn(missing_docs)]

use ande_primitives::consensus::ConsensusConfig;

/// Consensus engine
pub struct ConsensusEngine {
    config: ConsensusConfig,
}

impl ConsensusEngine {
    /// Create new consensus engine
    pub fn new(config: ConsensusConfig) -> Self {
        Self { config }
    }
    
    /// Get configuration
    pub fn config(&self) -> &ConsensusConfig {
        &self.config
    }
}
