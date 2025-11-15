//! Error types for consensus module

use alloy_primitives::Address;
use thiserror::Error;

/// Result type for consensus operations
pub type Result<T> = std::result::Result<T, ConsensusError>;

/// Errors that can occur in the consensus module
#[derive(Debug, Error)]
pub enum ConsensusError {
    /// Invalid proposer for this block
    #[error("Invalid proposer: expected {expected}, got {actual}")]
    InvalidProposer {
        /// Expected proposer address
        expected: Address,
        /// Actual proposer address
        actual: Address,
    },

    /// Block signature verification failed
    #[error("Invalid block signature from {signer}")]
    InvalidSignature {
        /// Address that signed the block
        signer: Address,
    },

    /// Not enough voting power for finality
    #[error("Insufficient voting power: have {have}, need {need}")]
    InsufficientVotingPower {
        /// Voting power received
        have: u64,
        /// Voting power needed
        need: u64,
    },

    /// Validator not found in set
    #[error("Validator {0} not found in validator set")]
    ValidatorNotFound(Address),

    /// Validator is not active
    #[error("Validator {0} is not active")]
    ValidatorNotActive(Address),

    /// Validator is jailed
    #[error("Validator {0} is jailed")]
    ValidatorJailed(Address),

    /// Timeout occurred
    #[error("Timeout: no blocks produced in {blocks} blocks")]
    Timeout {
        /// Number of blocks without production
        blocks: u64,
    },

    /// Contract interaction failed
    #[error("Contract error: {0}")]
    ContractError(String),

    /// RPC error
    #[error("RPC error: {0}")]
    RpcError(String),

    /// Configuration error
    #[error("Configuration error: {0}")]
    ConfigError(String),

    /// Invalid block number
    #[error("Invalid block number: expected {expected}, got {actual}")]
    InvalidBlockNumber {
        /// Expected block number
        expected: u64,
        /// Actual block number
        actual: u64,
    },

    /// Block already proposed
    #[error("Block {0} already proposed")]
    BlockAlreadyProposed(u64),

    /// Serialization/deserialization error
    #[error("Serialization error: {0}")]
    SerializationError(String),

    /// I/O error
    #[error("I/O error: {0}")]
    IoError(#[from] std::io::Error),

    /// Internal error
    #[error("Internal error: {0}")]
    Internal(String),
}

impl From<ethers::providers::ProviderError> for ConsensusError {
    fn from(err: ethers::providers::ProviderError) -> Self {
        Self::RpcError(err.to_string())
    }
}

impl From<ethers::contract::ContractError<ethers::providers::Provider<ethers::providers::Http>>>
    for ConsensusError
{
    fn from(
        err: ethers::contract::ContractError<ethers::providers::Provider<ethers::providers::Http>>,
    ) -> Self {
        Self::ContractError(err.to_string())
    }
}

impl From<serde_json::Error> for ConsensusError {
    fn from(err: serde_json::Error) -> Self {
        Self::SerializationError(err.to_string())
    }
}
