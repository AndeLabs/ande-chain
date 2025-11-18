//! Prometheus metrics for consensus monitoring

use prometheus::{
    Histogram, HistogramOpts, IntCounter, IntGauge, Opts, Registry,
};

/// Consensus metrics for observability
#[derive(Clone)]
pub struct ConsensusMetrics {
    /// Current block number
    pub current_block: IntGauge,

    /// Current epoch number
    pub current_epoch: IntGauge,

    /// Current rotation number
    pub current_rotation: IntGauge,

    /// Number of active validators
    pub active_validators: IntGauge,

    /// Total voting power
    pub total_voting_power: IntGauge,

    /// BFT threshold
    pub bft_threshold: IntGauge,

    /// Number of blocks produced by this sequencer
    pub blocks_produced: IntCounter,

    /// Number of blocks missed by this sequencer
    pub blocks_missed: IntCounter,

    /// Number of attestations received
    pub attestations_received: IntCounter,

    /// Number of finalized blocks
    pub blocks_finalized: IntCounter,

    /// Number of timeouts detected
    pub timeouts_detected: IntCounter,

    /// Number of forced rotations
    pub forced_rotations: IntCounter,

    /// Number of validator set updates
    pub validator_set_updates: IntCounter,

    /// Block production time (seconds)
    pub block_production_time: Histogram,

    /// Attestation propagation time (seconds)
    pub attestation_time: Histogram,

    /// Finalization time (seconds)
    pub finalization_time: Histogram,

    /// Whether this node is currently the proposer
    pub is_proposer: IntGauge,

    /// Current uptime (basis points)
    pub uptime: IntGauge,
}

impl ConsensusMetrics {
    /// Create new metrics and register with registry
    ///
    /// # Errors
    ///
    /// Returns error if metrics registration fails
    pub fn new(registry: &Registry) -> Result<Self, prometheus::Error> {
        let metrics = Self {
            current_block: IntGauge::with_opts(
                Opts::new("consensus_current_block", "Current block number")
                    .namespace("ande")
                    .subsystem("consensus"),
            )?,

            current_epoch: IntGauge::with_opts(
                Opts::new("consensus_current_epoch", "Current epoch number")
                    .namespace("ande")
                    .subsystem("consensus"),
            )?,

            current_rotation: IntGauge::with_opts(
                Opts::new("consensus_current_rotation", "Current rotation number")
                    .namespace("ande")
                    .subsystem("consensus"),
            )?,

            active_validators: IntGauge::with_opts(
                Opts::new(
                    "consensus_active_validators",
                    "Number of active validators",
                )
                .namespace("ande")
                .subsystem("consensus"),
            )?,

            total_voting_power: IntGauge::with_opts(
                Opts::new("consensus_total_voting_power", "Total voting power")
                    .namespace("ande")
                    .subsystem("consensus"),
            )?,

            bft_threshold: IntGauge::with_opts(
                Opts::new("consensus_bft_threshold", "BFT threshold (2/3+1)")
                    .namespace("ande")
                    .subsystem("consensus"),
            )?,

            blocks_produced: IntCounter::with_opts(
                Opts::new(
                    "consensus_blocks_produced_total",
                    "Total blocks produced by this sequencer",
                )
                .namespace("ande")
                .subsystem("consensus"),
            )?,

            blocks_missed: IntCounter::with_opts(
                Opts::new(
                    "consensus_blocks_missed_total",
                    "Total blocks missed by this sequencer",
                )
                .namespace("ande")
                .subsystem("consensus"),
            )?,

            attestations_received: IntCounter::with_opts(
                Opts::new(
                    "consensus_attestations_received_total",
                    "Total attestations received",
                )
                .namespace("ande")
                .subsystem("consensus"),
            )?,

            blocks_finalized: IntCounter::with_opts(
                Opts::new(
                    "consensus_blocks_finalized_total",
                    "Total blocks finalized",
                )
                .namespace("ande")
                .subsystem("consensus"),
            )?,

            timeouts_detected: IntCounter::with_opts(
                Opts::new(
                    "consensus_timeouts_detected_total",
                    "Total timeouts detected",
                )
                .namespace("ande")
                .subsystem("consensus"),
            )?,

            forced_rotations: IntCounter::with_opts(
                Opts::new(
                    "consensus_forced_rotations_total",
                    "Total forced rotations",
                )
                .namespace("ande")
                .subsystem("consensus"),
            )?,

            validator_set_updates: IntCounter::with_opts(
                Opts::new(
                    "consensus_validator_set_updates_total",
                    "Total validator set updates",
                )
                .namespace("ande")
                .subsystem("consensus"),
            )?,

            block_production_time: Histogram::with_opts(
                HistogramOpts::new(
                    "consensus_block_production_duration_seconds",
                    "Block production time in seconds",
                )
                .namespace("ande")
                .subsystem("consensus"),
            )?,

            attestation_time: Histogram::with_opts(
                HistogramOpts::new(
                    "consensus_attestation_duration_seconds",
                    "Attestation propagation time in seconds",
                )
                .namespace("ande")
                .subsystem("consensus"),
            )?,

            finalization_time: Histogram::with_opts(
                HistogramOpts::new(
                    "consensus_finalization_duration_seconds",
                    "Block finalization time in seconds",
                )
                .namespace("ande")
                .subsystem("consensus"),
            )?,

            is_proposer: IntGauge::with_opts(
                Opts::new(
                    "consensus_is_proposer",
                    "Whether this node is currently the proposer (0/1)",
                )
                .namespace("ande")
                .subsystem("consensus"),
            )?,

            uptime: IntGauge::with_opts(
                Opts::new(
                    "consensus_uptime_basis_points",
                    "Current uptime in basis points (10000 = 100%)",
                )
                .namespace("ande")
                .subsystem("consensus"),
            )?,
        };

        // Register all metrics
        registry.register(Box::new(metrics.current_block.clone()))?;
        registry.register(Box::new(metrics.current_epoch.clone()))?;
        registry.register(Box::new(metrics.current_rotation.clone()))?;
        registry.register(Box::new(metrics.active_validators.clone()))?;
        registry.register(Box::new(metrics.total_voting_power.clone()))?;
        registry.register(Box::new(metrics.bft_threshold.clone()))?;
        registry.register(Box::new(metrics.blocks_produced.clone()))?;
        registry.register(Box::new(metrics.blocks_missed.clone()))?;
        registry.register(Box::new(metrics.attestations_received.clone()))?;
        registry.register(Box::new(metrics.blocks_finalized.clone()))?;
        registry.register(Box::new(metrics.timeouts_detected.clone()))?;
        registry.register(Box::new(metrics.forced_rotations.clone()))?;
        registry.register(Box::new(metrics.validator_set_updates.clone()))?;
        registry.register(Box::new(metrics.block_production_time.clone()))?;
        registry.register(Box::new(metrics.attestation_time.clone()))?;
        registry.register(Box::new(metrics.finalization_time.clone()))?;
        registry.register(Box::new(metrics.is_proposer.clone()))?;
        registry.register(Box::new(metrics.uptime.clone()))?;

        Ok(metrics)
    }
}
