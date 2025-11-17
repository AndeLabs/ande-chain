//! ANDE Node Implementation - Custom Reth Fork
//!
//! This is the ANDE Chain custom node implementation based on Reth v1.8.2.
//! It integrates the Token Duality Precompile and custom EVM execution.
//!
//! ## Key Differences from Standard EthereumNode:
//! - Custom Executor: AndeExecutorBuilder (not EthereumExecutorBuilder)
//! - Custom EVM: AndeEvmFactory with AndePrecompileProvider
//! - Precompile: Token Duality at address 0xFD
//!
//! This is a FORK, not a wrapper. We own and maintain this code.

use crate::executor::AndeExecutorBuilder;
use crate::consensus::AndeConsensusBuilder;
use reth_chainspec::ChainSpec;
use reth_ethereum::{
    node::{
        api::{FullNodeTypes, NodeTypes},
        builder::{
            components::{BasicPayloadServiceBuilder, ComponentsBuilder},
            rpc::RpcAddOns,
            Node,
        },
        node::EthereumPoolBuilder,
        EthereumEthApiBuilder,
    },
};
use reth_ethereum_primitives::EthPrimitives;
use reth_node_builder::NodeAdapter;
use reth_node_ethereum::{
    EthEngineTypes,
    EthereumEngineValidatorBuilder,
    EthereumNetworkBuilder,
    EthereumPayloadBuilder,
};
use reth_provider::EthStorage;

/// ANDE Chain Node Type - Custom Reth Fork
///
/// This is NOT a wrapper around EthereumNode.
/// This is our custom node implementation that:
/// - Uses AndeExecutorBuilder for custom EVM execution
/// - Integrates Token Duality Precompile at 0xFD
/// - Maintains compatibility with Reth v1.8.2 architecture
///
/// ## Active Features:
/// ✅ Token Duality Precompile (0xFD)
/// ✅ Custom EVM Factory (AndeEvmFactory)
/// ✅ Custom Precompile Provider (AndePrecompileProvider)
///
/// ## Future Features:
/// ⏳ Parallel EVM Execution (Block-STM)
/// ⏳ MEV Detection & Fair Distribution
/// ⏳ Enhanced validator selection
#[derive(Debug, Clone, Default)]
#[non_exhaustive]
pub struct AndeNode;

impl AndeNode {
    /// Create a new ANDE node instance
    pub const fn new() -> Self {
        Self
    }
}

/// Node types for ANDE Chain
///
/// Uses Ethereum-compatible primitives for maximum compatibility with tooling,
/// but the execution layer is customized with ANDE-specific features.
impl NodeTypes for AndeNode {
    type Primitives = EthPrimitives;
    type ChainSpec = ChainSpec;
    type Storage = EthStorage;
    type Payload = EthEngineTypes;
}

/// ANDE node add-ons - RPC and validation configuration
/// 
/// Uses standard Ethereum RPC API builder and engine validator for compatibility.
pub type AndeAddOns<N> = RpcAddOns<N, EthereumEthApiBuilder, EthereumEngineValidatorBuilder>;

/// ANDE Chain Node Implementation
///
/// This is our CUSTOM node implementation. Key customizations:
/// 
/// 1. **Custom Executor**: Uses `AndeExecutorBuilder` instead of `EthereumExecutorBuilder`
/// 2. **Custom EVM**: AndeEvmFactory with Token Duality Precompile
/// 3. **Same architecture**: Compatible with Reth v1.8.2 and Evolve sequencer
///
/// The component structure follows Reth patterns but with ANDE customizations.
impl<N> Node<N> for AndeNode
where
    N: FullNodeTypes<Types = Self>,
{
    type ComponentsBuilder = ComponentsBuilder<
        N,
        EthereumPoolBuilder,
        BasicPayloadServiceBuilder<EthereumPayloadBuilder>,
        EthereumNetworkBuilder,
        AndeExecutorBuilder,
        AndeConsensusBuilder,
    >;

    type AddOns = AndeAddOns<NodeAdapter<N>>;

    fn components_builder(&self) -> Self::ComponentsBuilder {
        ComponentsBuilder::default()
            .node_types::<N>()
            .pool(EthereumPoolBuilder::default())
            .executor(AndeExecutorBuilder::default())
            .payload(BasicPayloadServiceBuilder::new(EthereumPayloadBuilder::default()))
            .network(EthereumNetworkBuilder::default())
            .consensus(AndeConsensusBuilder::default())
    }

    fn add_ons(&self) -> Self::AddOns {
        AndeAddOns::default()
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
