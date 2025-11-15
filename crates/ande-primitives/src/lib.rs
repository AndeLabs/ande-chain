//! # Ande Primitives
//!
//! Core types and primitives for Ande Chain.

#![warn(missing_docs, unreachable_pub, unused_crate_dependencies)]
#![deny(unused_must_use, rust_2018_idioms)]

pub mod block;
pub mod transaction;
pub mod consensus;
pub mod precompile;

pub use alloy_primitives::{Address, Bytes, B256, U256};

/// Ande Chain ID
pub const ANDE_CHAIN_ID: u64 = 1337;

/// Precompile address for token duality
pub const ANDE_TOKEN_DUALITY_ADDRESS: Address = Address::new([
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x10, 0x00,
]);
