//! Celestia RPC client for blob submission
//!
//! Implements celestia-node v0.28.2 JSON-RPC API
//! Reference: https://docs.celestia.org/developers/node-api

use crate::types::*;
use reqwest::Client;
use serde_json::json;
use std::time::Duration;
use tracing::{debug, info, warn};

/// Celestia RPC client for Matcha v6
pub struct CelestiaClient {
    client: Client,
    config: CelestiaConfig,
}

impl CelestiaClient {
    /// Create a new Celestia client with configuration
    pub fn new(config: CelestiaConfig) -> Result<Self> {
        let client = Client::builder()
            .timeout(Duration::from_millis(config.timeout_ms))
            .build()
            .map_err(|e| UploaderError::NetworkError(e))?;

        Ok(Self { client, config })
    }

    /// Submit a blob to Celestia
    ///
    /// # Arguments
    /// * `namespace` - 29-byte namespace identifier
    /// * `data` - Blob data (max 1.97 MB, recommended 500 KB)
    ///
    /// # Returns
    /// Upload result with height, commitment, and gas info
    pub async fn submit_blob(&self, namespace: &[u8; 29], data: &[u8]) -> Result<UploadResult> {
        // Validate blob size
        if data.len() > 1_973_786 {
            return Err(UploaderError::BlobTooLarge(data.len(), 1_973_786));
        }

        if data.len() > MAX_BLOB_SIZE {
            warn!(
                "Blob size {} exceeds recommended {} bytes. Inclusion may be slower.",
                data.len(),
                MAX_BLOB_SIZE
            );
        }

        // Estimate gas
        let gas_limit = self.estimate_gas(data.len()).await?;
        let gas_price = MIN_GAS_PRICE_UTIA;

        debug!(
            "Submitting blob: size={} bytes, gas_limit={}, gas_price={} utia",
            data.len(),
            gas_limit,
            gas_price
        );

        // Encode to hex
        let hex_namespace = hex::encode(namespace);
        let hex_data = hex::encode(data);

        // Build JSON-RPC request
        let mut params = json!([
            [hex_namespace],
            [hex_data]
        ]);

        // Add gas price if using custom value
        if self.config.gas_multiplier != 1.0 {
            let adjusted_gas_price = gas_price * self.config.gas_multiplier;
            params = json!([
                [hex_namespace],
                [hex_data],
                {
                    "gas_limit": gas_limit,
                    "fee": (gas_limit as f64 * adjusted_gas_price).to_string()
                }
            ]);
        }

        let payload = json!({
            "jsonrpc": "2.0",
            "method": "blob.Submit",
            "params": params,
            "id": 1
        });

        // Submit via JSON-RPC
        let mut request = self.client.post(&self.config.node_url).json(&payload);

        if let Some(ref token) = self.config.auth_token {
            request = request.bearer_auth(token);
        }

        let response = request.send().await?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();
            return Err(UploaderError::RpcError(format!(
                "HTTP {}: {}",
                status, error_text
            )));
        }

        let json_response: serde_json::Value = response.json().await?;

        // Check for JSON-RPC error
        if let Some(error) = json_response.get("error") {
            return Err(UploaderError::RpcError(format!(
                "RPC error: {}",
                error
            )));
        }

        // Parse result
        let result = json_response
            .get("result")
            .ok_or_else(|| UploaderError::RpcError("Missing result field".to_string()))?;

        let height = result
            .get("height")
            .and_then(|h| h.as_u64())
            .ok_or_else(|| UploaderError::RpcError("Missing height in result".to_string()))?;

        let commitment = result
            .get("commitment")
            .and_then(|c| c.as_str())
            .ok_or_else(|| UploaderError::RpcError("Missing commitment in result".to_string()))?
            .to_string();

        let tx_hash = result
            .get("tx_hash")
            .and_then(|h| h.as_str())
            .unwrap_or("unknown")
            .to_string();

        let gas_used = result
            .get("gas_used")
            .and_then(|g| g.as_u64())
            .unwrap_or(gas_limit);

        let fee_tia = (gas_used as f64 * gas_price) / 1_000_000.0; // Convert utia to TIA

        info!(
            "Blob submitted successfully: height={}, commitment={}, gas_used={}, fee={} TIA",
            height, commitment, gas_used, fee_tia
        );

        Ok(UploadResult {
            height,
            commitment,
            tx_hash,
            gas_used,
            fee_tia,
            chunk_index: 0,
            timestamp: chrono::Utc::now().timestamp() as u64,
        })
    }

    /// Estimate gas for blob submission using Celestia formula
    ///
    /// Formula: FC + Σ[SSN(Bi) × SS × GCPBB]
    /// - FC (Fixed Cost): 65,000 gas
    /// - SSN (Sparse Share Number): ceil(blob_size / share_size)
    /// - SS (Share Size): 512 bytes
    /// - GCPBB (Gas Cost Per Blob Byte): 8
    ///
    /// Additional 10,000 gas for first transaction (sequence == 0)
    async fn estimate_gas(&self, data_size: usize) -> Result<u64> {
        // Calculate sparse shares needed
        let sparse_shares = ((data_size as f64 / SHARE_SIZE_BYTES as f64).ceil()) as u64;

        // Apply formula
        let gas_limit = FIXED_COST_GAS + (sparse_shares * SHARE_SIZE_BYTES * GAS_PER_BYTE);

        // Add buffer for safety
        let buffered_gas = (gas_limit as f64 * self.config.gas_multiplier) as u64;

        debug!(
            "Gas estimation: data_size={}, sparse_shares={}, base_gas={}, buffered_gas={}",
            data_size, sparse_shares, gas_limit, buffered_gas
        );

        Ok(buffered_gas)
    }

    /// Retrieve a blob by height and commitment
    pub async fn get_blob(
        &self,
        height: u64,
        namespace: &[u8; 29],
        commitment: &str,
    ) -> Result<Vec<u8>> {
        let hex_namespace = hex::encode(namespace);

        let payload = json!({
            "jsonrpc": "2.0",
            "method": "blob.Get",
            "params": [height, hex_namespace, commitment],
            "id": 1
        });

        let mut request = self.client.post(&self.config.node_url).json(&payload);

        if let Some(ref token) = self.config.auth_token {
            request = request.bearer_auth(token);
        }

        let response = request.send().await?;
        let json_response: serde_json::Value = response.json().await?;

        if let Some(error) = json_response.get("error") {
            return Err(UploaderError::RpcError(format!(
                "Get blob error: {}",
                error
            )));
        }

        let result = json_response
            .get("result")
            .ok_or_else(|| UploaderError::RpcError("Missing result".to_string()))?;

        let hex_data = result
            .get("data")
            .and_then(|d| d.as_str())
            .ok_or_else(|| UploaderError::RpcError("Missing data in blob".to_string()))?;

        let data = hex::decode(hex_data)
            .map_err(|e| UploaderError::RpcError(format!("Invalid hex data: {}", e)))?;

        Ok(data)
    }

    /// Get current chain head height
    pub async fn get_height(&self) -> Result<u64> {
        let payload = json!({
            "jsonrpc": "2.0",
            "method": "header.LocalHead",
            "params": [],
            "id": 1
        });

        let mut request = self.client.post(&self.config.node_url).json(&payload);

        if let Some(ref token) = self.config.auth_token {
            request = request.bearer_auth(token);
        }

        let response = request.send().await?;
        let json_response: serde_json::Value = response.json().await?;

        if let Some(error) = json_response.get("error") {
            return Err(UploaderError::RpcError(format!(
                "Get height error: {}",
                error
            )));
        }

        let result = json_response
            .get("result")
            .ok_or_else(|| UploaderError::RpcError("Missing result".to_string()))?;

        let height = result
            .get("header")
            .and_then(|h| h.get("height"))
            .and_then(|h| h.as_str())
            .and_then(|h| h.parse::<u64>().ok())
            .ok_or_else(|| UploaderError::RpcError("Invalid height format".to_string()))?;

        Ok(height)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_gas_estimation() {
        let config = CelestiaConfig::default();
        let client = CelestiaClient::new(config).unwrap();

        // Test gas estimation for 500 KB blob
        let gas = client.estimate_gas(500_000).await.unwrap();

        // Expected: 65,000 + (976 * 512 * 8) = 65,000 + 4,001,792 = 4,066,792
        // With 10% buffer: ~4,473,471
        assert!(gas > 4_000_000 && gas < 5_000_000);
    }

    #[test]
    fn test_namespace_constant() {
        let namespace_str = std::str::from_utf8(&NAMESPACE_ANDEPLANTS_V1[..13]).unwrap();
        assert_eq!(namespace_str, "andeplants_v1");
        assert_eq!(NAMESPACE_ANDEPLANTS_V1.len(), 29);
    }
}
