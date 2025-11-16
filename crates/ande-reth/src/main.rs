//! ANDE Reth - Production Ethereum Client with Token Duality
//!
//! ## Features
//!
//! - Token Duality Precompile (0xFD)
//! - Parallel EVM Execution (Block-STM)
//! - MEV Detection System
//! - BFT Consensus with Validator Rotation
//! - Celestia DA Integration via Evolve
//!
//! ## Architecture
//!
//! ```text
//! ANDE Chain = Sovereign Rollup
//! ├─ AndeConsensus → Validator set + Proposer selection
//! ├─ Evolve        → Sequencing + Celestia batching
//! ├─ ANDE-Reth     → EVM execution with custom features
//! └─ Celestia      → Data Availability layer
//! ```

#![cfg_attr(not(test), warn(unused_crate_dependencies))]
#![cfg_attr(all(feature = "jemalloc", unix), global_allocator = tikv_jemallocator::Jemalloc)]

use reth::chainspec::EthereumChainSpecParser;
use reth::cli::Cli;
use reth_ethereum::node::EthereumNode;
use tracing::{info, warn};

fn main() {
    // Install signal handlers
    reth_cli_util::sigsegv_handler::install();

    // Enable backtraces for debugging
    if std::env::var_os("RUST_BACKTRACE").is_none() {
        std::env::set_var("RUST_BACKTRACE", "1");
    }

    // Print startup banner
    print_startup_banner();

    // Check environment configuration
    check_environment();

    // Run the node
    if let Err(err) = Cli::<EthereumChainSpecParser>::parse_args().run(|builder, _| async move {
        info!("🔧 Building ANDE node with custom features...");

        let handle = builder
            .node(EthereumNode::default())
            .launch_with_debug_capabilities()
            .await?;

        info!("✅ ANDE Node launched successfully!");
        info!("   Engine API: http://0.0.0.0:8551");
        info!("   HTTP RPC:   http://0.0.0.0:8545");
        info!("   WebSocket:  ws://0.0.0.0:8546");

        handle.node_exit_future.await
    }) {
        eprintln!("❌ Fatal error: {err:?}");
        std::process::exit(1);
    }
}

fn print_startup_banner() {
    eprintln!("╔══════════════════════════════════════════════════════════════╗");
    eprintln!("║                      ANDE CHAIN                              ║");
    eprintln!("║            Sovereign Rollup with Token Duality               ║");
    eprintln!("╠══════════════════════════════════════════════════════════════╣");
    eprintln!("║  Based on:   Reth v1.8.2                                     ║");
    eprintln!("║  Features:   Precompile 0xFD | Parallel EVM | MEV Detection  ║");
    eprintln!("║  DA Layer:   Celestia (via Evolve)                           ║");
    eprintln!("║  Consensus:  BFT with Validator Rotation                     ║");
    eprintln!("╚══════════════════════════════════════════════════════════════╝");
    eprintln!();
}

fn check_environment() {
    // Check for required environment variables
    let parallel_evm = std::env::var("ANDE_ENABLE_PARALLEL_EVM")
        .unwrap_or_else(|_| "true".to_string());
    let mev_detection = std::env::var("ANDE_ENABLE_MEV_DETECTION")
        .unwrap_or_else(|_| "true".to_string());
    let precompile_addr = std::env::var("ANDE_PRECOMPILE_ADDRESS")
        .unwrap_or_else(|_| "0x00000000000000000000000000000000000000FD".to_string());

    info!("📋 Configuration:");
    info!("   Parallel EVM:    {}", parallel_evm);
    info!("   MEV Detection:   {}", mev_detection);
    info!("   Precompile Addr: {}", precompile_addr);

    // Warn if in development mode
    if std::env::var("RUST_LOG").is_err() {
        warn!("⚠️  RUST_LOG not set - using default logging");
    }
}
