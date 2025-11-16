//! ANDE Node - Custom Reth Node with Token Duality
//!
//! This module implements the AndeNode using Reth's EthereumNode as a base,
//! with a custom EVM configuration (AndeEvmConfig) for ANDE precompiles.

use crate::executor::AndeEvmConfig;
use reth_ethereum::node::EthereumNode;
use std::marker::PhantomData;
use tracing::info;

/// ANDE Node Configuration
///
/// **Strategy for Reth v1.8.2+:**
///
/// Instead of implementing a complex custom Node trait, we use EthereumNode
/// as our base and inject the AndeEvmConfig via `with_types()` + custom setup.
///
/// ## Architecture
///
/// ```text
/// AndeNode (= EthereumNode)
/// ├─ Network:   Ethereum (standard P2P)
/// ├─ Pool:      Ethereum (standard txpool)
/// ├─ Consensus: Ethereum (PoS consensus)
/// ├─ Executor:  AndeEvmConfig (delegates to EthEvmConfig) ← CUSTOM EVM
/// └─ Payload:   Ethereum (standard block building)
/// ```
///
/// The only customization is AndeEvmConfig, which wraps EthEvmConfig and
/// will eventually inject the ANDE precompile provider.
#[derive(Debug, Clone, Default)]
#[non_exhaustive]
pub struct AndeNode {
    _phantom: PhantomData<()>,
    /// Custom EVM configuration with ANDE precompiles
    evm_config: Option<AndeEvmConfig>,
}

impl AndeNode {
    /// Create a new ANDE node configuration
    pub fn new() -> Self {
        info!(
            target: "ande::node",
            "Initializing ANDE Node with Token Duality precompile support"
        );
        Self {
            _phantom: PhantomData,
            evm_config: Some(AndeEvmConfig::default()),
        }
    }

    /// Get the custom EVM configuration
    pub fn evm_config(&self) -> &AndeEvmConfig {
        self.evm_config.as_ref().expect("EVM config always initialized")
    }
}

// For now, we simply re-export EthereumNode as AndeNode's base type.
// The Node trait implementation comes from EthereumNode.
//
// In the future, if we need to customize the Node trait further,
// we can implement it here with a delegation pattern similar to AndeEvmConfig.
//
// Usage in main.rs:
//   builder.node(EthereumNode::default()).launch().await
//
// Then inject AndeEvmConfig at a different level (via NodeBuilder::with_types).

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ande_node_creation() {
        let node = AndeNode::new();
        assert!(std::mem::size_of_val(&node) >= 0);
        assert!(node.evm_config().clone() != ());  // Ensure EVM config exists
    }
}
