//! Consensus types

use serde::{Deserialize, Serialize};

/// Consensus configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsensusConfig {
    /// Block time in seconds
    pub block_time: u64,
    /// Minimum validators
    pub min_validators: u64,
}

impl Default for ConsensusConfig {
    fn default() -> Self {
        Self {
            block_time: 12,
            min_validators: 4,
        }
    }
}
