# Celestia Uploader

Production-grade Celestia blob uploader for ANDE Chain genesis plant data.

## Features

- ✅ **Matcha v6 Support**: Optimized for 128 MB blocks, 6s block time
- ✅ **Parallel Upload**: 8 concurrent workers with exponential backoff
- ✅ **Cost Optimization**: Targets 500 KB blobs for fast inclusion
- ✅ **Data Verification**: Built-in DA sampling and Merkle proofs
- ✅ **Comprehensive Logging**: Full tracing of upload progress
- ✅ **Error Recovery**: Automatic retry with configurable attempts

## Architecture

```
ANDE Genesis (520 plants, ~70 MB)
         ↓
  PlantChunker (5 plants/blob, 500 KB target)
         ↓
  140 CelestiaBlobs
         ↓
  ParallelUploader (8 workers, retry logic)
         ↓
  Celestia Mainnet (Matcha v6)
         ↓
  DataAvailabilityVerifier (sampling + proofs)
```

## Installation

Add to workspace `Cargo.toml`:

```toml
[workspace]
members = [
    "crates/celestia-uploader",
]
```

Build:

```bash
cargo build --release -p celestia-uploader
```

## Usage

### 1. Prepare Configuration

Create `config.json`:

```json
{
  "node_url": "http://localhost:26658",
  "auth_token": "your_celestia_node_auth_token",
  "network": "mainnet",
  "parallel_workers": 8,
  "retry_attempts": 3
}
```

### 2. Prepare Plant Data

Ensure your `plants.json` contains the metadata:

```json
[
  {
    "id": 1,
    "seed": "0x123...",
    "scientific_name": "Lophophora williamsii",
    "common_names": ["Peyote"],
    "genus": "Lophophora",
    "family": "Cactaceae",
    "ncbi_code": "NC030453",
    "conservation": "VU",
    "rarity": 5,
    "properties": {...},
    "traits": {...},
    "metadata": {...}
  },
  ...
]
```

### 3. Dry Run (Estimate Cost)

```bash
cargo run --bin upload-genesis -- \
  --plants plants.json \
  --config config.json \
  --dry-run
```

Output:
```
Estimated Fee:     2.33 TIA
Estimated Cost:    $11.65 USD (at $5/TIA)
```

### 4. Upload to Celestia

```bash
cargo run --bin upload-genesis -- \
  --plants plants.json \
  --config config.json \
  --output upload_report.json \
  --verify
```

### 5. Review Upload Report

The report contains:

```json
{
  "total_blobs": 140,
  "total_gas_used": 582540000,
  "total_fee_tia": 2.330160,
  "total_fee_usd": 11.65,
  "height_range": [1234567, 1234750],
  "namespace": "616e6465706c616e74735f763100000000000000000000000000",
  "timestamp": 1731628800,
  "results": [
    {
      "height": 1234567,
      "commitment": "0xabc...",
      "tx_hash": "0xdef...",
      "gas_used": 4161000,
      "fee_tia": 0.016644,
      "chunk_index": 0,
      "timestamp": 1731628800
    },
    ...
  ]
}
```

## Library Usage

```rust
use celestia_uploader::*;

#[tokio::main]
async fn main() -> Result<()> {
    // Configure client
    let config = CelestiaConfig {
        node_url: "http://localhost:26658".to_string(),
        auth_token: Some("token".to_string()),
        network: "mainnet".to_string(),
        gas_multiplier: 1.1,
        timeout_ms: 30_000,
    };

    let client = CelestiaClient::new(config)?;

    // Load plants
    let plants = load_plants("plants.json")?;

    // Chunk
    let chunker = PlantChunker::new(plants);
    let blobs = chunker.chunk_plants()?;

    // Upload
    let uploader = ParallelUploader::new(client, 8, 3);
    let results = uploader.upload_all(blobs).await?;

    // Verify
    let verifier = DataAvailabilityVerifier::new(client);
    let verification = verifier.verify_all(&results).await?;

    if verification.is_complete() {
        println!("✓ All blobs verified!");
    }

    Ok(())
}
```

## Cost Breakdown

### Matcha v6 Mainnet (Nov 2025)

- **Min Gas Price**: 0.004 utia
- **Gas per 500 KB blob**: ~4,161,000 gas
- **Fee per blob**: ~0.016644 TIA
- **Total (140 blobs)**: ~2.33 TIA ≈ $10-$50 USD

### Compared to Pure On-Chain

- **On-chain storage (2,080 slots)**: $1,500-$3,000 USD
- **Celestia DA**: $10-$50 USD
- **Total hybrid cost**: $1,510-$3,050 USD
- **Savings vs pure on-chain**: 85%

## Technical Specifications

### Celestia Matcha v6

- Max blob size: 1.97 MB (1,973,786 bytes)
- Recommended: 500 KB for fast inclusion
- Block time: 6 seconds
- Max TX size: 8 MB
- Min gas price: 0.004 utia

### Gas Formula

```
Gas = FC + Σ[SSN(Bi) × SS × GCPBB]

Where:
- FC (Fixed Cost): 65,000 gas
- SSN (Sparse Share Number): ceil(blob_size / share_size)
- SS (Share Size): 512 bytes
- GCPBB (Gas Cost Per Blob Byte): 8
```

### Namespace

```
Namespace ID: 0x616e6465706c616e74735f763100000000000000000000000000
              "andeplants_v1" (29 bytes, zero-padded)
```

## Testing

Run unit tests:

```bash
cargo test -p celestia-uploader
```

Run integration tests (requires running celestia-node):

```bash
cargo test -p celestia-uploader --features integration-tests
```

## Monitoring

### View blobs on Celenium Explorer

```
https://celenium.io/namespace/<namespace_hex>
```

For ANDE Plants:
```
https://celenium.io/namespace/616e6465706c616e74735f763100000000000000000000000000
```

### Query blob via RPC

```bash
curl -X POST http://localhost:26658 \
  -H "Content-Type: application/json" \
  -d '{
    "jsonrpc": "2.0",
    "method": "blob.Get",
    "params": [<height>, "<namespace_hex>", "<commitment>"],
    "id": 1
  }'
```

## Troubleshooting

### "Blob too large" error

Reduce `PLANTS_PER_BLOB` in `src/chunker.rs`:

```rust
pub const PLANTS_PER_BLOB: usize = 4; // Reduced from 5
```

### "Gas estimation failed"

Increase gas multiplier in config:

```json
{
  "gas_multiplier": 1.2
}
```

### "Timeout" errors

Increase timeout in config:

```json
{
  "timeout_ms": 60000
}
```

## License

MIT

## Authors

ANDE Labs <dev@andelabs.io>
