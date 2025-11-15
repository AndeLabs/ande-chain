//! Consensus engine integration for ANDE Node

use ande_consensus::{
    ConsensusConfig, ConsensusEngine, ConsensusError, Result,
    types::ConsensusState,
};
use alloy_primitives::Address;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, info, warn};

/// Wrapper for consensus engine integration with ande-node
pub struct ConsensusIntegration {
    /// The consensus engine
    engine: Arc<RwLock<ConsensusEngine>>,

    /// Whether consensus is enabled
    enabled: bool,

    /// Sequencer address for this node
    sequencer_address: Address,
}

impl ConsensusIntegration {
    /// Create new consensus integration
    ///
    /// # Errors
    ///
    /// Returns error if consensus engine initialization fails
    pub async fn new(config: ConsensusConfig) -> Result<Self> {
        let sequencer_address = config.sequencer_address;
        let enabled = true;

        info!(
            sequencer = ?sequencer_address,
            "Initializing consensus integration"
        );

        let engine = ConsensusEngine::new(config).await?;
        let engine = Arc::new(RwLock::new(engine));

        Ok(Self {
            engine,
            enabled,
            sequencer_address,
        })
    }

    /// Create a disabled consensus integration (for testing or single-sequencer mode)
    pub fn disabled() -> Self {
        Self {
            engine: Arc::new(RwLock::new(unsafe { std::mem::zeroed() })),
            enabled: false,
            sequencer_address: Address::ZERO,
        }
    }

    /// Start the consensus engine
    pub async fn start(&self) -> Result<()> {
        if !self.enabled {
            info!("Consensus integration disabled, skipping start");
            return Ok(());
        }

        info!("Starting consensus engine");
        let engine = self.engine.read().await;
        engine.start().await?;

        info!("✅ Consensus engine started successfully");
        Ok(())
    }

    /// Stop the consensus engine
    pub async fn stop(&self) {
        if !self.enabled {
            return;
        }

        info!("Stopping consensus engine");
        let engine = self.engine.read().await;
        engine.stop().await;
    }

    /// Check if this node should produce the next block
    pub async fn should_produce_block(&self) -> bool {
        if !self.enabled {
            // Single-sequencer mode, always produce
            return true;
        }

        let engine = self.engine.read().await;
        engine.am_i_proposer().await
    }

    /// Get current proposer address
    pub async fn get_current_proposer(&self) -> Option<Address> {
        if !self.enabled {
            return Some(self.sequencer_address);
        }

        let engine = self.engine.read().await;
        engine.get_current_proposer().await
    }

    /// Verify block proposer
    pub async fn verify_block_proposer(&self, producer: Address) -> Result<()> {
        if !self.enabled {
            return Ok(());
        }

        let engine = self.engine.read().await;
        engine.verify_block_proposer(producer).await
    }

    /// Record successful block production
    pub async fn on_block_produced(&self, producer: Address, block_number: u64) -> Result<()> {
        if !self.enabled {
            return Ok(());
        }

        debug!(
            producer = ?producer,
            block = block_number,
            "Recording block production"
        );

        let engine = self.engine.read().await;
        engine.record_block_produced(producer, block_number).await
    }

    /// Record missed block
    pub async fn on_block_missed(&self, validator: Address) -> Result<()> {
        if !self.enabled {
            return Ok(());
        }

        warn!(validator = ?validator, "Recording missed block");

        let engine = self.engine.read().await;
        engine.record_block_missed(validator).await
    }

    /// Handle timeout condition
    pub async fn handle_timeout(&self) -> Result<()> {
        if !self.enabled {
            return Ok(());
        }

        warn!("Handling timeout - forcing rotation");

        let engine = self.engine.read().await;
        let timeout = engine.check_timeout().await?;

        if timeout {
            engine.force_rotation("timeout").await?;
        }

        Ok(())
    }

    /// Get current consensus state
    pub async fn get_state(&self) -> Option<ConsensusState> {
        if !self.enabled {
            return None;
        }

        let engine = self.engine.read().await;
        Some(engine.get_state().await)
    }

    /// Check if consensus is enabled
    pub const fn is_enabled(&self) -> bool {
        self.enabled
    }

    /// Get sequencer address
    pub const fn sequencer_address(&self) -> Address {
        self.sequencer_address
    }
}

/// Builder for consensus integration
pub struct ConsensusIntegrationBuilder {
    config: Option<ConsensusConfig>,
    enabled: bool,
}

impl ConsensusIntegrationBuilder {
    /// Create new builder
    pub fn new() -> Self {
        Self {
            config: None,
            enabled: true,
        }
    }

    /// Set consensus config
    pub fn with_config(mut self, config: ConsensusConfig) -> Self {
        self.config = Some(config);
        self
    }

    /// Enable or disable consensus
    pub fn enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }

    /// Build consensus integration
    ///
    /// # Errors
    ///
    /// Returns error if consensus initialization fails
    pub async fn build(self) -> Result<ConsensusIntegration> {
        if !self.enabled {
            info!("Building disabled consensus integration");
            return Ok(ConsensusIntegration::disabled());
        }

        let config = self.config.ok_or_else(|| {
            ConsensusError::ConfigError("Consensus config not provided".to_string())
        })?;

        ConsensusIntegration::new(config).await
    }
}

impl Default for ConsensusIntegrationBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_disabled_integration() {
        let integration = ConsensusIntegration::disabled();
        assert!(!integration.is_enabled());
        assert_eq!(integration.sequencer_address(), Address::ZERO);
    }

    #[tokio::test]
    async fn test_disabled_should_always_produce() {
        let integration = ConsensusIntegration::disabled();
        assert!(integration.should_produce_block().await);
    }
}
