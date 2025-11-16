//! ANDE Reth - Production Ethereum Client with Token Duality Precompile
//!
//! Based on Reth v1.8.2 with EthereumNode as the base.
//! Custom precompiles will be added in v0.2.

#![cfg_attr(not(test), warn(unused_crate_dependencies))]
#![cfg_attr(all(feature = "jemalloc", unix), global_allocator = tikv_jemallocator::Jemalloc)]

use reth::chainspec::EthereumChainSpecParser;
use reth::cli::Cli;
use reth_ethereum::node::EthereumNode;

fn main() {
    reth_cli_util::sigsegv_handler::install();

    // Enable backtraces unless a RUST_BACKTRACE value has already been explicitly provided.
    if std::env::var_os("RUST_BACKTRACE").is_none() {
        std::env::set_var("RUST_BACKTRACE", "1");
    }

    if let Err(err) = Cli::<EthereumChainSpecParser>::parse_args().run(|builder, _| async move {
        let handle = builder.node(EthereumNode::default()).launch_with_debug_capabilities().await?;

        handle.node_exit_future.await
    }) {
        eprintln!("Error: {err:?}");
        std::process::exit(1);
    }
}
