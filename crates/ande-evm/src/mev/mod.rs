//! MEV (Maximal Extractable Value) detection and fair distribution for ANDE Chain
//!
//! This module implements MEV redistribution following evstack's handler pattern
//! but enhanced with MEV type detection and classification.
//!
//! ## Components
//!
//! - `redirect`: MEV redirect policy and detection
//! - `handler`: Execution handler with MEV interception
//! - `config`: MEV configuration from environment (TODO)
//!
//! ## Usage
//!
//! ```rust,ignore
//! use ande_evm::mev::{AndeMevRedirect, AndeHandler, MevType};
//!
//! // Create redirect to distribution contract
//! let mev_sink = address!("0x...");
//! let redirect = AndeMevRedirect::with_default_threshold(mev_sink);
//!
//! // Create handler with MEV redirect
//! let handler = AndeHandler::new(Some(redirect));
//!
//! // Use handler in EVM execution...
//! ```

pub mod redirect;
pub mod handler;

pub use redirect::{AndeMevRedirect, MevDetection, MevRedirectError, MevType};
pub use handler::AndeHandler;
