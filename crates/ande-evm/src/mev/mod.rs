//! MEV Integration Module for AndeChain
//!
//! This module provides MEV detection, auction integration, and
//! revenue distribution capabilities for the Parallel EVM.

pub mod detector;
pub mod auction;
pub mod distributor;
pub mod types;

pub use detector::{MevDetector, MevOpportunity, MevType};
pub use auction::{MevAuctionClient, BundleSubmission};
pub use distributor::{MevDistributorClient, EpochData};
pub use types::{MevMetrics, MevConfig};
