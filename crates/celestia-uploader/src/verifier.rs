//! Data availability verification and sampling

use crate::client::CelestiaClient;
use crate::types::*;
use sha2::{Digest, Sha256};
use std::sync::Arc;
use tracing::{debug, info};

/// DA verifier for Celestia blobs
pub struct DataAvailabilityVerifier {
    client: Arc<CelestiaClient>,
}

impl DataAvailabilityVerifier {
    /// Create a new DA verifier
    pub fn new(client: CelestiaClient) -> Self {
        Self {
            client: Arc::new(client),
        }
    }

    /// Verify a blob is available and matches expected commitment
    ///
    /// # Arguments
    /// * `height` - Block height where blob was included
    /// * `commitment` - Expected blob commitment (hash)
    ///
    /// # Returns
    /// True if blob is available and commitment matches
    pub async fn verify_blob(&self, height: u64, commitment: &str) -> Result<bool> {
        debug!("Verifying blob at height {} with commitment {}", height, commitment);

        // Retrieve blob from Celestia
        let blob_data = self
            .client
            .get_blob(height, &NAMESPACE_ANDEPLANTS_V1, commitment)
            .await?;

        // Compute commitment of retrieved data
        let computed_commitment = self.compute_commitment(&blob_data);

        let matches = computed_commitment == commitment;

        if matches {
            info!("✓ Blob verified successfully at height {}", height);
        } else {
            info!(
                "✗ Blob verification failed: expected {}, got {}",
                commitment, computed_commitment
            );
        }

        Ok(matches)
    }

    /// Verify all blobs from upload report
    pub async fn verify_all(&self, results: &[UploadResult]) -> Result<VerificationReport> {
        info!("Verifying {} blobs", results.len());

        let mut verified = 0;
        let mut failed = Vec::new();

        for result in results {
            match self.verify_blob(result.height, &result.commitment).await {
                Ok(true) => verified += 1,
                Ok(false) => {
                    failed.push((result.chunk_index, "Commitment mismatch".to_string()));
                }
                Err(e) => {
                    failed.push((result.chunk_index, e.to_string()));
                }
            }
        }

        let report = VerificationReport {
            total_blobs: results.len(),
            verified,
            failed: failed.len(),
            failure_details: failed,
        };

        info!(
            "Verification complete: {}/{} successful",
            verified,
            results.len()
        );

        Ok(report)
    }

    /// Compute blob commitment (SHA-256 hash)
    fn compute_commitment(&self, data: &[u8]) -> String {
        let mut hasher = Sha256::new();
        hasher.update(data);
        hex::encode(hasher.finalize())
    }

    /// Sample random blobs for DA verification
    ///
    /// This implements a basic DA sampling strategy:
    /// - Sample N random blobs from the set
    /// - Verify each sampled blob
    /// - If all samples pass, data is likely available
    pub async fn sample_blobs(
        &self,
        results: &[UploadResult],
        sample_size: usize,
    ) -> Result<bool> {
        use rand::seq::SliceRandom;
        use rand::thread_rng;

        let mut rng = thread_rng();
        let samples: Vec<_> = results
            .choose_multiple(&mut rng, sample_size.min(results.len()))
            .collect();

        info!("Sampling {} blobs for DA verification", samples.len());

        for sample in samples {
            if !self.verify_blob(sample.height, &sample.commitment).await? {
                return Ok(false);
            }
        }

        info!("All {} samples verified successfully", sample_size);
        Ok(true)
    }

    /// Verify Merkle proof for a specific plant
    ///
    /// This allows verification that a plant is included in the dataset
    /// without downloading all blobs
    pub fn verify_merkle_proof(
        &self,
        leaf_hash: &str,
        proof: &[String],
        root: &str,
        index: usize,
    ) -> bool {
        let mut current_hash = hex::decode(leaf_hash).unwrap_or_default();
        let mut current_index = index;

        for sibling in proof {
            let sibling_bytes = hex::decode(sibling).unwrap_or_default();

            let mut hasher = Sha256::new();

            if current_index % 2 == 0 {
                // Current node is left child
                hasher.update(&current_hash);
                hasher.update(&sibling_bytes);
            } else {
                // Current node is right child
                hasher.update(&sibling_bytes);
                hasher.update(&current_hash);
            }

            current_hash = hasher.finalize().to_vec();
            current_index /= 2;
        }

        let computed_root = hex::encode(current_hash);
        computed_root == root
    }
}

/// Verification report
#[derive(Debug, Clone, serde::Serialize)]
pub struct VerificationReport {
    pub total_blobs: usize,
    pub verified: usize,
    pub failed: usize,
    pub failure_details: Vec<(u32, String)>,
}

impl VerificationReport {
    /// Check if all blobs were verified successfully
    pub fn is_complete(&self) -> bool {
        self.failed == 0 && self.verified == self.total_blobs
    }

    /// Print verification summary
    pub fn print_summary(&self) {
        println!("\n═══════════════════════════════════════════════════════════");
        println!("           DATA AVAILABILITY VERIFICATION");
        println!("═══════════════════════════════════════════════════════════");
        println!("Total Blobs:       {}", self.total_blobs);
        println!("Verified:          {} ✓", self.verified);
        println!("Failed:            {} ✗", self.failed);

        if !self.failure_details.is_empty() {
            println!("\nFailure Details:");
            for (index, error) in &self.failure_details {
                println!("  Blob {}: {}", index, error);
            }
        }

        println!("═══════════════════════════════════════════════════════════\n");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_merkle_proof_verification() {
        // Simple test case: verify leaf in 4-leaf tree
        let verifier = DataAvailabilityVerifier {
            client: Arc::new(CelestiaClient::new(CelestiaConfig::default()).unwrap()),
        };

        // Leaf hashes (example)
        let leaf0 = "a".repeat(64);
        let leaf1 = "b".repeat(64);
        let leaf2 = "c".repeat(64);
        let leaf3 = "d".repeat(64);

        // For this test, we'd need to compute the actual Merkle tree
        // This is a simplified illustration of the verification logic

        // TODO: Add complete Merkle proof test with real tree construction
    }

    #[test]
    fn test_commitment_computation() {
        let verifier = DataAvailabilityVerifier {
            client: Arc::new(CelestiaClient::new(CelestiaConfig::default()).unwrap()),
        };

        let data = b"test data";
        let commitment = verifier.compute_commitment(data);

        // SHA-256 hash should be 64 hex characters
        assert_eq!(commitment.len(), 64);

        // Same data should produce same commitment
        let commitment2 = verifier.compute_commitment(data);
        assert_eq!(commitment, commitment2);
    }
}
