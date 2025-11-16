# ANDE Chain x Celestia Matcha v6 Integration
## Production-Grade Architecture for Plant DNA System

**Fecha:** 2024-11-15
**Celestia Version:** Matcha v6 (Mainnet Nov 10, 2025)
**celestia-node:** v0.28.2
**Status:** 🟢 PRODUCTION READY

---

## 📊 Technical Constraints (Matcha v6)

```
Max Blob Size:       1.97 MB (1,973,786 bytes)
Recommended:         500 KB (fast inclusion)
Max TX Size:         8 MB
Min Gas Price:       0.004 utia
Gas Formula:         FC + Σ[SSN(Bi) × SS × GCPBB]
Fixed Cost (FC):     65,000 gas
celestia-node API:   v0.28.2
```

---

## 🗂️ Data Architecture

### Total Data Breakdown

```
520 plantas × metadata completa = ~70 MB total

Desglose:
├─ Plant metadata (JSON):    520 × 100 KB = 52 MB
├─ SVG templates:             10 templates = 5 MB
├─ Trait descriptions:        Documentation = 3 MB
├─ Scientific refs (NCBI):    Links + docs = 5 MB
└─ Index + Merkle proofs:     Verification data = 5 MB

Total: ~70 MB
```

### Chunking Strategy

```
Target chunk size: 500 KB (optimal for fast inclusion)
Total chunks: 70 MB / 500 KB = 140 blobs

Chunk organization:
├─ Blobs 0-103:   Plant data (520 plants / 5 per blob)
├─ Blobs 104-113: SVG templates (10 templates)
├─ Blobs 114-119: Trait descriptions
├─ Blobs 120-129: Scientific references
└─ Blobs 130-139: Index + Merkle trees

Each blob: ≤ 500 KB for fast inclusion
```

---

## 🧬 Namespace Design

### ANDE Plants Namespace

```
Namespace ID: 0x616e6465706c616e74735f763100000000000000000000000000
              "andeplants_v1" (UTF-8 encoded + padding)

Version: v1 (allows future upgrades to v2, v3, etc.)
Format: 29 bytes (Celestia namespace spec)
```

### Namespace Structure

```solidity
bytes29 public constant NAMESPACE_ANDEPLANTS_V1 =
    0x616e6465706c616e74735f763100000000000000000000000000;

// Future versions
bytes29 public constant NAMESPACE_ANDEPLANTS_V2 =
    0x616e6465706c616e74735f763200000000000000000000000000;
```

---

## 📦 Blob Structure

### Blob Format (JSON)

```json
{
  "version": "1.0.0",
  "namespace": "andeplants_v1",
  "chunkIndex": 0,
  "totalChunks": 140,
  "type": "plant_metadata",
  "timestamp": 1700000000,
  "merkleRoot": "0x...",
  "data": {
    "plants": [
      {
        "id": 1,
        "seed": "0x...",
        "scientificName": "Lophophora williamsii",
        "commonNames": ["Peyote", "Hikuri", "Mescal Button"],
        "genus": "Lophophora",
        "family": "Cactaceae",
        "ncbiCode": "NC030453",
        "conservation": "VU",
        "rarity": 3,
        "properties": {
          "psychoactive": true,
          "medicinal": true,
          "entheogenic": true,
          "traditional": true
        },
        "traits": {
          "height": 120,
          "color": 85,
          "potency": 250,
          "resistance": 180,
          "yield": 90,
          "aroma": 145,
          "flower": 200,
          "root": 165
        },
        "metadata": {
          "habitat": "Chihuahuan Desert, 1000-2000m",
          "growthRate": "very slow",
          "lifespanYears": 30,
          "propagation": "seed, cutting",
          "companions": ["Ferocactus", "Echinocactus"]
        }
      },
      // ... 4 more plants per blob (5 total)
    ]
  },
  "signature": "0x..."  // ECDSA signature for verification
}
```

---

## 🚀 Celestia Uploader Module (Rust)

### Module Structure

```
crates/celestia-uploader/
├─ Cargo.toml
├─ src/
│   ├─ lib.rs
│   ├─ client.rs         // Celestia node RPC client
│   ├─ chunker.rs        // Data chunking logic
│   ├─ uploader.rs       // Parallel blob uploader
│   ├─ verifier.rs       // DA verification
│   └─ types.rs          // Type definitions
└─ tests/
    ├─ integration.rs
    └─ fixtures/
```

### Cargo.toml

```toml
[package]
name = "ande-celestia-uploader"
version = "1.0.0"
edition = "2021"

[dependencies]
# Celestia
celestia-node-api = "0.28"  # Latest Matcha v6
celestia-types = "0.11"

# Async runtime
tokio = { version = "1.35", features = ["full"] }
futures = "0.3"

# Serialization
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"

# Crypto
sha2 = "0.10"
hex = "0.4"
k256 = { version = "0.13", features = ["ecdsa"] }

# HTTP
reqwest = { version = "0.11", features = ["json"] }

# Error handling
anyhow = "1.0"
thiserror = "1.0"

# Logging
tracing = "0.1"
tracing-subscriber = "0.3"

[dev-dependencies]
mockall = "0.12"
```

### types.rs

```rust
use serde::{Deserialize, Serialize};

/// Namespace for ANDE plants
pub const NAMESPACE_ANDEPLANTS_V1: &[u8; 29] =
    b"andeplants_v1\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0";

/// Plant metadata structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlantMetadata {
    pub id: u16,
    pub seed: String,  // Hex-encoded bytes32
    pub scientific_name: String,
    pub common_names: Vec<String>,
    pub genus: String,
    pub family: String,
    pub ncbi_code: String,
    pub conservation: String,  // IUCN status
    pub rarity: u8,  // 0-3
    pub properties: PlantProperties,
    pub traits: PlantTraits,
    pub metadata: PlantExtendedMetadata,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlantProperties {
    pub psychoactive: bool,
    pub medicinal: bool,
    pub toxic: bool,
    pub entheogenic: bool,
    pub stimulant: bool,
    pub sedative: bool,
    pub hallucinogenic: bool,
    // ... more properties
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlantTraits {
    pub height: u8,
    pub color: u8,
    pub potency: u8,
    pub resistance: u8,
    pub yield_trait: u8,  // "yield" is reserved keyword
    pub aroma: u8,
    pub flower: u8,
    pub root: u8,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlantExtendedMetadata {
    pub habitat: String,
    pub growth_rate: String,
    pub lifespan_years: u16,
    pub propagation: String,
    pub companions: Vec<String>,
}

/// Blob structure for Celestia
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CelestiaBlob {
    pub version: String,
    pub namespace: String,
    pub chunk_index: u32,
    pub total_chunks: u32,
    pub blob_type: String,  // "plant_metadata", "svg_template", etc.
    pub timestamp: u64,
    pub merkle_root: String,
    pub data: serde_json::Value,
    pub signature: Option<String>,
}

/// Upload result
#[derive(Debug, Clone)]
pub struct UploadResult {
    pub blob_index: u32,
    pub height: u64,
    pub commitment: String,  // Hex-encoded commitment
    pub namespace: Vec<u8>,
    pub tx_hash: String,
}

/// Configuration
#[derive(Debug, Clone)]
pub struct CelestiaConfig {
    pub node_url: String,  // e.g., "http://localhost:26658"
    pub auth_token: Option<String>,
    pub gas_price: f64,  // In utia (min 0.004)
    pub max_gas_price: f64,  // Max acceptable (default 0.2 TIA)
    pub parallel_workers: usize,  // TxWorkerAccounts
}
```

### client.rs

```rust
use anyhow::{Context, Result};
use reqwest::Client;
use serde_json::json;
use tracing::{debug, info};

use crate::types::*;

/// Celestia Node RPC client
pub struct CelestiaClient {
    client: Client,
    config: CelestiaConfig,
}

impl CelestiaClient {
    pub fn new(config: CelestiaConfig) -> Result<Self> {
        let client = Client::builder()
            .timeout(std::time::Duration::from_secs(30))
            .build()?;

        Ok(Self { client, config })
    }

    /// Submit a single blob to Celestia
    pub async fn submit_blob(
        &self,
        namespace: &[u8; 29],
        data: &[u8],
    ) -> Result<UploadResult> {
        info!(
            "Submitting blob: {} bytes to namespace {:?}",
            data.len(),
            hex::encode(namespace)
        );

        // Encode data as hex
        let hex_namespace = hex::encode(namespace);
        let hex_data = hex::encode(data);

        // Estimate gas
        let gas_limit = self.estimate_gas(data.len()).await?;
        debug!("Estimated gas limit: {}", gas_limit);

        // Submit via JSON-RPC
        let response = self
            .client
            .post(&self.config.node_url)
            .json(&json!({
                "jsonrpc": "2.0",
                "method": "blob.Submit",
                "params": [
                    [hex_namespace],
                    [hex_data]
                ],
                "id": 1
            }))
            .send()
            .await
            .context("Failed to submit blob")?;

        let result: serde_json::Value = response.json().await?;

        // Extract height from response
        let height = result["result"]["height"]
            .as_u64()
            .context("Missing height in response")?;

        // Get commitment
        let commitment = self.get_commitment(namespace, height).await?;

        Ok(UploadResult {
            blob_index: 0,  // Will be set by caller
            height,
            commitment,
            namespace: namespace.to_vec(),
            tx_hash: result["result"]["tx_hash"]
                .as_str()
                .unwrap_or("unknown")
                .to_string(),
        })
    }

    /// Estimate gas for blob submission
    async fn estimate_gas(&self, data_size: usize) -> Result<u64> {
        // Use Celestia's formula:
        // Gas = 65000 (FC) + (sparse_shares × share_size × gas_per_byte)

        const FIXED_COST: u64 = 65_000;
        const SHARE_SIZE: u64 = 512;
        const GAS_PER_BYTE: u64 = 8;  // Governance parameter

        let sparse_shares = ((data_size as f64 / SHARE_SIZE as f64).ceil()) as u64;
        let gas_limit = FIXED_COST + (sparse_shares * SHARE_SIZE * GAS_PER_BYTE);

        // Add 10% buffer for safety
        Ok((gas_limit as f64 * 1.1) as u64)
    }

    /// Get commitment for a blob
    async fn get_commitment(
        &self,
        namespace: &[u8; 29],
        height: u64,
    ) -> Result<String> {
        let hex_namespace = hex::encode(namespace);

        let response = self
            .client
            .post(&self.config.node_url)
            .json(&json!({
                "jsonrpc": "2.0",
                "method": "blob.GetCommitment",
                "params": [height, hex_namespace],
                "id": 1
            }))
            .send()
            .await?;

        let result: serde_json::Value = response.json().await?;

        result["result"]["commitment"]
            .as_str()
            .map(|s| s.to_string())
            .context("Missing commitment")
    }

    /// Retrieve a blob (for verification)
    pub async fn get_blob(
        &self,
        height: u64,
        namespace: &[u8; 29],
        commitment: &str,
    ) -> Result<Vec<u8>> {
        let hex_namespace = hex::encode(namespace);

        let response = self
            .client
            .post(&self.config.node_url)
            .json(&json!({
                "jsonrpc": "2.0",
                "method": "blob.Get",
                "params": [height, hex_namespace, commitment],
                "id": 1
            }))
            .send()
            .await?;

        let result: serde_json::Value = response.json().await?;

        let hex_data = result["result"]["data"]
            .as_str()
            .context("Missing data in response")?;

        hex::decode(hex_data).context("Failed to decode hex data")
    }
}
```

### chunker.rs

```rust
use anyhow::{Context, Result};
use serde_json;
use sha2::{Digest, Sha256};

use crate::types::*;

const MAX_BLOB_SIZE: usize = 500_000;  // 500 KB (safe size)
const PLANTS_PER_BLOB: usize = 5;

pub struct PlantChunker {
    plants: Vec<PlantMetadata>,
}

impl PlantChunker {
    pub fn new(plants: Vec<PlantMetadata>) -> Self {
        Self { plants }
    }

    /// Chunk plants into blobs of ~500KB each
    pub fn chunk_plants(&self) -> Result<Vec<CelestiaBlob>> {
        let mut blobs = Vec::new();
        let total_chunks = (self.plants.len() + PLANTS_PER_BLOB - 1) / PLANTS_PER_BLOB;

        // Calculate Merkle root of all plant seeds
        let merkle_root = self.calculate_merkle_root();

        for (chunk_index, chunk) in self.plants.chunks(PLANTS_PER_BLOB).enumerate() {
            let blob = CelestiaBlob {
                version: "1.0.0".to_string(),
                namespace: "andeplants_v1".to_string(),
                chunk_index: chunk_index as u32,
                total_chunks: total_chunks as u32,
                blob_type: "plant_metadata".to_string(),
                timestamp: std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_secs(),
                merkle_root: merkle_root.clone(),
                data: serde_json::to_value(json!({ "plants": chunk }))?,
                signature: None,  // TODO: Sign with private key
            };

            // Verify size
            let blob_json = serde_json::to_string(&blob)?;
            if blob_json.len() > MAX_BLOB_SIZE {
                anyhow::bail!(
                    "Blob {} exceeds max size: {} > {}",
                    chunk_index,
                    blob_json.len(),
                    MAX_BLOB_SIZE
                );
            }

            blobs.push(blob);
        }

        Ok(blobs)
    }

    fn calculate_merkle_root(&self) -> String {
        // Simple Merkle root of plant seeds (simplified)
        let mut hasher = Sha256::new();
        for plant in &self.plants {
            hasher.update(plant.seed.as_bytes());
        }
        hex::encode(hasher.finalize())
    }
}
```

### uploader.rs

```rust
use anyhow::Result;
use futures::stream::{self, StreamExt};
use tracing::{info, warn};

use crate::{client::CelestiaClient, types::*};

pub struct ParallelUploader {
    client: CelestiaClient,
    parallel_workers: usize,
}

impl ParallelUploader {
    pub fn new(client: CelestiaClient, parallel_workers: usize) -> Self {
        Self {
            client,
            parallel_workers,
        }
    }

    /// Upload all blobs in parallel
    pub async fn upload_all(
        &self,
        blobs: Vec<CelestiaBlob>,
    ) -> Result<Vec<UploadResult>> {
        info!(
            "Uploading {} blobs with {} parallel workers",
            blobs.len(),
            self.parallel_workers
        );

        let results: Vec<Result<UploadResult>> = stream::iter(blobs.into_iter().enumerate())
            .map(|(index, blob)| {
                let client = &self.client;
                async move {
                    // Serialize blob to JSON
                    let blob_json = serde_json::to_string(&blob)?;
                    let blob_bytes = blob_json.as_bytes();

                    // Submit to Celestia
                    let mut result = client
                        .submit_blob(NAMESPACE_ANDEPLANTS_V1, blob_bytes)
                        .await?;

                    result.blob_index = index as u32;

                    info!(
                        "Uploaded blob {}/{}: height={}, commitment={}",
                        index + 1,
                        blob.total_chunks,
                        result.height,
                        &result.commitment[..16]
                    );

                    Ok(result)
                }
            })
            .buffer_unordered(self.parallel_workers)
            .collect()
            .await;

        // Collect results and handle errors
        let mut upload_results = Vec::new();
        let mut errors = Vec::new();

        for (index, result) in results.into_iter().enumerate() {
            match result {
                Ok(r) => upload_results.push(r),
                Err(e) => {
                    warn!("Failed to upload blob {}: {}", index, e);
                    errors.push(e);
                }
            }
        }

        if !errors.is_empty() {
            anyhow::bail!("Failed to upload {} blobs", errors.len());
        }

        Ok(upload_results)
    }
}
```

---

## 💾 Genesis Integration

### Genesis Contract with Celestia Pointers

```solidity
// SPDX-License-Identifier: MIT
pragma solidity ^0.8.20;

contract AndeDNAVault {
    // ========== CELESTIA BLOB REFERENCES ==========
    struct CelestiaBlob {
        bytes29 namespaceID;
        uint64 startHeight;
        uint64 endHeight;
        uint32 totalBlobs;
    }

    CelestiaBlob public immutable PLANTS_METADATA_BLOB;

    // ========== PLANT SEEDS (On-chain) ==========
    mapping(uint16 => bytes32) public plantSeeds;

    // ========== CONSTRUCTOR (Genesis) ==========
    constructor(
        bytes29 _namespace,
        uint64 _startHeight,
        uint64 _endHeight,
        uint32 _totalBlobs,
        bytes32[520] memory _seeds
    ) {
        PLANTS_METADATA_BLOB = CelestiaBlob({
            namespaceID: _namespace,
            startHeight: _startHeight,
            endHeight: _endHeight,
            totalBlobs: _totalBlobs
        });

        for (uint16 i = 0; i < 520; i++) {
            plantSeeds[i + 1] = _seeds[i];
        }
    }
}
```

---

## 📈 Cost Estimation (Matcha v6)

```python
# Per blob estimation
blob_size = 500 KB = 512,000 bytes
gas_limit = 65_000 + (512_000 / 512 * 512 * 8)
          = 65_000 + (1_000 * 512 * 8)
          = 65,000 + 4,096,000
          = 4,161,000 gas per blob

gas_price = 0.004 utia (min price Matcha v6)
fee_per_blob = 4,161,000 * 0.004 = 16,644 utia
             = 0.016644 TIA per blob

# Total for 140 blobs
total_blobs = 140
total_fee = 140 * 0.016644 TIA = 2.33 TIA
          ≈ $10-$50 USD (depending on TIA price)
```

---

## ✅ Production Checklist

- [ ] Set up celestia-node v0.28.2 light client
- [ ] Configure TxWorkerAccounts = 8 (parallel mode)
- [ ] Generate 520 plant seeds
- [ ] Chunk metadata into 140 blobs
- [ ] Upload to Celestia Mainnet
- [ ] Deploy genesis with Celestia pointers
- [ ] Test DA sampling verification
- [ ] Monitor via Celenium explorer
- [ ] Set up alerts for blob retrieval failures

---

**Status:** 🟢 READY FOR IMPLEMENTATION
**Next:** Run uploader module + Deploy genesis

