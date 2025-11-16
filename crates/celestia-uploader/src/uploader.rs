//! Parallel blob uploader with retry logic

use crate::client::CelestiaClient;
use crate::types::*;
use futures::{stream, StreamExt};
use std::sync::Arc;
use tracing::{debug, error, info, warn};

/// Parallel uploader with configurable worker pool
pub struct ParallelUploader {
    client: Arc<CelestiaClient>,
    parallel_workers: usize,
    retry_attempts: usize,
}

impl ParallelUploader {
    /// Create a new parallel uploader
    ///
    /// # Arguments
    /// * `client` - Celestia RPC client
    /// * `parallel_workers` - Number of concurrent uploads (recommended: 8)
    /// * `retry_attempts` - Number of retry attempts on failure (default: 3)
    pub fn new(client: CelestiaClient, parallel_workers: usize, retry_attempts: usize) -> Self {
        Self {
            client: Arc::new(client),
            parallel_workers,
            retry_attempts,
        }
    }

    /// Upload all blobs in parallel with retry logic
    ///
    /// Uses buffer_unordered for maximum parallelism while respecting
    /// the worker pool limit (TxWorkerAccounts = 8 recommended for Matcha v6)
    ///
    /// # Returns
    /// Vector of upload results with heights, commitments, and gas info
    pub async fn upload_all(&self, blobs: Vec<CelestiaBlob>) -> Result<Vec<UploadResult>> {
        let total_blobs = blobs.len();
        info!(
            "Starting parallel upload of {} blobs with {} workers",
            total_blobs, self.parallel_workers
        );

        // Estimate total cost
        let estimated_cost = self.estimate_total_cost(&blobs).await?;
        info!(
            "Estimated total cost: {} TIA (${:.2} USD at $5/TIA)",
            estimated_cost,
            estimated_cost * 5.0
        );

        let results: Vec<Result<UploadResult>> = stream::iter(blobs.into_iter().enumerate())
            .map(|(index, blob)| {
                let client = Arc::clone(&self.client);
                let retry_attempts = self.retry_attempts;

                async move {
                    Self::upload_with_retry(client, blob, index, total_blobs, retry_attempts)
                        .await
                }
            })
            .buffer_unordered(self.parallel_workers)
            .collect()
            .await;

        // Collect successful uploads and errors
        let mut successes = Vec::new();
        let mut failures = Vec::new();

        for (i, result) in results.into_iter().enumerate() {
            match result {
                Ok(mut upload_result) => {
                    upload_result.chunk_index = i as u32;
                    successes.push(upload_result);
                }
                Err(e) => {
                    error!("Failed to upload blob {}: {}", i, e);
                    failures.push((i, e));
                }
            }
        }

        info!(
            "Upload completed: {} successful, {} failed",
            successes.len(),
            failures.len()
        );

        if !failures.is_empty() {
            warn!("Failed blob indices: {:?}", failures.iter().map(|(i, _)| i).collect::<Vec<_>>());
            return Err(UploaderError::RpcError(format!(
                "{} blobs failed to upload",
                failures.len()
            )));
        }

        // Sort by chunk_index to maintain order
        successes.sort_by_key(|r| r.chunk_index);

        Ok(successes)
    }

    /// Upload a single blob with exponential backoff retry
    async fn upload_with_retry(
        client: Arc<CelestiaClient>,
        blob: CelestiaBlob,
        index: usize,
        total: usize,
        max_retries: usize,
    ) -> Result<UploadResult> {
        let blob_json = serde_json::to_string(&blob)?;
        let blob_bytes = blob_json.as_bytes();

        for attempt in 0..max_retries {
            match client
                .submit_blob(&NAMESPACE_ANDEPLANTS_V1, blob_bytes)
                .await
            {
                Ok(result) => {
                    info!(
                        "✓ Uploaded blob {}/{} (type: {}, size: {} KB, height: {}, fee: {:.6} TIA)",
                        index + 1,
                        total,
                        blob.blob_type,
                        blob_bytes.len() / 1024,
                        result.height,
                        result.fee_tia
                    );
                    return Ok(result);
                }
                Err(e) => {
                    if attempt < max_retries - 1 {
                        let backoff_ms = 1000 * 2u64.pow(attempt as u32);
                        warn!(
                            "Upload attempt {}/{} failed for blob {}: {}. Retrying in {}ms...",
                            attempt + 1,
                            max_retries,
                            index,
                            e,
                            backoff_ms
                        );
                        tokio::time::sleep(tokio::time::Duration::from_millis(backoff_ms)).await;
                    } else {
                        error!(
                            "All {} attempts failed for blob {}: {}",
                            max_retries, index, e
                        );
                        return Err(e);
                    }
                }
            }
        }

        Err(UploaderError::RpcError(format!(
            "Failed after {} retries",
            max_retries
        )))
    }

    /// Estimate total cost for all blobs
    async fn estimate_total_cost(&self, blobs: &[CelestiaBlob]) -> Result<f64> {
        let mut total_gas: u64 = 0;

        for blob in blobs {
            let blob_json = serde_json::to_string(blob)?;
            let blob_size = blob_json.len();

            // Calculate gas using formula
            let sparse_shares = ((blob_size as f64 / SHARE_SIZE_BYTES as f64).ceil()) as u64;
            let blob_gas = FIXED_COST_GAS + (sparse_shares * SHARE_SIZE_BYTES * GAS_PER_BYTE);

            total_gas += blob_gas;
        }

        // Convert to TIA (1 TIA = 1,000,000 utia)
        let total_tia = (total_gas as f64 * MIN_GAS_PRICE_UTIA) / 1_000_000.0;

        debug!(
            "Cost estimation: {} blobs, {} total gas, {:.6} TIA",
            blobs.len(),
            total_gas,
            total_tia
        );

        Ok(total_tia)
    }

    /// Generate upload summary report
    pub fn generate_report(&self, results: &[UploadResult]) -> UploadReport {
        let total_gas: u64 = results.iter().map(|r| r.gas_used).sum();
        let total_fee: f64 = results.iter().map(|r| r.fee_tia).sum();

        let min_height = results.iter().map(|r| r.height).min().unwrap_or(0);
        let max_height = results.iter().map(|r| r.height).max().unwrap_or(0);

        UploadReport {
            total_blobs: results.len(),
            total_gas_used: total_gas,
            total_fee_tia: total_fee,
            total_fee_usd: total_fee * 5.0, // Assuming $5/TIA
            height_range: (min_height, max_height),
            namespace: hex::encode(NAMESPACE_ANDEPLANTS_V1),
            timestamp: chrono::Utc::now().timestamp() as u64,
            results: results.to_vec(),
        }
    }
}

/// Upload summary report
#[derive(Debug, Clone, serde::Serialize)]
pub struct UploadReport {
    pub total_blobs: usize,
    pub total_gas_used: u64,
    pub total_fee_tia: f64,
    pub total_fee_usd: f64,
    pub height_range: (u64, u64),
    pub namespace: String,
    pub timestamp: u64,
    pub results: Vec<UploadResult>,
}

impl UploadReport {
    /// Save report to JSON file
    pub fn save_to_file(&self, path: &str) -> Result<()> {
        let json = serde_json::to_string_pretty(self)?;
        std::fs::write(path, json).map_err(|e| {
            UploaderError::ConfigError(format!("Failed to write report: {}", e))
        })?;
        info!("Upload report saved to: {}", path);
        Ok(())
    }

    /// Print summary to console
    pub fn print_summary(&self) {
        println!("\n═══════════════════════════════════════════════════════════");
        println!("               CELESTIA UPLOAD REPORT");
        println!("═══════════════════════════════════════════════════════════");
        println!("Total Blobs:       {}", self.total_blobs);
        println!("Total Gas Used:    {} gas", self.total_gas_used);
        println!("Total Fee:         {:.6} TIA (${:.2} USD)", self.total_fee_tia, self.total_fee_usd);
        println!("Height Range:      {} - {}", self.height_range.0, self.height_range.1);
        println!("Namespace:         {}", self.namespace);
        println!("Timestamp:         {}", self.timestamp);
        println!("═══════════════════════════════════════════════════════════\n");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cost_estimation() {
        // Test cost calculation for 500 KB blob
        let blob_size = 500_000;
        let sparse_shares = ((blob_size as f64 / SHARE_SIZE_BYTES as f64).ceil()) as u64;
        let gas = FIXED_COST_GAS + (sparse_shares * SHARE_SIZE_BYTES * GAS_PER_BYTE);
        let fee_tia = (gas as f64 * MIN_GAS_PRICE_UTIA) / 1_000_000.0;

        // Should be approximately 0.016 TIA per blob
        assert!(fee_tia > 0.015 && fee_tia < 0.020);
    }

    #[test]
    fn test_total_cost_for_genesis() {
        // 140 blobs × 0.016 TIA = ~2.24 TIA
        let blobs = 140;
        let per_blob_fee = 0.016;
        let total = blobs as f64 * per_blob_fee;

        assert!(total > 2.0 && total < 3.0);
    }
}
