//! Transaction types

use alloy_primitives::{Address, Bytes, U256};
use serde::{Deserialize, Serialize};

/// Ande transaction
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Transaction {
    /// Nonce
    pub nonce: u64,
    /// Gas price
    pub gas_price: U256,
    /// Gas limit
    pub gas_limit: u64,
    /// To address
    pub to: Option<Address>,
    /// Value
    pub value: U256,
    /// Data
    pub data: Bytes,
    /// Chain ID
    pub chain_id: u64,
}
