//! Blob chunking and organization
//!
//! Handles splitting plant metadata into optimally-sized blobs

use crate::types::*;
use sha2::{Digest, Sha256};
use std::time::{SystemTime, UNIX_EPOCH};
use tracing::{debug, info};

/// Plant data chunker for Celestia blob organization
pub struct PlantChunker {
    plants: Vec<PlantMetadata>,
}

impl PlantChunker {
    /// Create a new chunker with plant data
    pub fn new(plants: Vec<PlantMetadata>) -> Self {
        Self { plants }
    }

    /// Chunk plants into optimally-sized blobs
    ///
    /// Strategy:
    /// - 5 plants per blob (target ~500 KB each)
    /// - Total 520 plants = 104 blobs
    /// - Additional blobs for templates, traits, references
    ///
    /// Returns vector of CelestiaBlob ready for upload
    pub fn chunk_plants(&self) -> Result<Vec<CelestiaBlob>> {
        info!(
            "Chunking {} plants into blobs (target size: {} KB)",
            self.plants.len(),
            MAX_BLOB_SIZE / 1024
        );

        let total_chunks = (self.plants.len() + PLANTS_PER_BLOB - 1) / PLANTS_PER_BLOB;
        let merkle_root = self.calculate_merkle_root()?;
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        let mut blobs = Vec::new();

        for (chunk_index, chunk) in self.plants.chunks(PLANTS_PER_BLOB).enumerate() {
            let blob = CelestiaBlob {
                version: "1.0.0".to_string(),
                namespace: "andeplants_v1".to_string(),
                chunk_index: chunk_index as u32,
                total_chunks: total_chunks as u32,
                blob_type: BlobType::PlantMetadata,
                timestamp,
                merkle_root: merkle_root.clone(),
                data: serde_json::json!({
                    "plants": chunk,
                }),
                signature: None,
            };

            // Verify size constraint
            let blob_json = serde_json::to_string(&blob)?;
            let blob_size = blob_json.len();

            if blob_size > MAX_BLOB_SIZE {
                return Err(UploaderError::BlobTooLarge(blob_size, MAX_BLOB_SIZE));
            }

            debug!(
                "Created blob {}/{}: {} plants, {} bytes",
                chunk_index + 1,
                total_chunks,
                chunk.len(),
                blob_size
            );

            blobs.push(blob);
        }

        info!(
            "Created {} blobs, total data size: ~{} MB",
            blobs.len(),
            (blobs.len() * MAX_BLOB_SIZE) / 1_000_000
        );

        Ok(blobs)
    }

    /// Calculate Merkle root of all plant seeds for verification
    fn calculate_merkle_root(&self) -> Result<String> {
        if self.plants.is_empty() {
            return Err(UploaderError::ConfigError(
                "No plants to calculate Merkle root".to_string(),
            ));
        }

        // Build leaf hashes from plant seeds
        let mut leaves: Vec<[u8; 32]> = self
            .plants
            .iter()
            .map(|plant| {
                let seed_bytes = hex::decode(&plant.seed).unwrap_or_default();
                let mut hasher = Sha256::new();
                hasher.update(&seed_bytes);
                let result = hasher.finalize();
                let mut hash = [0u8; 32];
                hash.copy_from_slice(&result);
                hash
            })
            .collect();

        // Build Merkle tree bottom-up
        while leaves.len() > 1 {
            let mut next_level = Vec::new();

            for i in (0..leaves.len()).step_by(2) {
                if i + 1 < leaves.len() {
                    // Hash pair
                    let mut hasher = Sha256::new();
                    hasher.update(&leaves[i]);
                    hasher.update(&leaves[i + 1]);
                    let result = hasher.finalize();
                    let mut hash = [0u8; 32];
                    hash.copy_from_slice(&result);
                    next_level.push(hash);
                } else {
                    // Odd one out - promote to next level
                    next_level.push(leaves[i]);
                }
            }

            leaves = next_level;
        }

        let root = hex::encode(leaves[0]);
        debug!("Calculated Merkle root: {}", root);

        Ok(root)
    }

    /// Create supplementary blobs for templates, traits, etc.
    pub fn create_supplementary_blobs(
        &self,
        svg_templates: Vec<String>,
        trait_descriptions: serde_json::Value,
        scientific_refs: serde_json::Value,
    ) -> Result<Vec<CelestiaBlob>> {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        let merkle_root = self.calculate_merkle_root()?;
        let mut blobs = Vec::new();
        let mut chunk_index = (self.plants.len() + PLANTS_PER_BLOB - 1) / PLANTS_PER_BLOB;

        // SVG templates blob
        let svg_blob = CelestiaBlob {
            version: "1.0.0".to_string(),
            namespace: "andeplants_v1".to_string(),
            chunk_index: chunk_index as u32,
            total_chunks: 0, // Will be updated later
            blob_type: BlobType::SvgTemplates,
            timestamp,
            merkle_root: merkle_root.clone(),
            data: serde_json::json!({
                "templates": svg_templates,
            }),
            signature: None,
        };
        blobs.push(svg_blob);
        chunk_index += 1;

        // Trait descriptions blob
        let traits_blob = CelestiaBlob {
            version: "1.0.0".to_string(),
            namespace: "andeplants_v1".to_string(),
            chunk_index: chunk_index as u32,
            total_chunks: 0,
            blob_type: BlobType::TraitDescriptions,
            timestamp,
            merkle_root: merkle_root.clone(),
            data: trait_descriptions,
            signature: None,
        };
        blobs.push(traits_blob);
        chunk_index += 1;

        // Scientific references blob
        let refs_blob = CelestiaBlob {
            version: "1.0.0".to_string(),
            namespace: "andeplants_v1".to_string(),
            chunk_index: chunk_index as u32,
            total_chunks: 0,
            blob_type: BlobType::ScientificReferences,
            timestamp,
            merkle_root,
            data: scientific_refs,
            signature: None,
        };
        blobs.push(refs_blob);

        Ok(blobs)
    }

    /// Generate Merkle proofs for individual plants
    pub fn generate_merkle_proofs(&self) -> Result<Vec<Vec<String>>> {
        // TODO: Implement Merkle proof generation for DA verification
        // Each proof allows verification that a specific plant is included
        // in the dataset without downloading the entire dataset

        Ok(Vec::new())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_plant(id: u16) -> PlantMetadata {
        PlantMetadata {
            id,
            seed: format!("{:064x}", id),
            scientific_name: format!("Planta testus {}", id),
            common_names: vec![format!("Test Plant {}", id)],
            genus: "Testus".to_string(),
            family: "Testaceae".to_string(),
            ncbi_code: format!("NC{:06}", id),
            conservation: "LC".to_string(),
            rarity: 1,
            properties: PlantProperties {
                medicinal: true,
                psychoactive: false,
                toxic: false,
                endangered: false,
                ceremonial: false,
                cultivated: true,
            },
            traits: PlantTraits {
                height: 100,
                color: 50,
                potency: 75,
                resistance: 80,
                yield_trait: 60,
                aroma: 40,
                flower: 55,
                root: 65,
            },
            metadata: PlantExtendedMetadata {
                description: "Test plant".to_string(),
                uses: vec!["Testing".to_string()],
                compounds: vec!["TestCompound".to_string()],
                regions: vec!["Test Region".to_string()],
                traditional_uses: vec!["Test use".to_string()],
            },
        }
    }

    #[test]
    fn test_chunking() {
        let plants: Vec<PlantMetadata> = (1..=25).map(create_test_plant).collect();
        let chunker = PlantChunker::new(plants);

        let blobs = chunker.chunk_plants().unwrap();

        // 25 plants / 5 per blob = 5 blobs
        assert_eq!(blobs.len(), 5);

        // Verify chunk indices
        for (i, blob) in blobs.iter().enumerate() {
            assert_eq!(blob.chunk_index, i as u32);
            assert_eq!(blob.total_chunks, 5);
        }
    }

    #[test]
    fn test_merkle_root_consistency() {
        let plants: Vec<PlantMetadata> = (1..=10).map(create_test_plant).collect();
        let chunker = PlantChunker::new(plants.clone());

        let root1 = chunker.calculate_merkle_root().unwrap();
        let root2 = chunker.calculate_merkle_root().unwrap();

        // Same input should produce same root
        assert_eq!(root1, root2);

        // Root should be 64 hex characters (32 bytes)
        assert_eq!(root1.len(), 64);
    }

    #[test]
    fn test_blob_size_constraint() {
        let plants: Vec<PlantMetadata> = (1..=520).map(create_test_plant).collect();
        let chunker = PlantChunker::new(plants);

        let blobs = chunker.chunk_plants().unwrap();

        // Verify all blobs are under size limit
        for blob in blobs {
            let blob_json = serde_json::to_string(&blob).unwrap();
            assert!(
                blob_json.len() <= MAX_BLOB_SIZE,
                "Blob size {} exceeds max {}",
                blob_json.len(),
                MAX_BLOB_SIZE
            );
        }
    }
}
