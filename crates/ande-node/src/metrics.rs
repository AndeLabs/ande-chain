//! Enhanced metrics for ANDE Chain
//!
//! Comprehensive monitoring covering:
//! - Parallel execution performance
//! - MEV detection and value extraction
//! - Data Availability (Celestia) operations
//! - Consensus and validator metrics
//! - Network health and connectivity

use prometheus::{
    core::{AtomicU64, GenericCounter, GenericGauge},
    Histogram, HistogramOpts, IntCounter, IntCounterVec, IntGauge, IntGaugeVec, Opts, Registry,
};
use std::sync::Arc;

/// Central metrics registry for ANDE Chain
#[derive(Clone)]
pub struct AndeMetrics {
    /// Prometheus registry
    registry: Arc<Registry>,
    
    /// Parallel execution metrics
    pub parallel: ParallelExecutionMetrics,
    
    /// MEV protection metrics
    pub mev: MevMetrics,
    
    /// Data Availability metrics
    pub da: DataAvailabilityMetrics,
    
    /// Consensus metrics
    pub consensus: ConsensusMetrics,
    
    /// Network metrics
    pub network: NetworkMetrics,
    
    /// RPC metrics
    pub rpc: RpcMetrics,
}

/// Parallel execution performance metrics
#[derive(Clone)]
pub struct ParallelExecutionMetrics {
    /// Total successful parallel executions
    pub success_total: IntCounter,
    
    /// Total execution conflicts (rollbacks needed)
    pub conflicts_total: IntCounter,
    
    /// Duration of parallel execution (seconds)
    pub duration: Histogram,
    
    /// Number of active parallel workers
    pub workers_active: IntGauge,
    
    /// Transaction throughput (TPS)
    pub throughput: IntGauge,
    
    /// Conflict rate (percentage)
    pub conflict_rate: IntGauge,
    
    /// Batch size distribution
    pub batch_size: Histogram,
}

/// MEV protection and detection metrics
#[derive(Clone)]
pub struct MevMetrics {
    /// Total MEV bundles detected
    pub bundles_detected: IntCounter,
    
    /// Total MEV value extracted (wei)
    pub value_extracted: IntCounter,
    
    /// Number of searchers participating in auctions
    pub auction_participants: IntGauge,
    
    /// MEV auction duration
    pub auction_duration: Histogram,
    
    /// Validator MEV share (wei)
    pub validator_share: IntCounter,
    
    /// Protocol MEV share (wei)
    pub protocol_share: IntCounter,
}

/// Data Availability layer metrics
#[derive(Clone)]
pub struct DataAvailabilityMetrics {
    /// Total DA submissions
    pub submissions_total: IntCounter,
    
    /// Successful DA verifications
    pub verifications_success: IntCounter,
    
    /// Failed DA verifications
    pub verifications_failed: IntCounter,
    
    /// DA submission latency (seconds)
    pub submission_latency: Histogram,
    
    /// DA data size (bytes)
    pub data_size: Histogram,
    
    /// DA batch size (blocks per batch)
    pub batch_size: IntGauge,
    
    /// Current DA namespace height
    pub namespace_height: IntGauge,
}

/// Consensus layer metrics
#[derive(Clone)]
pub struct ConsensusMetrics {
    /// Blocks proposed by validators
    pub blocks_proposed: IntCounterVec,
    
    /// Attestations received
    pub attestations_received: IntCounterVec,
    
    /// Validator participation rate (0-100)
    pub participation_rate: IntGauge,
    
    /// Time to finality (seconds)
    pub finality_time: Histogram,
    
    /// Missed slots
    pub missed_slots: IntCounter,
    
    /// Current epoch
    pub current_epoch: IntGauge,
    
    /// Active validator count
    pub active_validators: IntGauge,
}

/// Network health metrics
#[derive(Clone)]
pub struct NetworkMetrics {
    /// Connected peer count
    pub peer_count: IntGauge,
    
    /// Messages received by type
    pub messages_received: IntCounterVec,
    
    /// Messages sent by type
    pub messages_sent: IntCounterVec,
    
    /// Network bandwidth in (bytes/sec)
    pub bandwidth_in: IntGauge,
    
    /// Network bandwidth out (bytes/sec)
    pub bandwidth_out: IntGauge,
    
    /// P2P connection errors
    pub connection_errors: IntCounter,
}

/// RPC endpoint metrics
#[derive(Clone)]
pub struct RpcMetrics {
    /// RPC requests by method
    pub requests_total: IntCounterVec,
    
    /// RPC request duration by method
    pub request_duration: Histogram,
    
    /// Rate limit hits
    pub rate_limit_hits: IntCounter,
    
    /// Active RPC connections
    pub active_connections: IntGauge,
}

impl AndeMetrics {
    /// Create a new metrics instance with all sub-metrics registered
    pub fn new() -> eyre::Result<Self> {
        let registry = Arc::new(Registry::new());
        
        let parallel = ParallelExecutionMetrics::new(&registry)?;
        let mev = MevMetrics::new(&registry)?;
        let da = DataAvailabilityMetrics::new(&registry)?;
        let consensus = ConsensusMetrics::new(&registry)?;
        let network = NetworkMetrics::new(&registry)?;
        let rpc = RpcMetrics::new(&registry)?;
        
        Ok(Self {
            registry,
            parallel,
            mev,
            da,
            consensus,
            network,
            rpc,
        })
    }
    
    /// Get the Prometheus registry for exporting metrics
    pub fn registry(&self) -> &Registry {
        &self.registry
    }
    
    /// Export all metrics in Prometheus format
    pub fn export(&self) -> String {
        use prometheus::Encoder;
        let encoder = prometheus::TextEncoder::new();
        let metric_families = self.registry.gather();
        
        let mut buffer = Vec::new();
        encoder.encode(&metric_families, &mut buffer).unwrap();
        String::from_utf8(buffer).unwrap()
    }
}

impl Default for AndeMetrics {
    fn default() -> Self {
        Self::new().expect("Failed to initialize metrics")
    }
}

impl ParallelExecutionMetrics {
    fn new(registry: &Registry) -> eyre::Result<Self> {
        let success_total = IntCounter::new(
            "ande_parallel_execution_success_total",
            "Total successful parallel executions",
        )?;
        registry.register(Box::new(success_total.clone()))?;
        
        let conflicts_total = IntCounter::new(
            "ande_parallel_execution_conflicts_total",
            "Total execution conflicts requiring rollback",
        )?;
        registry.register(Box::new(conflicts_total.clone()))?;
        
        let duration = Histogram::with_opts(
            HistogramOpts::new(
                "ande_parallel_execution_duration_seconds",
                "Duration of parallel execution batches",
            )
            .buckets(vec![0.001, 0.005, 0.01, 0.05, 0.1, 0.5, 1.0, 2.0]),
        )?;
        registry.register(Box::new(duration.clone()))?;
        
        let workers_active = IntGauge::new(
            "ande_parallel_workers_active",
            "Number of active parallel execution workers",
        )?;
        registry.register(Box::new(workers_active.clone()))?;
        
        let throughput = IntGauge::new(
            "ande_parallel_throughput_tps",
            "Current transaction throughput (TPS)",
        )?;
        registry.register(Box::new(throughput.clone()))?;
        
        let conflict_rate = IntGauge::new(
            "ande_parallel_conflict_rate_percent",
            "Percentage of transactions with conflicts",
        )?;
        registry.register(Box::new(conflict_rate.clone()))?;
        
        let batch_size = Histogram::with_opts(
            HistogramOpts::new(
                "ande_parallel_batch_size",
                "Distribution of parallel execution batch sizes",
            )
            .buckets(vec![10.0, 25.0, 50.0, 100.0, 250.0, 500.0, 1000.0]),
        )?;
        registry.register(Box::new(batch_size.clone()))?;
        
        Ok(Self {
            success_total,
            conflicts_total,
            duration,
            workers_active,
            throughput,
            conflict_rate,
            batch_size,
        })
    }
}

impl MevMetrics {
    fn new(registry: &Registry) -> eyre::Result<Self> {
        let bundles_detected = IntCounter::new(
            "ande_mev_bundles_detected_total",
            "Total MEV bundles detected",
        )?;
        registry.register(Box::new(bundles_detected.clone()))?;
        
        let value_extracted = IntCounter::new(
            "ande_mev_value_extracted_wei",
            "Total MEV value extracted in wei",
        )?;
        registry.register(Box::new(value_extracted.clone()))?;
        
        let auction_participants = IntGauge::new(
            "ande_mev_auction_participants",
            "Number of searchers in current auction",
        )?;
        registry.register(Box::new(auction_participants.clone()))?;
        
        let auction_duration = Histogram::with_opts(
            HistogramOpts::new(
                "ande_mev_auction_duration_seconds",
                "Duration of MEV auctions",
            )
            .buckets(vec![0.5, 1.0, 2.0, 3.0, 5.0, 10.0]),
        )?;
        registry.register(Box::new(auction_duration.clone()))?;
        
        let validator_share = IntCounter::new(
            "ande_mev_validator_share_wei",
            "MEV value distributed to validators",
        )?;
        registry.register(Box::new(validator_share.clone()))?;
        
        let protocol_share = IntCounter::new(
            "ande_mev_protocol_share_wei",
            "MEV value distributed to protocol treasury",
        )?;
        registry.register(Box::new(protocol_share.clone()))?;
        
        Ok(Self {
            bundles_detected,
            value_extracted,
            auction_participants,
            auction_duration,
            validator_share,
            protocol_share,
        })
    }
}

impl DataAvailabilityMetrics {
    fn new(registry: &Registry) -> eyre::Result<Self> {
        let submissions_total = IntCounter::new(
            "ande_da_submissions_total",
            "Total DA layer submissions",
        )?;
        registry.register(Box::new(submissions_total.clone()))?;
        
        let verifications_success = IntCounter::new(
            "ande_da_verifications_success_total",
            "Successful DA verifications",
        )?;
        registry.register(Box::new(verifications_success.clone()))?;
        
        let verifications_failed = IntCounter::new(
            "ande_da_verifications_failed_total",
            "Failed DA verifications",
        )?;
        registry.register(Box::new(verifications_failed.clone()))?;
        
        let submission_latency = Histogram::with_opts(
            HistogramOpts::new(
                "ande_da_submission_latency_seconds",
                "Latency of DA submissions to Celestia",
            )
            .buckets(vec![0.1, 0.5, 1.0, 2.0, 5.0, 10.0, 30.0, 60.0]),
        )?;
        registry.register(Box::new(submission_latency.clone()))?;
        
        let data_size = Histogram::with_opts(
            HistogramOpts::new(
                "ande_da_data_size_bytes",
                "Size of DA submissions in bytes",
            )
            .buckets(vec![1024.0, 10240.0, 102400.0, 1048576.0, 10485760.0]),
        )?;
        registry.register(Box::new(data_size.clone()))?;
        
        let batch_size = IntGauge::new(
            "ande_da_batch_size",
            "Number of blocks in current DA batch",
        )?;
        registry.register(Box::new(batch_size.clone()))?;
        
        let namespace_height = IntGauge::new(
            "ande_da_namespace_height",
            "Current height of DA namespace",
        )?;
        registry.register(Box::new(namespace_height.clone()))?;
        
        Ok(Self {
            submissions_total,
            verifications_success,
            verifications_failed,
            submission_latency,
            data_size,
            batch_size,
            namespace_height,
        })
    }
}

impl ConsensusMetrics {
    fn new(registry: &Registry) -> eyre::Result<Self> {
        let blocks_proposed = IntCounterVec::new(
            Opts::new("ande_blocks_proposed_total", "Blocks proposed by validator"),
            &["validator"],
        )?;
        registry.register(Box::new(blocks_proposed.clone()))?;
        
        let attestations_received = IntCounterVec::new(
            Opts::new("ande_attestations_received_total", "Attestations received"),
            &["validator"],
        )?;
        registry.register(Box::new(attestations_received.clone()))?;
        
        let participation_rate = IntGauge::new(
            "ande_validator_participation_rate_percent",
            "Percentage of validators participating",
        )?;
        registry.register(Box::new(participation_rate.clone()))?;
        
        let finality_time = Histogram::with_opts(
            HistogramOpts::new(
                "ande_finality_time_seconds",
                "Time from block proposal to finality",
            )
            .buckets(vec![1.0, 2.0, 3.0, 5.0, 10.0, 30.0, 60.0]),
        )?;
        registry.register(Box::new(finality_time.clone()))?;
        
        let missed_slots = IntCounter::new(
            "ande_missed_slots_total",
            "Total missed consensus slots",
        )?;
        registry.register(Box::new(missed_slots.clone()))?;
        
        let current_epoch = IntGauge::new(
            "ande_current_epoch",
            "Current consensus epoch",
        )?;
        registry.register(Box::new(current_epoch.clone()))?;
        
        let active_validators = IntGauge::new(
            "ande_active_validators",
            "Number of active validators",
        )?;
        registry.register(Box::new(active_validators.clone()))?;
        
        Ok(Self {
            blocks_proposed,
            attestations_received,
            participation_rate,
            finality_time,
            missed_slots,
            current_epoch,
            active_validators,
        })
    }
}

impl NetworkMetrics {
    fn new(registry: &Registry) -> eyre::Result<Self> {
        let peer_count = IntGauge::new(
            "ande_peer_count",
            "Number of connected peers",
        )?;
        registry.register(Box::new(peer_count.clone()))?;
        
        let messages_received = IntCounterVec::new(
            Opts::new("ande_messages_received_total", "Messages received by type"),
            &["message_type"],
        )?;
        registry.register(Box::new(messages_received.clone()))?;
        
        let messages_sent = IntCounterVec::new(
            Opts::new("ande_messages_sent_total", "Messages sent by type"),
            &["message_type"],
        )?;
        registry.register(Box::new(messages_sent.clone()))?;
        
        let bandwidth_in = IntGauge::new(
            "ande_network_bandwidth_in_bytes_per_sec",
            "Inbound network bandwidth",
        )?;
        registry.register(Box::new(bandwidth_in.clone()))?;
        
        let bandwidth_out = IntGauge::new(
            "ande_network_bandwidth_out_bytes_per_sec",
            "Outbound network bandwidth",
        )?;
        registry.register(Box::new(bandwidth_out.clone()))?;
        
        let connection_errors = IntCounter::new(
            "ande_network_connection_errors_total",
            "Total P2P connection errors",
        )?;
        registry.register(Box::new(connection_errors.clone()))?;
        
        Ok(Self {
            peer_count,
            messages_received,
            messages_sent,
            bandwidth_in,
            bandwidth_out,
            connection_errors,
        })
    }
}

impl RpcMetrics {
    fn new(registry: &Registry) -> eyre::Result<Self> {
        let requests_total = IntCounterVec::new(
            Opts::new("ande_rpc_requests_total", "RPC requests by method"),
            &["method"],
        )?;
        registry.register(Box::new(requests_total.clone()))?;
        
        let request_duration = Histogram::with_opts(
            HistogramOpts::new(
                "ande_rpc_request_duration_seconds",
                "RPC request duration",
            )
            .buckets(vec![0.001, 0.01, 0.1, 0.5, 1.0, 5.0, 10.0]),
        )?;
        registry.register(Box::new(request_duration.clone()))?;
        
        let rate_limit_hits = IntCounter::new(
            "ande_rpc_rate_limit_hits_total",
            "Number of rate limit rejections",
        )?;
        registry.register(Box::new(rate_limit_hits.clone()))?;
        
        let active_connections = IntGauge::new(
            "ande_rpc_active_connections",
            "Number of active RPC connections",
        )?;
        registry.register(Box::new(active_connections.clone()))?;
        
        Ok(Self {
            requests_total,
            request_duration,
            rate_limit_hits,
            active_connections,
        })
    }
}
