//! Type definitions for Celestia blob uploads
//!
//! Matcha v6 specifications:
//! - Max blob size: 1.97 MB (1,973,786 bytes)
//! - Recommended: 500 KB for fast inclusion
//! - Block time: 6 seconds
//! - Max TX size: 8 MB

use serde::{Deserialize, Serialize};
use std::fmt;

/// ANDE Plants v1 namespace (29 bytes)
/// "andeplants_v1" padded with zeros
pub const NAMESPACE_ANDEPLANTS_V1: [u8; 29] = [
    0x61, 0x6e, 0x64, 0x65, 0x70, 0x6c, 0x61, 0x6e, 0x74, 0x73, 0x5f, 0x76, 0x31, // "andeplants_v1"
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // padding
];

/// Maximum recommended blob size for fast inclusion (500 KB)
pub const MAX_BLOB_SIZE: usize = 500_000;

/// Number of plants per blob chunk
pub const PLANTS_PER_BLOB: usize = 5;

/// Gas constants from Celestia Matcha v6
pub const FIXED_COST_GAS: u64 = 65_000;
pub const FIRST_TX_EXTRA_GAS: u64 = 10_000;
pub const SHARE_SIZE_BYTES: u64 = 512;
pub const GAS_PER_BYTE: u64 = 8;
pub const MIN_GAS_PRICE_UTIA: f64 = 0.004;

/// Plant metadata structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlantMetadata {
    pub id: u16,
    pub seed: String,
    pub scientific_name: String,
    pub common_names: Vec<String>,
    pub genus: String,
    pub family: String,
    pub ncbi_code: String,
    pub conservation: String,
    pub rarity: u8,
    pub properties: PlantProperties,
    pub traits: PlantTraits,
    pub metadata: PlantExtendedMetadata,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlantProperties {
    pub medicinal: bool,
    pub psychoactive: bool,
    pub toxic: bool,
    pub endangered: bool,
    pub ceremonial: bool,
    pub cultivated: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlantTraits {
    pub height: u8,
    pub color: u8,
    pub potency: u8,
    pub resistance: u8,
    pub yield_trait: u8,
    pub aroma: u8,
    pub flower: u8,
    pub root: u8,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlantExtendedMetadata {
    pub description: String,
    pub uses: Vec<String>,
    pub compounds: Vec<String>,
    pub regions: Vec<String>,
    pub traditional_uses: Vec<String>,
}

/// Celestia blob structure with versioning and integrity verification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CelestiaBlob {
    /// Protocol version (semantic versioning)
    pub version: String,

    /// Namespace identifier
    pub namespace: String,

    /// Index of this chunk (0-based)
    pub chunk_index: u32,

    /// Total number of chunks in the set
    pub total_chunks: u32,

    /// Type of blob content
    pub blob_type: BlobType,

    /// Unix timestamp of creation
    pub timestamp: u64,

    /// Merkle root of all chunks for verification
    pub merkle_root: String,

    /// Actual data payload
    pub data: serde_json::Value,

    /// Optional ECDSA signature for authenticity
    pub signature: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BlobType {
    PlantMetadata,
    SvgTemplates,
    TraitDescriptions,
    ScientificReferences,
    IndexData,
    MerkleProofs,
}

impl fmt::Display for BlobType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            BlobType::PlantMetadata => write!(f, "plant_metadata"),
            BlobType::SvgTemplates => write!(f, "svg_templates"),
            BlobType::TraitDescriptions => write!(f, "trait_descriptions"),
            BlobType::ScientificReferences => write!(f, "scientific_references"),
            BlobType::IndexData => write!(f, "index_data"),
            BlobType::MerkleProofs => write!(f, "merkle_proofs"),
        }
    }
}

/// Result from successful blob upload
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UploadResult {
    /// Celestia block height where blob was included
    pub height: u64,

    /// Blob commitment (hash)
    pub commitment: String,

    /// Transaction hash
    pub tx_hash: String,

    /// Gas used
    pub gas_used: u64,

    /// Fee paid (in TIA)
    pub fee_tia: f64,

    /// Chunk index
    pub chunk_index: u32,

    /// Upload timestamp
    pub timestamp: u64,
}

/// Configuration for Celestia client
#[derive(Debug, Clone)]
pub struct CelestiaConfig {
    /// Celestia node RPC URL (light client or full node)
    pub node_url: String,

    /// Authentication token for node access
    pub auth_token: Option<String>,

    /// Network (mainnet, mocha-4, arabica-11)
    pub network: String,

    /// Gas price multiplier for safety (default: 1.1 = 10% buffer)
    pub gas_multiplier: f64,

    /// Timeout for RPC calls (milliseconds)
    pub timeout_ms: u64,
}

impl Default for CelestiaConfig {
    fn default() -> Self {
        Self {
            node_url: "http://localhost:26658".to_string(),
            auth_token: None,
            network: "mainnet".to_string(),
            gas_multiplier: 1.1,
            timeout_ms: 30_000,
        }
    }
}

/// Error types for uploader operations
#[derive(Debug, thiserror::Error)]
pub enum UploaderError {
    #[error("Blob size {0} exceeds maximum {1} bytes")]
    BlobTooLarge(usize, usize),

    #[error("Network error: {0}")]
    NetworkError(#[from] reqwest::Error),

    #[error("Serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),

    #[error("Celestia RPC error: {0}")]
    RpcError(String),

    #[error("Gas estimation failed: {0}")]
    GasEstimationError(String),

    #[error("Invalid namespace: {0}")]
    InvalidNamespace(String),

    #[error("Upload timeout after {0}ms")]
    Timeout(u64),

    #[error("Invalid configuration: {0}")]
    ConfigError(String),
}

pub type Result<T> = std::result::Result<T, UploaderError>;
