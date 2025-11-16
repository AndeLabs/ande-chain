//! ANDE Chain Consensus Builder
//!
//! Provides a consensus builder compatible with Evolve sequencing.
//! Uses standard Ethereum consensus rules but is packaged as a separate
//! builder to maintain the same component structure as Evolve.

use reth_chainspec::EthChainSpec;
use reth_ethereum_consensus::EthBeaconConsensus;
use reth_ethereum_primitives::EthPrimitives;
use reth_node_api::{FullNodeTypes, NodeTypes};
use reth_node_builder::{BuilderContext, components::ConsensusBuilder};
use std::sync::Arc;

/// ANDE Chain Consensus Builder
///
/// This is a simple wrapper around EthBeaconConsensus that maintains
/// compatibility with the Reth v1.8.2 builder pattern.
/// 
/// For now, we use standard Ethereum consensus rules. In the future,
/// we can customize this for ANDE-specific consensus requirements.
#[derive(Debug, Default, Clone)]
#[non_exhaustive]
pub struct AndeConsensusBuilder;

impl AndeConsensusBuilder {
    /// Create a new ANDE consensus builder
    pub const fn new() -> Self {
        Self
    }
}

impl<Node> ConsensusBuilder<Node> for AndeConsensusBuilder
where
    Node: FullNodeTypes<
        Types: NodeTypes<
            ChainSpec: EthChainSpec + reth_chainspec::EthereumHardforks,
            Primitives = EthPrimitives,
        >,
    >,
{
    type Consensus = Arc<EthBeaconConsensus<<Node::Types as NodeTypes>::ChainSpec>>;

    async fn build_consensus(
        self,
        ctx: &BuilderContext<Node>,
    ) -> eyre::Result<Self::Consensus> {
        Ok(Arc::new(EthBeaconConsensus::new(ctx.chain_spec())))
    }
}
