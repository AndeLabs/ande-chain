//! ANDE Node - Custom Reth Node with Token Duality
//!
//! This module implements the main AndeNode struct and its Node trait implementation,
//! following the pattern established by op-reth for Optimism.

use crate::executor::AndeExecutorBuilder;
use reth_ethereum::node::{
    EthereumAddOns, EthereumConsensusBuilder, EthereumNetworkBuilder, EthereumPoolBuilder,
};
use reth_ethereum_payload_builder::EthereumPayloadBuilder;
use reth_node_api::{FullNodeTypes, NodeTypes};
use reth_node_builder::{
    components::{ComponentsBuilder, ConsensusBuilder, ExecutorBuilder, NetworkBuilder, PayloadServiceBuilder, PoolBuilder},
    BuilderContext, Node, PayloadBuilderConfig,
};
use reth_payload_builder::{PayloadBuilderHandle, PayloadBuilderService};
use reth_primitives::EthPrimitives;
use std::marker::PhantomData;
use tracing::info;

/// ANDE Node Configuration
///
/// This struct encapsulates ANDE-specific configuration and implements
/// the Node trait to provide custom components to the Reth builder.
///
/// ## Architecture
///
/// ```text
/// AndeNode
/// ├─ Network:   Ethereum (standard P2P)
/// ├─ Pool:      Ethereum (standard txpool)
/// ├─ Consensus: Ethereum (PoS consensus)
/// ├─ Executor:  ANDE (custom EVM with precompiles) ← KEY DIFFERENCE
/// └─ Payload:   Ethereum (standard block building)
/// ```
///
/// The only customization is the Executor, which injects the ANDE Token Duality
/// precompile at address 0xFD into the EVM.
#[derive(Debug, Clone, Default)]
#[non_exhaustive]
pub struct AndeNode {
    _phantom: PhantomData<()>,
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
        }
    }
}

/// ANDE Components Builder
///
/// Defines the complete set of components for the ANDE node.
/// Most components reuse Ethereum defaults, with the Executor being custom.
pub type AndeComponentsBuilder<N> = ComponentsBuilder<
    N,
    EthereumPoolBuilder,          // Standard Ethereum transaction pool
    EthereumPayloadBuilder,        // Standard Ethereum block building
    EthereumNetworkBuilder,        // Standard Ethereum P2P networking
    AndeExecutorBuilder<N>,        // ✅ CUSTOM: ANDE executor with precompiles
    EthereumConsensusBuilder,      // Standard Ethereum PoS consensus
>;

impl<N> Node<N> for AndeNode
where
    N: FullNodeTypes<Types: NodeTypes<Primitives = EthPrimitives>>,
{
    type ComponentsBuilder = AndeComponentsBuilder<N>;
    type AddOns = EthereumAddOns;

    fn components_builder(&self) -> Self::ComponentsBuilder {
        info!(
            target: "ande::node",
            "Building ANDE components with custom executor"
        );

        ComponentsBuilder::default()
            .pool(EthereumPoolBuilder::default())
            .payload(EthereumPayloadBuilder::default())
            .network(EthereumNetworkBuilder::default())
            .executor(AndeExecutorBuilder::new())  // ✅ ANDE custom executor
            .consensus(EthereumConsensusBuilder::default())
    }

    fn add_ons(&self) -> Self::AddOns {
        // Use standard Ethereum add-ons (RPC, etc.)
        EthereumAddOns::default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ande_node_creation() {
        let node = AndeNode::new();
        assert!(std::mem::size_of_val(&node) >= 0);
    }
}
