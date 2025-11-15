//! Core types for consensus module

use alloy_primitives::{Address, Bytes, B256, U256};
use serde::{Deserialize, Serialize};

/// Complete information about a validator
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ValidatorInfo {
    /// Validator's Ethereum address
    pub validator: Address,

    /// libp2p peer ID for P2P networking
    pub p2p_peer_id: B256,

    /// RPC endpoint URL
    pub rpc_endpoint: String,

    /// Amount of ANDE staked
    pub stake: U256,

    /// Voting power (stake + bonuses)
    pub power: u64,

    /// Accumulated priority for proposer selection (CometBFT algorithm)
    pub accumulated_priority: i64,

    /// Total number of blocks produced
    pub total_blocks_produced: u64,

    /// Total number of blocks missed
    pub total_blocks_missed: u64,

    /// Uptime in basis points (10000 = 100%)
    pub uptime: u16,

    /// Timestamp of last block produced
    pub last_block_produced: u64,

    /// Timestamp when validator was registered
    pub registered_at: u64,

    /// Whether the validator is jailed
    pub jailed: bool,

    /// Whether the validator is active
    pub active: bool,

    /// Whether this is a permanent genesis validator
    pub is_permanent: bool,
}

impl ValidatorInfo {
    /// Check if validator can propose blocks
    pub const fn can_propose(&self) -> bool {
        self.active && !self.jailed
    }

    /// Calculate this validator's contribution to BFT threshold
    pub const fn voting_power(&self) -> u64 {
        if self.can_propose() {
            self.power
        } else {
            0
        }
    }
}

/// Block proposal submitted by a validator
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlockProposal {
    /// Block number
    pub block_number: u64,

    /// Block hash
    pub block_hash: B256,

    /// Address of the block producer
    pub producer: Address,

    /// Producer's signature
    pub signature: Bytes,

    /// Timestamp of proposal
    pub timestamp: u64,

    /// Whether the proposal has been verified
    pub verified: bool,
}

/// Attestation (vote) for a block by a validator
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AttestationInfo {
    /// Validator submitting attestation
    pub validator: Address,

    /// Hash of the attested block
    pub block_hash: B256,

    /// Validator's signature
    pub signature: Bytes,

    /// Timestamp of attestation
    pub timestamp: u64,

    /// Voting power of this validator
    pub voting_power: u64,
}

/// Information about an epoch
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EpochInfo {
    /// Epoch number
    pub epoch_number: u64,

    /// Starting block number
    pub start_block: u64,

    /// Ending block number (0 if current)
    pub end_block: u64,

    /// Start timestamp
    pub start_time: u64,

    /// End timestamp (0 if current)
    pub end_time: u64,

    /// Active validators in this epoch
    pub validators: Vec<Address>,

    /// Total voting power in epoch
    pub total_voting_power: u64,
}

/// Information about a rotation period
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RotationInfo {
    /// Rotation number
    pub rotation_number: u64,

    /// Starting block
    pub start_block: u64,

    /// Ending block (0 if current)
    pub end_block: u64,

    /// Leader for this rotation
    pub leader: Address,

    /// Number of blocks successfully produced
    pub blocks_produced: u64,

    /// Number of blocks missed
    pub missed_blocks: u64,

    /// Whether rotation completed successfully
    pub completed_successfully: bool,
}

/// Update to the validator set
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidatorSetUpdate {
    /// Epoch number for this update
    pub epoch: u64,

    /// New validator addresses
    pub validators: Vec<Address>,

    /// Corresponding voting powers
    pub powers: Vec<u64>,

    /// New total voting power
    pub total_power: u64,

    /// Block number of the update
    pub block_number: u64,

    /// Timestamp of the update
    pub timestamp: u64,
}

impl ValidatorSetUpdate {
    /// Calculate BFT threshold (2/3 + 1)
    pub const fn bft_threshold(&self) -> u64 {
        (self.total_power * 2) / 3 + 1
    }
}

/// Consensus state snapshot
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsensusState {
    /// Current block number
    pub current_block: u64,

    /// Current epoch
    pub current_epoch: u64,

    /// Current rotation number
    pub current_rotation: u64,

    /// Current proposer address
    pub current_proposer: Address,

    /// Number of active validators
    pub active_validators: usize,

    /// Total voting power
    pub total_voting_power: u64,

    /// BFT threshold
    pub bft_threshold: u64,

    /// Timestamp of last state update
    pub last_update: u64,
}
