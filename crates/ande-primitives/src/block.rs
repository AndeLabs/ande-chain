//! Block types and structures

use alloy_primitives::{Address, B256};
use serde::{Deserialize, Serialize};

/// Ande block header
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BlockHeader {
    /// Parent hash
    pub parent_hash: B256,
    /// Block number
    pub number: u64,
    /// Timestamp
    pub timestamp: u64,
    /// Gas limit
    pub gas_limit: u64,
    /// Gas used
    pub gas_used: u64,
    /// Coinbase
    pub coinbase: Address,
    /// State root
    pub state_root: B256,
    /// Transactions root
    pub transactions_root: B256,
    /// Receipts root
    pub receipts_root: B256,
}
