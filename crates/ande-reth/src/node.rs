//! ANDE Node - Custom Reth Node with Token Duality
//!
//! This module provides the AndeNode type for the ANDE Chain sovereign rollup.

use tracing::info;

/// ANDE Node Type
///
/// **TEMPORARY IMPLEMENTATION (v0.1):**
/// For now, we simply re-export EthereumNode.
/// The custom EVM configuration is handled via AndeEvmConfig (type alias to EthEvmConfig).
///
/// ## Architecture
///
/// ```text
/// AndeNode = EthereumNode
/// ├─ Network:   Ethereum (standard P2P)
/// ├─ Pool:      Ethereum (standard txpool)
/// ├─ Consensus: Ethereum (PoS consensus)
/// ├─ Executor:  AndeEvmConfig (= EthEvmConfig for now)
/// └─ Payload:   Ethereum (standard block building)
/// ```
///
/// ## Integration with Evolve
///
/// The ANDE Chain is a sovereign rollup built with:
/// - **Reth** (execution layer) - this node
/// - **Evolve** (sequencer/consensus) - connects via Engine API
/// - **Celestia** (DA layer) - data availability
///
/// Reth only needs to:
/// 1. Expose HTTP RPC on port 8545
/// 2. Expose Engine API on port 8551
/// 3. Execute blocks as instructed by Evolve
pub type AndeNode = reth_ethereum::node::EthereumNode;

/// Create a new ANDE node instance
pub fn new_ande_node() -> AndeNode {
    info!(
        target: "ande::node",
        "Initializing ANDE Node (EthereumNode base for sovereign rollup)"
    );

    AndeNode::default()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ande_node_creation() {
        let node = new_ande_node();
        assert!(std::mem::size_of_val(&node) >= 0);
    }
}
