//! # ANDE RPC
//!
//! **Status: PARTIAL IMPLEMENTATION**
//!
//! This crate provides RPC infrastructure for ANDE Chain.
//!
//! ## Implemented Features
//!
//! - **Rate Limiting**: DDoS protection with per-IP and global limits
//!   - Per-IP request quotas
//!   - Burst capacity
//!   - Auto-ban for abusive IPs
//!   - Method-specific rate limits
//!
//! ## Planned Features
//!
//! - Custom ANDE RPC methods
//! - Token Duality balance queries
//! - MEV statistics endpoints
//! - Staking information
//! - Sequencer status
//!
//! ## Current Implementation
//!
//! The main RPC server is provided by Reth's infrastructure. This crate
//! adds ANDE-specific middleware and extensions.
//!
//! ## Usage
//!
//! ```rust,ignore
//! use ande_rpc::rate_limiter::RpcRateLimiter;
//!
//! let limiter = RpcRateLimiter::new();
//! let result = limiter.check_rate_limit(ip_addr, Some("eth_call"));
//! ```
//!
//! ## Module Structure
//!
//! - `rate_limiter`: Request rate limiting and DDoS protection

#![warn(missing_docs)]

pub mod rate_limiter;

pub use rate_limiter::{RpcRateLimiter, RateLimitConfig, RateLimitError, RateLimitStats};
