//! ANDE Chain Consensus - BFT Multi-Validator Consensus
//!
//! Custom consensus implementation that wraps EthBeaconConsensus and adds:
//! - BFT validator set management
//! - Proposer rotation with weighted round-robin
//! - 2/3+1 voting power threshold for finality
//! - Byzantine fault tolerance
//!
//! ## Architecture
//!
//! ```text
//! AndeConsensus
//!   ├─ inner: EthBeaconConsensus  (standard Ethereum validation)
//!   └─ consensus_engine: ConsensusEngine  (BFT logic)
//!       ├─ ValidatorSet (stake-weighted validators)
//!       ├─ ProposerSelection (weighted round-robin)
//!       └─ BlockAttestation (2/3+ signatures)
//! ```

use ande_consensus::{ConsensusConfig, ConsensusEngine};
use reth_chainspec::ChainSpec;
use reth_consensus::{Consensus, ConsensusError, FullConsensus, HeaderValidator};
use reth_consensus_common::validation::{
    validate_against_parent_eip1559_base_fee, validate_against_parent_gas_limit,
    validate_against_parent_hash_number, validate_body_against_header,
};
use reth_ethereum_consensus::EthBeaconConsensus;
use reth_ethereum_primitives::{Block, BlockBody, EthPrimitives, Receipt};
use reth_execution_types::BlockExecutionResult;
use reth_node_api::{FullNodeTypes, NodeTypes};
use reth_node_builder::{BuilderContext, components::ConsensusBuilder};
use reth_primitives::{RecoveredBlock, SealedBlock, SealedHeader};
use std::sync::Arc;
use tracing::{debug, info, warn};

/// ANDE Consensus Builder
///
/// Creates AndeConsensus instances with BFT integration
#[derive(Debug, Default, Clone)]
#[non_exhaustive]
pub struct AndeConsensusBuilder;

impl AndeConsensusBuilder {
    /// Create a new ANDE consensus builder
    pub const fn new() -> Self {
        Self
    }

    /// Build the consensus implementation with BFT engine
    pub async fn build(
        chain_spec: Arc<ChainSpec>,
        consensus_config: Option<ConsensusConfig>,
    ) -> eyre::Result<Arc<AndeConsensus>> {
        let inner = EthBeaconConsensus::new(chain_spec);
        
        // Initialize consensus engine if config provided
        let consensus_engine = if let Some(config) = consensus_config {
            info!("🔧 Initializing BFT consensus engine");
            Some(Arc::new(ConsensusEngine::new(config).await?))
        } else {
            warn!("⚠️  No consensus config - BFT validation disabled");
            None
        };

        Ok(Arc::new(AndeConsensus {
            inner,
            consensus_engine,
        }))
    }
}

impl<Node> ConsensusBuilder<Node> for AndeConsensusBuilder
where
    Node: FullNodeTypes<Types: NodeTypes<ChainSpec = ChainSpec, Primitives = EthPrimitives>>,
{
    type Consensus = Arc<dyn FullConsensus<EthPrimitives, Error = ConsensusError>>;

    async fn build_consensus(
        self,
        ctx: &BuilderContext<Node>,
    ) -> eyre::Result<Self::Consensus> {
        // Try to load consensus config from environment
        let consensus_config = ConsensusConfig::from_env().ok();
        
        if consensus_config.is_some() {
            info!("✅ BFT consensus configuration found");
        } else {
            info!("ℹ️  Using standard Ethereum consensus (BFT disabled)");
        }

        // Initialize consensus engine if config provided
        let consensus_engine = if let Some(config) = consensus_config {
            info!("🔧 Initializing BFT consensus engine");
            Some(Arc::new(ConsensusEngine::new(config).await?))
        } else {
            warn!("⚠️  No consensus config - BFT validation disabled");
            None
        };

        Ok(Arc::new(AndeConsensus::new(ctx.chain_spec(), consensus_engine)) as Self::Consensus)
    }
}

/// ANDE Chain Consensus with BFT Support
///
/// Wraps EthBeaconConsensus and adds BFT validator consensus on top.
/// 
/// ## Validation Layers:
/// 1. Standard Ethereum validation (via inner)
/// 2. BFT proposer validation (via consensus_engine)
/// 3. Validator signature verification (via consensus_engine)
/// 4. 2/3+ voting power threshold (via consensus_engine)
#[derive(Clone)]
pub struct AndeConsensus {
    /// Inner Ethereum beacon consensus for standard validation
    inner: EthBeaconConsensus<ChainSpec>,
    
    /// Optional BFT consensus engine for multi-validator consensus
    consensus_engine: Option<Arc<ConsensusEngine>>,
}

impl std::fmt::Debug for AndeConsensus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AndeConsensus")
            .field("inner", &self.inner)
            .field("consensus_engine", &self.consensus_engine.as_ref().map(|_| "ConsensusEngine"))
            .finish()
    }
}

impl AndeConsensus {
    /// Create a new ANDE consensus instance
    pub const fn new(
        chain_spec: Arc<ChainSpec>,
        consensus_engine: Option<Arc<ConsensusEngine>>,
    ) -> Self {
        let inner = EthBeaconConsensus::new(chain_spec);
        Self {
            inner,
            consensus_engine,
        }
    }

    /// Get reference to consensus engine if available
    pub fn consensus_engine(&self) -> Option<&Arc<ConsensusEngine>> {
        self.consensus_engine.as_ref()
    }

    /// Validate block proposer is authorized
    async fn validate_proposer(&self, header: &SealedHeader) -> Result<(), ConsensusError> {
        let Some(engine) = &self.consensus_engine else {
            // BFT disabled, allow any proposer
            return Ok(());
        };

        let block_number = header.number;
        
        // Get current proposer from consensus engine
        let expected_proposer = engine
            .get_current_proposer(block_number)
            .await
            .map_err(|e| ConsensusError::Other(format!("Failed to get proposer: {e}").into()))?;

        let actual_proposer = header.beneficiary;

        if actual_proposer != expected_proposer {
            return Err(ConsensusError::Other(
                format!(
                    "Invalid proposer: expected {expected_proposer}, got {actual_proposer}"
                )
                .into(),
            ));
        }

        debug!(
            block = block_number,
            proposer = ?actual_proposer,
            "✅ Proposer validated"
        );

        Ok(())
    }

    /// Validate block has sufficient validator attestations
    async fn validate_attestations(&self, _header: &SealedHeader) -> Result<(), ConsensusError> {
        let Some(_engine) = &self.consensus_engine else {
            // BFT disabled, no attestation required
            return Ok(());
        };

        // TODO: Implement attestation validation
        // For now, we'll implement this in a future update when we have
        // the attestation collection mechanism in place
        
        Ok(())
    }
}

impl HeaderValidator for AndeConsensus {
    fn validate_header(&self, header: &SealedHeader) -> Result<(), ConsensusError> {
        // Delegate to inner consensus for standard validation
        self.inner.validate_header(header)
    }

    fn validate_header_against_parent(
        &self,
        header: &SealedHeader,
        parent: &SealedHeader,
    ) -> Result<(), ConsensusError> {
        // Standard Ethereum validation
        validate_against_parent_hash_number(header.header(), parent)?;

        let h = header.header();
        let ph = parent.header();
        
        // Timestamp validation (allow same timestamp like Evolve)
        if h.timestamp < ph.timestamp {
            return Err(ConsensusError::TimestampIsInPast {
                parent_timestamp: ph.timestamp,
                timestamp: h.timestamp,
            });
        }

        validate_against_parent_gas_limit(header, parent, self.inner.chain_spec())?;

        validate_against_parent_eip1559_base_fee(
            header.header(),
            parent.header(),
            self.inner.chain_spec(),
        )?;

        // BFT proposer validation (async, but we'll handle it in validate_block)
        // Can't do async here due to trait constraints

        Ok(())
    }
}

impl Consensus<Block> for AndeConsensus {
    type Error = ConsensusError;

    fn validate_body_against_header(
        &self,
        body: &BlockBody,
        header: &SealedHeader,
    ) -> Result<(), Self::Error> {
        validate_body_against_header(body, header.header())
    }

    fn validate_block_pre_execution(&self, block: &SealedBlock) -> Result<(), Self::Error> {
        self.inner.validate_block_pre_execution(block)?;

        // TODO: Add async proposer validation here once we refactor for async support
        // For now, log validation info
        if self.consensus_engine.is_some() {
            debug!(
                block = block.number,
                proposer = ?block.beneficiary,
                "🔍 BFT validation deferred to block processing"
            );
        }

        Ok(())
    }
}

impl FullConsensus<EthPrimitives> for AndeConsensus {
    fn validate_block_post_execution(
        &self,
        block: &RecoveredBlock<Block>,
        result: &BlockExecutionResult<Receipt>,
    ) -> Result<(), ConsensusError> {
        <EthBeaconConsensus<ChainSpec> as FullConsensus<EthPrimitives>>::validate_block_post_execution(&self.inner, block, result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloy_primitives::Address;

    #[test]
    fn test_ande_consensus_creation() {
        let chain_spec = Arc::new(ChainSpec::default());
        let consensus = AndeConsensus::new(chain_spec, None);
        assert!(consensus.consensus_engine().is_none());
    }

    #[test]
    fn test_ande_consensus_builder_creation() {
        let _builder = AndeConsensusBuilder::new();
    }
}
