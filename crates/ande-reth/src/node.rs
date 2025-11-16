//! ANDE Node Implementation
//!
//! Production-ready node type for ANDE Chain sovereign rollup.
//! Uses standard Ethereum components with ANDE chain specification.

use reth_chainspec::ChainSpec;
use reth_ethereum_engine_primitives::{
    EthBuiltPayload, EthPayloadAttributes, EthPayloadBuilderAttributes,
};
use reth_ethereum_primitives::EthPrimitives;
use reth_node_builder::node::NodeTypes;
use reth_node_ethereum::{EthEngineTypes, EthereumNode};
use reth_provider::EthStorage;

/// ANDE Chain node type
///
/// Delegates to EthereumNode for stability and compatibility.
/// Custom features (precompiles, parallel EVM, MEV) are implemented
/// in ande-evm crate and will be integrated in future phases.
#[derive(Debug, Clone, Default)]
#[non_exhaustive]
pub struct AndeNode;

impl AndeNode {
    /// Create a new ANDE node instance
    pub fn new() -> Self {
        Self
    }

    /// Returns a [`ComponentsBuilder`] configured for ANDE Chain.
    ///
    /// Uses the same components as EthereumNode for maximum compatibility.
    pub fn components<Node>() -> reth_node_builder::components::ComponentsBuilder<
        Node,
        reth_node_ethereum::EthereumPoolBuilder,
        reth_node_builder::components::BasicPayloadServiceBuilder<
            reth_node_ethereum::EthereumPayloadBuilder,
        >,
        reth_node_ethereum::EthereumNetworkBuilder,
        reth_node_ethereum::EthereumExecutorBuilder,
        reth_node_ethereum::EthereumConsensusBuilder,
    >
    where
        Node: reth_node_builder::FullNodeTypes<
            Types: reth_node_builder::node::NodeTypes<
                ChainSpec: reth_chainspec::Hardforks 
                    + reth_chainspec::EthereumHardforks 
                    + reth_evm::eth::spec::EthExecutorSpec,
                Primitives = EthPrimitives,
            >,
        >,
        <Node::Types as reth_node_builder::node::NodeTypes>::Payload: reth_payload_primitives::PayloadTypes<
            BuiltPayload = EthBuiltPayload,
            PayloadAttributes = EthPayloadAttributes,
            PayloadBuilderAttributes = EthPayloadBuilderAttributes,
        >,
    {
        // Delegate to EthereumNode components
        EthereumNode::components()
    }
}

/// Node types for ANDE Chain
///
/// Uses standard Ethereum types for maximum compatibility
impl NodeTypes for AndeNode {
    type Primitives = EthPrimitives;
    type ChainSpec = ChainSpec;
    type Storage = EthStorage;
    type Payload = EthEngineTypes;
}

/// Implement Node trait for ANDE Chain
///
/// This implementation uses the same component structure as EthereumNode
/// for maximum compatibility with Reth v1.8.2 and Evolve sequencer.
impl<N> reth_node_builder::Node<N> for AndeNode
where
    N: reth_node_builder::FullNodeTypes<Types = Self>,
{
    type ComponentsBuilder = reth_node_builder::components::ComponentsBuilder<
        N,
        reth_node_ethereum::EthereumPoolBuilder,
        reth_node_builder::components::BasicPayloadServiceBuilder<
            reth_node_ethereum::EthereumPayloadBuilder,
        >,
        reth_node_ethereum::EthereumNetworkBuilder,
        reth_node_ethereum::EthereumExecutorBuilder,
        reth_node_ethereum::EthereumConsensusBuilder,
    >;

    type AddOns = reth_node_ethereum::EthereumAddOns<
        reth_node_builder::NodeAdapter<N>,
        reth_node_ethereum::EthereumEthApiBuilder,
        reth_node_ethereum::EthereumEngineValidatorBuilder,
    >;

    fn components_builder(&self) -> Self::ComponentsBuilder {
        EthereumNode::components()
    }

    fn add_ons(&self) -> Self::AddOns {
        reth_node_ethereum::EthereumAddOns::default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ande_node_creation() {
        let node = AndeNode::new();
        assert_eq!(std::mem::size_of_val(&node), 0);
    }
}
