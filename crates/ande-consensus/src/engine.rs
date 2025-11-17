//! Main consensus engine coordinating all consensus operations

use crate::{
    config::ConsensusConfig,
    contract_client::ContractClient,
    error::{ConsensusError, Result},
    metrics::ConsensusMetrics,
    types::{ConsensusState, ValidatorSetUpdate},
    validator_set::{ValidatorSet, ValidatorSetStats},
};
use alloy_primitives::Address;
use prometheus::Registry;
use std::{sync::Arc, time::Duration};
use tokio::{
    sync::{Mutex, RwLock},
    time::interval,
};
use tracing::{debug, error, info, warn};

/// Main consensus engine
pub struct ConsensusEngine {
    /// Configuration
    config: ConsensusConfig,

    /// Contract client for on-chain interactions
    client: Arc<ContractClient>,

    /// Validator set manager
    validator_set: Arc<RwLock<ValidatorSet>>,

    /// Current consensus state
    state: Arc<RwLock<ConsensusState>>,

    /// Metrics
    metrics: Arc<ConsensusMetrics>,

    /// Whether engine is running
    running: Arc<Mutex<bool>>,

    /// Last block where timeout was checked
    last_timeout_check: Arc<RwLock<u64>>,
}

impl ConsensusEngine {
    /// Create new consensus engine
    ///
    /// # Errors
    ///
    /// Returns error if initialization fails
    pub async fn new(config: ConsensusConfig) -> Result<Self> {
        // Validate configuration
        config
            .validate()
            .map_err(ConsensusError::ConfigError)?;

        info!(
            consensus_contract = ?config.consensus_contract,
            sequencer = ?config.sequencer_address,
            "Initializing consensus engine"
        );

        // Initialize contract client
        let client = Arc::new(
            ContractClient::new(
                &config.rpc_url,
                Some(&config.ws_url),
                config.consensus_contract,
                config.coordinator_contract,
            )
            .await?,
        );

        // Initialize validator set
        let validator_set = Arc::new(RwLock::new(ValidatorSet::new()));

        // Initialize metrics
        let registry = Registry::new();
        let metrics = Arc::new(
            ConsensusMetrics::new(&registry)
                .map_err(|e| ConsensusError::Internal(format!("Metrics init failed: {e}")))?,
        );

        // Initialize state
        let state = Arc::new(RwLock::new(ConsensusState {
            current_block: 0,
            current_epoch: 0,
            current_rotation: 0,
            current_proposer: Address::ZERO,
            active_validators: 0,
            total_voting_power: 0,
            bft_threshold: 0,
            last_update: 0,
        }));

        Ok(Self {
            config,
            client,
            validator_set,
            state,
            metrics,
            running: Arc::new(Mutex::new(false)),
            last_timeout_check: Arc::new(RwLock::new(0)),
        })
    }

    /// Start the consensus engine
    ///
    /// # Errors
    ///
    /// Returns error if engine start fails
    pub async fn start(&self) -> Result<()> {
        let mut running = self.running.lock().await;
        if *running {
            return Err(ConsensusError::Internal(
                "Engine already running".to_string(),
            ));
        }
        *running = true;
        drop(running);

        info!("Starting consensus engine");

        // Initial sync
        self.sync_validator_set().await?;

        // Spawn background tasks
        self.spawn_sync_task();
        self.spawn_timeout_monitor();
        self.spawn_metrics_updater();

        info!("Consensus engine started successfully");
        Ok(())
    }

    /// Stop the consensus engine
    pub async fn stop(&self) {
        let mut running = self.running.lock().await;
        *running = false;

        info!("Consensus engine stopped");
    }

    /// Check if engine is running
    pub async fn is_running(&self) -> bool {
        *self.running.lock().await
    }

    /// Sync validator set from on-chain contracts
    pub async fn sync_validator_set(&self) -> Result<()> {
        info!("Syncing validator set from chain");

        // Fetch validator info from contract
        let validators = self.client.get_all_validators_info().await?;
        let block_number = self.client.get_block_number().await?;
        let epoch = self.client.get_current_epoch().await?;

        // Update validator set
        let mut validator_set = self.validator_set.write().await;
        validator_set.update_from_chain(validators, block_number)?;

        // Update state
        let mut state = self.state.write().await;
        state.current_block = block_number;
        state.current_epoch = epoch;
        state.active_validators = validator_set.active_count();
        state.total_voting_power = validator_set.total_voting_power();
        state.bft_threshold = validator_set.bft_threshold();
        state.current_proposer = validator_set.current_proposer().unwrap_or(Address::ZERO);
        state.last_update = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();

        // Update metrics
        self.metrics
            .validator_set_updates
            .inc();
        self.update_metrics_from_state(&state, &validator_set.stats());

        info!(
            epoch,
            block_number,
            active_validators = state.active_validators,
            total_power = state.total_voting_power,
            "Validator set synced successfully"
        );

        Ok(())
    }

    /// Get current proposer address (without block number)
    pub async fn current_proposer(&self) -> Option<Address> {
        self.validator_set.read().await.current_proposer()
    }

    /// Get proposer for specific block number
    pub async fn get_current_proposer(&self, _block_number: u64) -> Result<Address> {
        // For now, return current proposer
        // TODO: Implement historical proposer lookup based on block number
        self.validator_set
            .read()
            .await
            .current_proposer()
            .ok_or_else(|| ConsensusError::Internal("No proposer available".to_string()))
    }

    /// Check if this node is the current proposer
    pub async fn am_i_proposer(&self) -> bool {
        self.current_proposer()
            .await
            .map_or(false, |proposer| proposer == self.config.sequencer_address)
    }

    /// Verify that a block was produced by the correct proposer
    pub async fn verify_block_proposer(&self, producer: Address) -> Result<()> {
        let validator_set = self.validator_set.read().await;
        validator_set.verify_proposer(producer)
    }

    /// Record that a block was produced
    pub async fn record_block_produced(&self, producer: Address, block_number: u64) -> Result<()> {
        let mut validator_set = self.validator_set.write().await;
        validator_set.record_block_produced(&producer, block_number)?;

        // Update metrics
        if producer == self.config.sequencer_address {
            self.metrics.blocks_produced.inc();
        }

        // Update state
        let mut state = self.state.write().await;
        state.current_block = block_number;

        info!(producer = ?producer, block = block_number, "Block produced");
        Ok(())
    }

    /// Record that a block was missed
    pub async fn record_block_missed(&self, validator: Address) -> Result<()> {
        let mut validator_set = self.validator_set.write().await;
        validator_set.record_block_missed(&validator)?;

        // Update metrics
        if validator == self.config.sequencer_address {
            self.metrics.blocks_missed.inc();
        }

        warn!(validator = ?validator, "Block missed");
        Ok(())
    }

    /// Check for timeout condition
    pub async fn check_timeout(&self) -> Result<bool> {
        let current_block = self.client.get_block_number().await?;
        let last_check = *self.last_timeout_check.read().await;

        // Only check once per block
        if current_block <= last_check {
            return Ok(false);
        }

        // Check if timeout reached on-chain
        let timeout_reached = self.client.is_timeout_reached().await?;

        if timeout_reached {
            warn!(
                current_block,
                timeout_blocks = self.config.timeout_blocks,
                "Timeout detected - rotation needed"
            );

            self.metrics.timeouts_detected.inc();

            // Update last check
            *self.last_timeout_check.write().await = current_block;

            return Ok(true);
        }

        Ok(false)
    }

    /// Force rotation to next proposer
    pub async fn force_rotation(&self, reason: &str) -> Result<Address> {
        info!(reason, "Forcing proposer rotation");

        let mut validator_set = self.validator_set.write().await;
        let next_proposer = validator_set.force_next_proposer()?;

        // Update metrics
        self.metrics.forced_rotations.inc();

        // Update state
        let mut state = self.state.write().await;
        state.current_proposer = next_proposer;
        state.current_rotation += 1;

        info!(
            next_proposer = ?next_proposer,
            rotation = state.current_rotation,
            reason,
            "Proposer rotated"
        );

        Ok(next_proposer)
    }

    /// Get current consensus state
    pub async fn get_state(&self) -> ConsensusState {
        self.state.read().await.clone()
    }

    /// Get validator set statistics
    pub async fn get_validator_stats(&self) -> ValidatorSetStats {
        self.validator_set.read().await.stats()
    }

    /// Spawn background task to periodically sync validator set
    fn spawn_sync_task(&self) {
        let engine = self.clone_self();
        let interval_duration = engine.config.sync_interval;

        tokio::spawn(async move {
            let mut ticker = interval(interval_duration);

            loop {
                ticker.tick().await;

                if !engine.is_running().await {
                    break;
                }

                if let Err(e) = engine.sync_validator_set().await {
                    error!(error = %e, "Failed to sync validator set");
                }
            }

            debug!("Sync task stopped");
        });
    }

    /// Spawn background task to monitor timeouts
    fn spawn_timeout_monitor(&self) {
        let engine = self.clone_self();
        let check_interval = engine.config.block_time;

        tokio::spawn(async move {
            let mut ticker = interval(check_interval);

            loop {
                ticker.tick().await;

                if !engine.is_running().await {
                    break;
                }

                match engine.check_timeout().await {
                    Ok(true) => {
                        // Timeout detected, force rotation
                        if let Err(e) = engine.force_rotation("timeout").await {
                            error!(error = %e, "Failed to force rotation on timeout");
                        }
                    }
                    Ok(false) => {
                        // No timeout, continue
                    }
                    Err(e) => {
                        error!(error = %e, "Failed to check timeout");
                    }
                }
            }

            debug!("Timeout monitor stopped");
        });
    }

    /// Spawn background task to update metrics
    fn spawn_metrics_updater(&self) {
        let engine = self.clone_self();

        tokio::spawn(async move {
            let mut ticker = interval(Duration::from_secs(10));

            loop {
                ticker.tick().await;

                if !engine.is_running().await {
                    break;
                }

                let state = engine.state.read().await;
                let stats = engine.validator_set.read().await.stats();
                engine.update_metrics_from_state(&state, &stats);
            }

            debug!("Metrics updater stopped");
        });
    }

    /// Update prometheus metrics
    fn update_metrics_from_state(&self, state: &ConsensusState, stats: &ValidatorSetStats) {
        self.metrics.current_block.set(state.current_block as i64);
        self.metrics.current_epoch.set(state.current_epoch as i64);
        self.metrics
            .current_rotation
            .set(state.current_rotation as i64);
        self.metrics
            .active_validators
            .set(stats.active_count as i64);
        self.metrics
            .total_voting_power
            .set(stats.total_voting_power as i64);
        self.metrics
            .bft_threshold
            .set(stats.bft_threshold as i64);
        self.metrics.uptime.set(i64::from(stats.average_uptime));

        // Set proposer flag
        let is_proposer =
            if state.current_proposer == self.config.sequencer_address { 1 } else { 0 };
        self.metrics.is_proposer.set(is_proposer);
    }

    /// Clone self for spawning tasks
    fn clone_self(&self) -> Self {
        Self {
            config: self.config.clone(),
            client: Arc::clone(&self.client),
            validator_set: Arc::clone(&self.validator_set),
            state: Arc::clone(&self.state),
            metrics: Arc::clone(&self.metrics),
            running: Arc::clone(&self.running),
            last_timeout_check: Arc::clone(&self.last_timeout_check),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    #[ignore] // Requires running blockchain
    async fn test_consensus_engine_creation() {
        let config = ConsensusConfig::default();
        let engine = ConsensusEngine::new(config).await;

        // Should succeed with default config
        assert!(engine.is_ok());
    }
}
