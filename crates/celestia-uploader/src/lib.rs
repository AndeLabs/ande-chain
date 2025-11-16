//! Celestia Uploader for ANDE Chain
//!
//! Production-grade Celestia blob uploader for genesis plant data.
//! Supports Matcha v6 (128 MB blocks, 6s block time).
//!
//! # Architecture
//!
//! ```text
//! ┌─────────────────────────────────────────────────────────┐
//! │                   ANDE Genesis Data                     │
//! │              520 plants × ~100 KB = 70 MB               │
//! └─────────────────────────────────────────────────────────┘
//!                            │
//!                            ▼
//!              ┌─────────────────────────┐
//!              │   PlantChunker          │
//!              │   5 plants/blob         │
//!              │   Target: 500 KB        │
//!              └─────────────────────────┘
//!                            │
//!                            ▼
//!              ┌─────────────────────────┐
//!              │   140 CelestiaBlobs     │
//!              │   - 104 plant metadata  │
//!              │   - 10 SVG templates    │
//!              │   - 26 supplementary    │
//!              └─────────────────────────┘
//!                            │
//!                            ▼
//!              ┌─────────────────────────┐
//!              │  ParallelUploader       │
//!              │  8 concurrent workers   │
//!              │  Exponential backoff    │
//!              └─────────────────────────┘
//!                            │
//!                            ▼
//!              ┌─────────────────────────┐
//!              │  Celestia Mainnet       │
//!              │  Matcha v6 (128 MB)     │
//!              │  Namespace: andeplants  │
//!              └─────────────────────────┘
//!                            │
//!                            ▼
//!              ┌─────────────────────────┐
//!              │  DataAvailabilityVerifier│
//!              │  Sampling + Proofs      │
//!              └─────────────────────────┘
//! ```
//!
//! # Example
//!
//! ```no_run
//! use celestia_uploader::*;
//!
//! #[tokio::main]
//! async fn main() -> Result<()> {
//!     // Configure Celestia client
//!     let config = CelestiaConfig {
//!         node_url: "http://localhost:26658".to_string(),
//!         auth_token: Some("your_auth_token".to_string()),
//!         network: "mainnet".to_string(),
//!         gas_multiplier: 1.1,
//!         timeout_ms: 30_000,
//!     };
//!
//!     let client = CelestiaClient::new(config)?;
//!
//!     // Load plant data
//!     let plants = load_plant_metadata("plants.json")?;
//!
//!     // Chunk into blobs
//!     let chunker = PlantChunker::new(plants);
//!     let blobs = chunker.chunk_plants()?;
//!
//!     // Upload in parallel
//!     let uploader = ParallelUploader::new(client, 8, 3);
//!     let results = uploader.upload_all(blobs).await?;
//!
//!     // Generate report
//!     let report = uploader.generate_report(&results);
//!     report.print_summary();
//!     report.save_to_file("upload_report.json")?;
//!
//!     Ok(())
//! }
//! ```

pub mod chunker;
pub mod client;
pub mod types;
pub mod uploader;
pub mod verifier;

pub use chunker::PlantChunker;
pub use client::CelestiaClient;
pub use types::*;
pub use uploader::{ParallelUploader, UploadReport};
pub use verifier::{DataAvailabilityVerifier, VerificationReport};

/// Initialize tracing for logging
pub fn init_logging() {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive(tracing::Level::INFO.into()),
        )
        .init();
}

#[cfg(test)]
mod integration_tests {
    use super::*;

    #[test]
    fn test_namespace_encoding() {
        let namespace = NAMESPACE_ANDEPLANTS_V1;
        let hex = hex::encode(namespace);

        // Should start with "andeplants_v1" in hex
        assert!(hex.starts_with("616e6465706c616e74735f7631"));
        assert_eq!(hex.len(), 58); // 29 bytes = 58 hex chars
    }
}
