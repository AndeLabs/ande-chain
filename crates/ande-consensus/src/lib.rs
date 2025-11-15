//! # ANDE Consensus - Production Decentralized Consensus
//!
//! This module implements the complete consensus mechanism for ANDE Chain,
//! including validator set management, proposer selection, block attestation,
//! and Byzantine fault tolerance.
//!
//! ## Architecture
//!
//! ```text
//! ┌──────────────────────────────────────────────────────────┐
//! │                   Consensus Engine                        │
//! ├──────────────────────────────────────────────────────────┤
//! │                                                            │
//! │  ┌─────────────────┐  ┌──────────────────┐              │
//! │  │ Validator Set   │  │ Contract Client  │              │
//! │  │ Management      │←→│ (on-chain sync)  │              │
//! │  └─────────────────┘  └──────────────────┘              │
//! │           ↓                     ↓                         │
//! │  ┌─────────────────┐  ┌──────────────────┐              │
//! │  │ Proposer        │  │ Block Validation │              │
//! │  │ Selection       │  │ & Attestation    │              │
//! │  └─────────────────┘  └──────────────────┘              │
//! │           ↓                     ↓                         │
//! │  ┌─────────────────────────────────────┐                │
//! │  │    BFT Finality Engine              │                │
//! │  │    (2/3+1 voting power threshold)   │                │
//! │  └─────────────────────────────────────┘                │
//! │                                                            │
//! └──────────────────────────────────────────────────────────┘
//! ```
//!
//! ## Features
//!
//! - **Weighted Round-Robin**: CometBFT-style proposer selection
//! - **BFT Finality**: 2/3+1 voting power threshold
//! - **Timeout Detection**: Automatic rotation on missed blocks
//! - **Slashing Integration**: Report invalid blocks on-chain
//! - **Metrics & Observability**: Prometheus metrics export
//! - **Production Ready**: Error handling, logging, testing

#![warn(missing_docs)]
#![warn(clippy::all)]
#![warn(clippy::pedantic)]
#![allow(clippy::module_name_repetitions)]

pub mod config;
pub mod contract_client;
pub mod engine;
pub mod error;
pub mod metrics;
pub mod types;
pub mod validator_set;

pub use config::ConsensusConfig;
pub use engine::ConsensusEngine;
pub use error::{ConsensusError, Result};
pub use types::{
    AttestationInfo, BlockProposal, EpochInfo, RotationInfo, ValidatorInfo, ValidatorSetUpdate,
};

/// Re-export commonly used types
pub mod prelude {
    pub use crate::{
        config::ConsensusConfig,
        engine::ConsensusEngine,
        error::{ConsensusError, Result},
        types::{AttestationInfo, BlockProposal, ValidatorInfo},
    };
}
