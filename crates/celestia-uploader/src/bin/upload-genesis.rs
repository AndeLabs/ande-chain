//! CLI tool to upload ANDE genesis plant data to Celestia
//!
//! Usage:
//!   cargo run --bin upload-genesis -- --plants plants.json --config config.json
//!
//! Configuration file format (config.json):
//! ```json
//! {
//!   "node_url": "http://localhost:26658",
//!   "auth_token": "your_auth_token",
//!   "network": "mainnet",
//!   "parallel_workers": 8,
//!   "retry_attempts": 3
//! }
//! ```

use celestia_uploader::*;
use clap::Parser;
use std::fs;
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(name = "upload-genesis")]
#[command(about = "Upload ANDE genesis plant data to Celestia Mainnet", long_about = None)]
struct Args {
    /// Path to plant metadata JSON file
    #[arg(short, long, value_name = "FILE")]
    plants: PathBuf,

    /// Path to configuration JSON file
    #[arg(short, long, value_name = "FILE")]
    config: PathBuf,

    /// Output path for upload report
    #[arg(short, long, value_name = "FILE", default_value = "upload_report.json")]
    output: PathBuf,

    /// Dry run (estimate cost only, don't upload)
    #[arg(long)]
    dry_run: bool,

    /// Verify uploaded blobs after upload
    #[arg(long)]
    verify: bool,
}

#[derive(Debug, serde::Deserialize)]
struct Config {
    node_url: String,
    auth_token: Option<String>,
    network: String,
    #[serde(default = "default_workers")]
    parallel_workers: usize,
    #[serde(default = "default_retries")]
    retry_attempts: usize,
}

fn default_workers() -> usize {
    8
}

fn default_retries() -> usize {
    3
}

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    init_logging();

    // Parse command line arguments
    let args = Args::parse();

    tracing::info!("ANDE Genesis Uploader - Celestia Matcha v6");
    tracing::info!("═══════════════════════════════════════════════════════════");

    // Load configuration
    let config_json = fs::read_to_string(&args.config).map_err(|e| {
        UploaderError::ConfigError(format!("Failed to read config file: {}", e))
    })?;

    let config: Config = serde_json::from_str(&config_json)?;

    tracing::info!("Configuration:");
    tracing::info!("  Network:           {}", config.network);
    tracing::info!("  Node URL:          {}", config.node_url);
    tracing::info!("  Parallel Workers:  {}", config.parallel_workers);
    tracing::info!("  Retry Attempts:    {}", config.retry_attempts);

    // Load plant metadata
    let plants_json = fs::read_to_string(&args.plants).map_err(|e| {
        UploaderError::ConfigError(format!("Failed to read plants file: {}", e))
    })?;

    let plants: Vec<PlantMetadata> = serde_json::from_str(&plants_json)?;

    tracing::info!("\nPlant Data:");
    tracing::info!("  Total Plants:      {}", plants.len());

    // Create Celestia client
    let celestia_config = CelestiaConfig {
        node_url: config.node_url,
        auth_token: config.auth_token,
        network: config.network,
        gas_multiplier: 1.1,
        timeout_ms: 30_000,
    };

    let client = CelestiaClient::new(celestia_config.clone())?;

    // Get current height
    let current_height = client.get_height().await?;
    tracing::info!("  Current Height:    {}", current_height);

    // Chunk plants into blobs
    tracing::info!("\n[1/4] Chunking plant data...");
    let chunker = PlantChunker::new(plants);
    let blobs = chunker.chunk_plants()?;

    tracing::info!("  Created {} blobs", blobs.len());

    // Create uploader
    let uploader = ParallelUploader::new(client, config.parallel_workers, config.retry_attempts);

    // Estimate cost
    tracing::info!("\n[2/4] Estimating cost...");
    let estimated_cost = estimate_blob_costs(&blobs);
    tracing::info!("  Total Blobs:       {}", blobs.len());
    tracing::info!("  Estimated Gas:     {} gas", estimated_cost.0);
    tracing::info!("  Estimated Fee:     {:.6} TIA", estimated_cost.1);
    tracing::info!("  Estimated Cost:    ${:.2} USD (at $5/TIA)", estimated_cost.1 * 5.0);

    if args.dry_run {
        tracing::info!("\n✓ Dry run complete. No blobs uploaded.");
        return Ok(());
    }

    // Confirm upload
    tracing::info!("\n[3/4] Uploading to Celestia Mainnet...");
    tracing::warn!("This will cost approximately {:.6} TIA (${:.2} USD)", estimated_cost.1, estimated_cost.1 * 5.0);
    tracing::warn!("Press Ctrl+C within 5 seconds to cancel...");

    tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;

    // Upload blobs
    let results = uploader.upload_all(blobs).await?;

    // Generate report
    let report = uploader.generate_report(&results);
    report.print_summary();

    // Save report
    report.save_to_file(args.output.to_str().unwrap())?;

    // Verify if requested
    if args.verify {
        tracing::info!("\n[4/4] Verifying data availability...");

        let verifier = DataAvailabilityVerifier::new(
            CelestiaClient::new(CelestiaConfig {
                node_url: celestia_config.node_url.clone(),
                auth_token: celestia_config.auth_token.clone(),
                network: celestia_config.network.clone(),
                gas_multiplier: 1.1,
                timeout_ms: 30_000,
            })?
        );

        let verification = verifier.verify_all(&results).await?;
        verification.print_summary();

        if !verification.is_complete() {
            tracing::error!("⚠ Some blobs failed verification!");
            std::process::exit(1);
        }
    }

    tracing::info!("\n✓ Genesis upload complete!");
    tracing::info!("  Height Range:      {} - {}", report.height_range.0, report.height_range.1);
    tracing::info!("  Namespace:         {}", report.namespace);
    tracing::info!("  Report:            {}", args.output.display());

    Ok(())
}

/// Estimate gas and fees for blobs
fn estimate_blob_costs(blobs: &[CelestiaBlob]) -> (u64, f64) {
    let mut total_gas: u64 = 0;

    for blob in blobs {
        let blob_json = serde_json::to_string(blob).unwrap();
        let blob_size = blob_json.len();

        let sparse_shares = ((blob_size as f64 / SHARE_SIZE_BYTES as f64).ceil()) as u64;
        let blob_gas = FIXED_COST_GAS + (sparse_shares * SHARE_SIZE_BYTES * GAS_PER_BYTE);

        total_gas += blob_gas;
    }

    let total_tia = (total_gas as f64 * MIN_GAS_PRICE_UTIA) / 1_000_000.0;

    (total_gas, total_tia)
}
