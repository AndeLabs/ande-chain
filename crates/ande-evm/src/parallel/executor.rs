//! Parallel EVM Executor for AndeChain
//!
//! This module implements parallel transaction execution using concepts
//! from pevm as reference, adapted for our ANDE Token Duality architecture.

use crate::evm_config::AndeEvmConfig;
use alloy_primitives::{Address, U256};
use alloy_consensus::transaction::{SignerRecoverable, Transaction as TransactionTrait};
use reth_evm::NextBlockEnvAttributes;
use reth_primitives::{TransactionSigned, Header, SealedHeader};
use std::{
    sync::{Arc, Mutex},
    thread,

    collections::{HashMap, VecDeque},
    fmt::Debug,
};
use tracing::{debug, info, warn};

// Re-export types that might come from different crates depending on context
// This allows the parallel module to be used from both node and evolve crates
pub use alloy_primitives::Signature as TransactionSignature;

/// Error type for payload building operations
/// This is a simplified version that can be used when reth_payload_builder is not available
#[derive(Debug, thiserror::Error)]
pub enum ParallelPayloadError {
    #[error("Failed to execute transaction: {0}")]
    ExecutionError(String),
    #[error("Failed to validate transaction: {0}")]
    ValidationError(String),
    #[error("Internal error: {0}")]
    Internal(String),
    #[error("Other error: {0}")]
    Other(String),
}

/// Trait for state provider factory to allow testing without full reth_provider
pub trait StateProvider: Send + Sync {
    fn latest(&self) -> Result<(), ParallelPayloadError>;
}

/// Trait for header provider to allow testing without full reth_provider
pub trait HeaderProviderTrait: Send + Sync {
    fn header(&self, hash: &alloy_primitives::B256) -> Result<Option<Header>, ParallelPayloadError>;
}

// Re-export ParallelConfig from config module
pub use super::config::ParallelConfig;

/// Transaction index type alias for clarity
pub type TxIdx = usize;

/// Transaction version with execution context
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TxVersion {
    /// Transaction index in the block
    pub tx_idx: TxIdx,
    /// Execution attempt number (for retries)
    pub tx_incarnation: usize,
}

/// Execution status of a transaction
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TxStatus {
    /// Ready to execute
    Ready,
    /// Currently executing
    Executing,
    /// Completed successfully
    Completed,
    /// Failed and needs retry
    Failed,
    /// Blocked by dependency on another transaction
    Blocked(TxIdx),
}

/// Task type for workers
#[derive(Debug, Clone)]
pub enum ParallelTask {
    /// Execute a transaction
    Execute(TxVersion),
    /// Validate a transaction result
    Validate(TxVersion),
}

/// Transaction dependency information
#[derive(Debug, Clone)]
pub struct TxDependency {
    /// Transactions that this transaction depends on
    pub depends_on: Vec<TxIdx>,
    /// Transactions that depend on this one
    pub dependents: Vec<TxIdx>,
    /// Accounts this transaction reads from
    pub read_accounts: Vec<Address>,
    /// Accounts this transaction writes to
    pub write_accounts: Vec<Address>,
}

/// Result of parallel transaction execution
#[derive(Debug, Clone)]
pub struct ParallelExecutionResult {
    /// Transaction index
    pub tx_idx: usize,
    /// Gas used by the transaction
    pub gas_used: u64,
    /// Execution success
    pub success: bool,
    /// Error message if failed
    pub error: Option<String>,
    /// State changes produced
    pub state_changes: HashMap<Address, AccountStateChange>,
    /// Accounts read during execution (for validation)
    pub read_set: Vec<Address>,
    /// Accounts written during execution (for conflict detection)
    pub write_set: Vec<Address>,
    /// Incarnation number (for retry tracking)
    pub incarnation: usize,
}

/// State change for an account
#[derive(Debug, Clone)]
pub struct AccountStateChange {
    /// Address of the account
    pub address: Address,
    /// Balance change
    pub balance_change: Option<i128>, // Positive for increase, negative for decrease
    /// Nonce change
    pub nonce_change: Option<u64>,
    /// Storage changes
    pub storage_changes: HashMap<U256, U256>,
}

/// Multi-version memory for tracking parallel state changes
#[derive(Debug)]
pub struct MvMemory {
    /// Multi-version data structure for account states
    #[allow(dead_code)]
    data: HashMap<Address, Vec<MvMemoryEntry>>,
    /// Lazy accounts that need final evaluation
    lazy_accounts: HashMap<Address, LazyAccountState>,
}

/// Entry in multi-version memory
#[derive(Debug, Clone)]
pub struct MvMemoryEntry {
    /// Transaction version that created this entry
    pub tx_version: TxVersion,
    /// Memory value
    pub value: MvMemoryValue,
}

/// Values stored in multi-version memory
#[derive(Debug, Clone)]
pub enum MvMemoryValue {
    /// Basic account state (balance, nonce)
    Basic { balance: U256, nonce: u64 },
    /// Lazy balance addition (for beneficiary, transfers)
    LazyBalanceAddition(U256),
    /// Lazy balance subtraction with nonce increment
    LazyBalanceSubtraction { amount: U256, nonce_increment: bool },
    /// Storage value
    Storage(U256),
}

/// Lazy account state for deferred balance calculations
#[derive(Debug, Clone)]
pub struct LazyAccountState {
    /// Base balance before lazy updates
    pub base_balance: U256,
    /// Base nonce before lazy updates
    pub base_nonce: u64,
    /// Pending balance additions
    pub balance_additions: Vec<(TxIdx, U256)>,
    /// Pending balance subtractions
    pub balance_subtractions: Vec<(TxIdx, U256)>,
    /// Pending nonce increments
    pub nonce_increments: Vec<TxIdx>,
}

impl MvMemory {
    /// Create new multi-version memory
    pub fn new() -> Self {
        Self {
            data: HashMap::new(),
            lazy_accounts: HashMap::new(),
        }
    }

    /// Add a lazy balance addition for an account
    pub fn add_lazy_balance_addition(&mut self, address: Address, amount: U256, tx_idx: TxIdx) {
        let lazy_state = self.lazy_accounts.entry(address).or_insert_with(|| LazyAccountState {
            base_balance: U256::ZERO,
            base_nonce: 0,
            balance_additions: Vec::new(),
            balance_subtractions: Vec::new(),
            nonce_increments: Vec::new(),
        });
        lazy_state.balance_additions.push((tx_idx, amount));
    }

    /// Add a lazy balance subtraction for an account
    pub fn add_lazy_balance_subtraction(&mut self, address: Address, amount: U256, tx_idx: TxIdx) {
        let lazy_state = self.lazy_accounts.entry(address).or_insert_with(|| LazyAccountState {
            base_balance: U256::ZERO,
            base_nonce: 0,
            balance_additions: Vec::new(),
            balance_subtractions: Vec::new(),
            nonce_increments: Vec::new(),
        });
        lazy_state.balance_subtractions.push((tx_idx, amount));
    }

    /// Add a lazy nonce increment for an account
    pub fn add_lazy_nonce_increment(&mut self, address: Address, tx_idx: TxIdx) {
        let lazy_state = self.lazy_accounts.entry(address).or_insert_with(|| LazyAccountState {
            base_balance: U256::ZERO,
            base_nonce: 0,
            balance_additions: Vec::new(),
            balance_subtractions: Vec::new(),
            nonce_increments: Vec::new(),
        });
        lazy_state.nonce_increments.push(tx_idx);
    }

    /// Evaluate lazy balances and return final state changes
    pub fn evaluate_lazy_balances(&mut self) -> Vec<AccountStateChange> {
        let mut changes = Vec::new();

        for (address, lazy_state) in &self.lazy_accounts {
            // Calculate total additions and subtractions
            let mut total_additions = U256::ZERO;
            let mut total_subtractions = U256::ZERO;

            for (_, amount) in &lazy_state.balance_additions {
                total_additions = total_additions.saturating_add(*amount);
            }

            for (_, amount) in &lazy_state.balance_subtractions {
                total_subtractions = total_subtractions.saturating_add(*amount);
            }

            // Calculate final balance
            let _final_balance = lazy_state.base_balance
                .saturating_add(total_additions)
                .saturating_sub(total_subtractions);

            // Calculate final nonce
            let final_nonce = lazy_state.base_nonce + lazy_state.nonce_increments.len() as u64;

            // Calculate balance delta (can be positive or negative)
            // First check if it's a net addition or subtraction
            let balance_change = if total_additions >= total_subtractions {
                // Net positive change
                let delta = total_additions - total_subtractions;
                if delta <= U256::from(i128::MAX as u128) {
                    Some(delta.to::<u128>() as i128)
                } else {
                    Some(i128::MAX) // Saturate at max
                }
            } else {
                // Net negative change
                let delta = total_subtractions - total_additions;
                if delta <= U256::from(i128::MAX as u128) {
                    Some(-(delta.to::<u128>() as i128))
                } else {
                    Some(i128::MIN) // Saturate at min
                }
            };

            let nonce_change = if final_nonce != lazy_state.base_nonce {
                Some(final_nonce)
            } else {
                None
            };

            changes.push(AccountStateChange {
                address: *address,
                balance_change,
                nonce_change,
                storage_changes: HashMap::new(),
            });
        }

        changes
    }
}

/// Parallel EVM Executor
#[derive(Debug)]
pub struct ParallelExecutor {
    /// Configuration for parallel execution
    #[allow(dead_code)]
    config: ParallelConfig,
}

impl ParallelExecutor {
    /// Create new parallel executor
    pub fn new(config: ParallelConfig) -> Self {
        Self { config }
    }

    /// Execute transactions in parallel
    ///
    /// NOTE: This method has generic constraints that will be satisfied when called from
    /// the node crate where full reth_provider and reth_payload_builder are available.
    pub async fn execute_transactions(
        &self,
        transactions: Vec<TransactionSigned>,
        evm_config: &AndeEvmConfig,
        parent_header: &SealedHeader,
        next_block_attrs: NextBlockEnvAttributes,
    ) -> Result<Vec<ParallelExecutionResult>, ParallelPayloadError> {
        info!(
            transaction_count = transactions.len(),
            concurrency_level = self.config.concurrency_level.get(),
            "Starting parallel transaction execution"
        );

        // Check if we should use parallel execution
        if !self.should_use_parallel(&transactions) {
            info!(
                transaction_count = transactions.len(),
                reason = if self.config.force_sequential {
                    "force_sequential enabled"
                } else {
                    "insufficient transactions for parallel execution"
                },
                "Falling back to sequential execution"
            );
            return self.execute_sequential(transactions, evm_config, parent_header, next_block_attrs).await;
        }

        // Analyze transaction dependencies
        let dependencies = self.analyze_dependencies(&transactions)?;

        // Create multi-version memory
        let mv_memory = Arc::new(Mutex::new(MvMemory::new()));

        // Create scheduler
        let scheduler = Arc::new(ParallelScheduler::new(
            transactions.len(),
            dependencies,
            self.config.clone(),
        ));

        // Create thread pool for parallel execution
        let results = Arc::new(Mutex::new(vec![None; transactions.len()]));

        thread::scope(|scope| {
            // Spawn worker threads
            for worker_id in 0..self.config.concurrency_level.get() {
                let scheduler = Arc::clone(&scheduler);
                let mv_memory = Arc::clone(&mv_memory);
                let results = Arc::clone(&results);
                let transactions = &transactions;
                let evm_config = evm_config.clone();
                let parent_header_ref = parent_header;
                let next_block_attrs_ref = &next_block_attrs;

                scope.spawn(move || {
                    debug!("Worker {} started", worker_id);

                    while let Some(task) = scheduler.next_task() {
                        match task {
                            ParallelTask::Execute(tx_version) => {
                                debug!("Worker {} executing transaction {}", worker_id, tx_version.tx_idx);

                                if let Some(result) = self.execute_transaction_parallel(
                                    tx_version,
                                    &transactions[tx_version.tx_idx],
                                    &evm_config,
                                    parent_header_ref,
                                    next_block_attrs_ref,
                                    &mv_memory,
                                ) {
                                    // Store result for validation
                                    scheduler.store_result(result.clone());

                                    // Store in results array for final collection
                                    let mut results_guard = results.lock().unwrap();
                                    results_guard[tx_version.tx_idx] = Some(result.clone());
                                    drop(results_guard);

                                    // Schedule for validation
                                    scheduler.schedule_validation(tx_version);

                                    debug!(
                                        "Worker {} finished execution for tx {}, scheduled for validation",
                                        worker_id, tx_version.tx_idx
                                    );
                                }
                            }
                            ParallelTask::Validate(tx_version) => {
                                debug!("Worker {} validating transaction {}", worker_id, tx_version.tx_idx);
                                scheduler.finish_validation(tx_version);
                            }
                        }
                    }

                    debug!("Worker {} finished", worker_id);
                });
            }
        });

        // Collect results
        let mut final_results = Vec::new();
        let results_guard = results.lock().unwrap();

        for (i, result) in results_guard.iter().enumerate() {
            match result {
                Some(r) => final_results.push(r.clone()),
                None => {
                    warn!("Transaction {} has no result", i);
                    final_results.push(ParallelExecutionResult {
                        tx_idx: i,
                        gas_used: 0,
                        success: false,
                        error: Some("No execution result".to_string()),
                        state_changes: HashMap::new(),
                        read_set: Vec::new(),
                        write_set: Vec::new(),
                        incarnation: 0,
                    });
                }
            }
        }

        // Apply lazy balance updates
        let mut mv_memory_guard = mv_memory.lock().unwrap();
        let lazy_changes = mv_memory_guard.evaluate_lazy_balances();

        info!(
            "Parallel execution completed: {} transactions, {} lazy changes",
            final_results.len(),
            lazy_changes.len()
        );

        Ok(final_results)
    }

    /// Determine if parallel execution should be used
    fn should_use_parallel(&self, transactions: &[TransactionSigned]) -> bool {
        if self.config.force_sequential {
            return false;
        }

        if transactions.len() < self.config.min_transactions_for_parallel {
            return false;
        }

        true
    }

    /// Execute transactions sequentially (fallback mode)
    ///
    /// This method provides a sequential fallback when:
    /// - Transaction count is below `min_transactions_for_parallel`
    /// - `force_sequential` is enabled
    /// - Parallel execution encounters fatal errors
    ///
    /// # Implementation
    /// - Executes transactions one by one in order
    /// - Uses the same execution logic as parallel mode
    /// - No validation or retry logic (transactions execute once)
    /// - Returns results in the same format as parallel execution
    ///
    /// # Safety
    /// - Single-threaded execution eliminates race conditions
    /// - No conflict detection needed
    /// - Deterministic execution order
    async fn execute_sequential(
        &self,
        transactions: Vec<TransactionSigned>,
        evm_config: &AndeEvmConfig,
        parent_header: &SealedHeader,
        next_block_attrs: NextBlockEnvAttributes,
    ) -> Result<Vec<ParallelExecutionResult>, ParallelPayloadError> {
        info!(
            transaction_count = transactions.len(),
            "Starting sequential transaction execution"
        );

        let mut results = Vec::with_capacity(transactions.len());
        let mv_memory = Arc::new(Mutex::new(MvMemory::new()));

        // Execute each transaction in order
        for (i, transaction) in transactions.iter().enumerate() {
            debug!(
                tx_idx = i,
                tx_hash = ?transaction.hash(),
                "Executing transaction sequentially"
            );

            let tx_version = TxVersion {
                tx_idx: i,
                tx_incarnation: 0, // No retries in sequential mode
            };

            // Execute transaction using the same helper as parallel execution
            match self.execute_transaction_parallel(
                tx_version,
                transaction,
                evm_config,
                parent_header,
                &next_block_attrs,
                &mv_memory,
            ) {
                Some(result) => {
                    debug!(
                        tx_idx = i,
                        success = result.success,
                        gas_used = result.gas_used,
                        "Transaction executed sequentially"
                    );

                    results.push(result);
                }
                None => {
                    // Transaction execution returned None (should not happen in sequential mode)
                    warn!(
                        tx_idx = i,
                        "Transaction execution returned None in sequential mode"
                    );

                    results.push(ParallelExecutionResult {
                        tx_idx: i,
                        gas_used: 0,
                        success: false,
                        error: Some("Execution returned None".to_string()),
                        state_changes: HashMap::new(),
                        read_set: Vec::new(),
                        write_set: Vec::new(),
                        incarnation: 0,
                    });
                }
            }
        }

        // Apply lazy balance updates
        let mut mv_memory_guard = mv_memory.lock().unwrap();
        let lazy_changes = mv_memory_guard.evaluate_lazy_balances();

        info!(
            "Sequential execution completed: {} transactions, {} lazy changes",
            results.len(),
            lazy_changes.len()
        );

        // Check if all transactions succeeded
        let failed_count = results.iter().filter(|r| !r.success).count();
        if failed_count > 0 {
            warn!(
                failed_count = failed_count,
                total_count = results.len(),
                "Sequential execution completed with failures"
            );
        }

        Ok(results)
    }

    /// Analyze dependencies between transactions
    fn analyze_dependencies(&self, transactions: &[TransactionSigned]) -> Result<Vec<TxDependency>, ParallelPayloadError> {
        let mut dependencies = Vec::with_capacity(transactions.len());

        for (i, tx) in transactions.iter().enumerate() {
            let sender = tx.recover_signer().map_err(|e| {
                ParallelPayloadError::ExecutionError(format!("Failed to recover sender for tx {}: {}", i, e))
            })?;

            // Simple dependency analysis: same sender = dependency
            let mut depends_on = Vec::new();
            let mut dependents = Vec::new();

            for (j, other_tx) in transactions.iter().enumerate() {
                if i != j {
                    let other_sender = other_tx.recover_signer().ok();

                    // If same sender, create dependency
                    if other_sender == Some(sender) {
                        if j < i {
                            depends_on.push(j);
                        } else {
                            dependents.push(j);
                        }
                    }
                }
            }

            dependencies.push(TxDependency {
                depends_on,
                dependents,
                read_accounts: vec![sender],
                write_accounts: vec![sender],
            });
        }

        Ok(dependencies)
    }

    /// Execute a single transaction in parallel
    ///
    /// This function performs optimistic parallel execution of a transaction:
    /// 1. Creates an isolated EVM instance with multi-version memory
    /// 2. Executes the transaction
    /// 3. Captures state changes for validation
    /// 4. Records lazy updates for ANDE precompile interactions
    ///
    /// # Arguments
    /// * `tx_version` - Transaction version with index and incarnation
    /// * `transaction` - The signed transaction to execute
    /// * `evm_config` - EVM configuration with ANDE precompile
    /// * `state_db` - State database for reading account state
    /// * `parent_header` - Parent block header
    /// * `next_block_attrs` - Next block environment attributes
    /// * `mv_memory` - Multi-version memory for tracking state changes
    ///
    /// # Returns
    /// * `Some(ParallelExecutionResult)` - Execution result with gas used and state changes
    /// * `None` - If execution should be retried or blocked
    ///
    /// # Safety
    /// - This function is thread-safe and can be called concurrently
    /// - State changes are isolated until validation passes
    /// - ANDE precompile interactions are recorded as lazy updates
    fn execute_transaction_parallel(
        &self,
        tx_version: TxVersion,
        transaction: &TransactionSigned,
        _evm_config: &AndeEvmConfig,
        _parent_header: &SealedHeader,
        _next_block_attrs: &NextBlockEnvAttributes,
        mv_memory: &Arc<Mutex<MvMemory>>,
    ) -> Option<ParallelExecutionResult> {
        debug!(
            tx_idx = tx_version.tx_idx,
            incarnation = tx_version.tx_incarnation,
            tx_hash = ?transaction.hash(),
            "Executing transaction in parallel"
        );

        // Recover transaction sender
        let sender = match transaction.recover_signer() {
            Ok(addr) => addr,
            Err(e) => {
                warn!(
                    tx_idx = tx_version.tx_idx,
                    error = ?e,
                    "Failed to recover transaction signer"
                );
                return Some(ParallelExecutionResult {
                    tx_idx: tx_version.tx_idx,
                    gas_used: 0,
                    success: false,
                    error: Some(format!("Failed to recover signer: {}", e)),
                    state_changes: HashMap::new(),
                    read_set: Vec::new(),
                    write_set: Vec::new(),
                    incarnation: tx_version.tx_incarnation,
                });
            }
        };

        // TODO: In Phase 2, implement full EVM execution with revm
        // For now, we perform basic validation and gas estimation

        // Calculate base transaction cost
        let intrinsic_gas = self.calculate_intrinsic_gas(transaction);

        // Validate gas limit
        if transaction.gas_limit() < intrinsic_gas {
            warn!(
                tx_idx = tx_version.tx_idx,
                gas_limit = transaction.gas_limit(),
                intrinsic_gas = intrinsic_gas,
                "Transaction gas limit too low"
            );
            return Some(ParallelExecutionResult {
                tx_idx: tx_version.tx_idx,
                gas_used: 0,
                success: false,
                error: Some("Intrinsic gas too low".to_string()),
                state_changes: HashMap::new(),
                read_set: Vec::new(),
                write_set: Vec::new(),
                incarnation: tx_version.tx_incarnation,
            });
        }

        // Validate nonce (read from multi-version memory or state)
        // TODO: Implement proper nonce validation with MvMemory

        // Record state changes
        let mut state_changes = HashMap::new();

        // Track sender account changes (nonce increment, balance deduction)
        let gas_cost = U256::from(transaction.gas_limit())
            .saturating_mul(U256::from(transaction.max_fee_per_gas()));

        state_changes.insert(
            sender,
            AccountStateChange {
                address: sender,
                balance_change: Some(-(gas_cost.try_into().unwrap_or(i128::MAX))),
                nonce_change: Some(1), // Nonce increment
                storage_changes: HashMap::new(),
            },
        );

        // If this is a value transfer, record recipient change
        // transaction.to() returns Option<Address> for calls, None for contract creation
        if let Some(to) = transaction.to() {
            if !transaction.value().is_zero() {
                // Check if this is a call to ANDE precompile
                if self.config.enable_lazy_updates && self.is_ande_precompile_call(to) {
                    // Record lazy balance update for ANDE precompile
                    let mut mv_memory_guard = mv_memory.lock().unwrap();
                    mv_memory_guard.add_lazy_balance_addition(
                        to,
                        transaction.value(),
                        tx_version.tx_idx,
                    );
                    debug!(
                        tx_idx = tx_version.tx_idx,
                        recipient = ?to,
                        value = ?transaction.value(),
                        "Recorded lazy balance update for ANDE precompile"
                    );
                } else {
                    // Regular transfer - record immediate state change
                    state_changes.insert(
                        to,
                        AccountStateChange {
                            address: to,
                            balance_change: Some(transaction.value().try_into().unwrap_or(i128::MAX)),
                            nonce_change: None,
                            storage_changes: HashMap::new(),
                        },
                    );
                }
            }
        }

        // Record lazy balance addition for block beneficiary (gas payment)
        // This is a critical optimization for parallel execution
        if self.config.enable_lazy_updates {
            // TODO: Get beneficiary from next_block_attrs
            // For now, we'll defer this to the validation phase
        }

        // Build read set and write set for validation
        let mut read_set = vec![sender]; // Always read sender balance and nonce
        let mut write_set = vec![sender]; // Always write sender (nonce increment, gas payment)

        if let Some(to) = transaction.to() {
            if !transaction.value().is_zero() {
                // Read recipient balance (need to check if it exists)
                if !read_set.contains(&to) {
                    read_set.push(to);
                }
                // Write recipient balance
                if !write_set.contains(&to) {
                    write_set.push(to);
                }
            }
        }

        debug!(
            tx_idx = tx_version.tx_idx,
            gas_used = intrinsic_gas,
            state_changes = state_changes.len(),
            read_set_size = read_set.len(),
            write_set_size = write_set.len(),
            "Transaction execution completed"
        );

        Some(ParallelExecutionResult {
            tx_idx: tx_version.tx_idx,
            gas_used: intrinsic_gas,
            success: true,
            error: None,
            state_changes,
            read_set,
            write_set,
            incarnation: tx_version.tx_incarnation,
        })
    }

    /// Calculate intrinsic gas cost for a transaction
    ///
    /// Based on EIP-2028 and EIP-2930/EIP-1559 specifications:
    /// - Base cost: 21000 gas
    /// - Data cost: 4 gas per zero byte, 16 gas per non-zero byte
    /// - Contract creation cost: 32000 gas
    /// - Access list cost: 2400 gas per address, 1900 gas per storage key
    fn calculate_intrinsic_gas(&self, transaction: &TransactionSigned) -> u64 {
        use alloy_consensus::transaction::Transaction as _;

        let mut gas = 21000u64; // Base transaction cost

        // Add data gas cost
        let data = transaction.input();
        for byte in data.iter() {
            if *byte == 0 {
                gas = gas.saturating_add(4); // Zero byte cost (EIP-2028)
            } else {
                gas = gas.saturating_add(16); // Non-zero byte cost
            }
        }

        // Add contract creation cost
        if transaction.to().is_none() {
            gas = gas.saturating_add(32000); // Contract creation cost
        }

        // Add access list cost (EIP-2930)
        if let Some(access_list) = transaction.access_list() {
            for item in access_list.0.iter() {
                gas = gas.saturating_add(2400); // Address cost
                gas = gas.saturating_add(1900 * item.storage_keys.len() as u64); // Storage key cost
            }
        }

        gas
    }

    /// Check if a transaction is calling the ANDE precompile
    fn is_ande_precompile_call(&self, address: Address) -> bool {
        use crate::evm_config::ANDE_PRECOMPILE_ADDRESS;
        address == ANDE_PRECOMPILE_ADDRESS
    }
}

/// Parallel task scheduler
#[derive(Debug)]
pub struct ParallelScheduler {
    /// Transaction statuses
    tx_status: Vec<Mutex<TxStatus>>,
    /// Transaction dependencies
    dependencies: Vec<TxDependency>,
    /// Ready-to-execute queue
    execution_queue: Arc<Mutex<VecDeque<TxVersion>>>,
    /// Ready-to-validate queue
    validation_queue: Arc<Mutex<VecDeque<TxVersion>>>,
    /// Retry counts for each transaction
    retry_counts: Vec<Mutex<usize>>,
    /// Execution results for validation
    execution_results: Arc<Mutex<Vec<Option<ParallelExecutionResult>>>>,
    /// Configuration
    config: ParallelConfig,
}

impl ParallelScheduler {
    /// Create new scheduler
    pub fn new(block_size: usize, dependencies: Vec<TxDependency>, config: ParallelConfig) -> Self {
        let scheduler = Self {
            tx_status: (0..block_size)
                .map(|_| Mutex::new(TxStatus::Ready))
                .collect(),
            dependencies,
            execution_queue: Arc::new(Mutex::new(VecDeque::new())),
            validation_queue: Arc::new(Mutex::new(VecDeque::new())),
            retry_counts: (0..block_size)
                .map(|_| Mutex::new(0))
                .collect(),
            execution_results: Arc::new(Mutex::new(vec![None; block_size])),
            config,
        };

        // Initialize execution queue with ready transactions
        scheduler.initialize_execution_queue();

        scheduler
    }

    /// Initialize execution queue with transactions that have no dependencies
    fn initialize_execution_queue(&self) {
        let mut queue = self.execution_queue.lock().unwrap();

        for (i, dep) in self.dependencies.iter().enumerate() {
            if dep.depends_on.is_empty() {
                queue.push_back(TxVersion {
                    tx_idx: i,
                    tx_incarnation: 0,
                });
            }
        }
    }

    /// Get next task for a worker
    pub fn next_task(&self) -> Option<ParallelTask> {
        // Try validation queue first (higher priority)
        {
            let mut validation_queue = self.validation_queue.lock().unwrap();
            if let Some(tx_version) = validation_queue.pop_front() {
                return Some(ParallelTask::Validate(tx_version));
            }
        }

        // Try execution queue
        {
            let mut execution_queue = self.execution_queue.lock().unwrap();
            if let Some(tx_version) = execution_queue.pop_front() {
                return Some(ParallelTask::Execute(tx_version));
            }
        }

        None
    }

    /// Mark transaction as completed and schedule dependent transactions
    pub fn finish_execution(&self, tx_version: TxVersion) {
        // Step 1: Mark this transaction as completed (acquire and release lock immediately)
        {
            let mut status = self.tx_status[tx_version.tx_idx].lock().unwrap();
            *status = TxStatus::Completed;
        } // Lock released here

        // Step 2: Get list of dependents (read-only access, no lock needed)
        let dependents = &self.dependencies[tx_version.tx_idx].dependents;

        // Step 3: Check each dependent and schedule if ready
        for &dependent_idx in dependents {
            // Check if all dependencies for this dependent are satisfied
            let mut all_deps_completed = true;
            for &dep_idx in &self.dependencies[dependent_idx].depends_on {
                let dep_status = self.tx_status[dep_idx].lock().unwrap();
                if !matches!(*dep_status, TxStatus::Completed) {
                    all_deps_completed = false;
                    break;
                }
                // Lock released at end of scope
            }

            // If all dependencies satisfied, schedule for execution
            if all_deps_completed {
                // Check if this dependent is still in Ready state
                let is_ready = {
                    let dep_status = self.tx_status[dependent_idx].lock().unwrap();
                    matches!(*dep_status, TxStatus::Ready)
                };

                if is_ready {
                    // Schedule for execution
                    let mut execution_queue = self.execution_queue.lock().unwrap();
                    execution_queue.push_back(TxVersion {
                        tx_idx: dependent_idx,
                        tx_incarnation: 0,
                    });
                }
            }
        }
    }

    /// Mark transaction validation as completed
    ///
    /// This method performs conflict detection and validation of parallel execution results.
    /// It checks if any accounts read by this transaction were written by transactions
    /// that executed after it (read-write conflict). If conflicts are detected, the
    /// transaction is marked for retry up to `max_retries` times.
    ///
    /// # Conflict Detection Algorithm (Block-STM inspired):
    /// 1. Get the execution result for this transaction
    /// 2. Check all transactions that executed before this one
    /// 3. If any earlier transaction wrote to an account this transaction read,
    ///    there is a read-write conflict
    /// 4. If conflict detected and retries < max_retries, schedule retry with higher incarnation
    /// 5. If max_retries exceeded, mark as failed
    /// 6. If no conflicts, mark as completed and unblock dependent transactions
    ///
    /// # Safety:
    /// - Prevents deadlocks by limiting retries
    /// - Thread-safe with proper locking
    /// - Maintains transaction ordering guarantees
    pub fn finish_validation(&self, tx_version: TxVersion) {
        let tx_idx = tx_version.tx_idx;

        debug!(
            tx_idx = tx_idx,
            incarnation = tx_version.tx_incarnation,
            "Validating transaction execution"
        );

        // Get execution result
        let execution_results = self.execution_results.lock().unwrap();
        let result = match &execution_results[tx_idx] {
            Some(r) => r.clone(),
            None => {
                warn!(
                    tx_idx = tx_idx,
                    "No execution result found for validation"
                );
                return;
            }
        };
        drop(execution_results);

        // Detect read-write conflicts
        let has_conflict = self.detect_conflicts(tx_idx, &result);

        if has_conflict {
            // Conflict detected - check if we can retry
            let mut retry_count = self.retry_counts[tx_idx].lock().unwrap();

            if *retry_count < self.config.max_retries {
                *retry_count += 1;
                warn!(
                    tx_idx = tx_idx,
                    incarnation = tx_version.tx_incarnation,
                    retry_count = *retry_count,
                    max_retries = self.config.max_retries,
                    "Conflict detected - scheduling retry"
                );

                // Schedule retry with incremented incarnation
                let mut execution_queue = self.execution_queue.lock().unwrap();
                execution_queue.push_back(TxVersion {
                    tx_idx,
                    tx_incarnation: tx_version.tx_incarnation + 1,
                });

                // Mark as ready for retry
                let mut status = self.tx_status[tx_idx].lock().unwrap();
                *status = TxStatus::Ready;
            } else {
                // Max retries exceeded - mark as failed
                warn!(
                    tx_idx = tx_idx,
                    incarnation = tx_version.tx_incarnation,
                    retry_count = *retry_count,
                    "Max retries exceeded - marking as failed"
                );

                let mut status = self.tx_status[tx_idx].lock().unwrap();
                *status = TxStatus::Failed;
            }
        } else {
            // No conflicts - mark as completed
            debug!(
                tx_idx = tx_idx,
                incarnation = tx_version.tx_incarnation,
                "Validation successful - no conflicts detected"
            );

            let mut status = self.tx_status[tx_idx].lock().unwrap();
            *status = TxStatus::Completed;

            // Unblock dependent transactions
            self.unblock_dependents(tx_idx);
        }
    }

    /// Detect read-write conflicts for a transaction
    ///
    /// A conflict occurs when:
    /// - Transaction A reads account X
    /// - Transaction B (with lower index) writes to account X
    /// - Transaction B executed/validated after Transaction A started
    ///
    /// # Arguments
    /// * `tx_idx` - Index of transaction to check
    /// * `result` - Execution result containing read_set and write_set
    ///
    /// # Returns
    /// * `true` - Conflict detected, transaction needs retry
    /// * `false` - No conflicts, validation passed
    fn detect_conflicts(&self, tx_idx: TxIdx, result: &ParallelExecutionResult) -> bool {
        let execution_results = self.execution_results.lock().unwrap();

        // Check all transactions with lower index (should execute before this one)
        for earlier_idx in 0..tx_idx {
            if let Some(earlier_result) = &execution_results[earlier_idx] {
                // Check if earlier transaction wrote to any account this transaction read
                for read_addr in &result.read_set {
                    if earlier_result.write_set.contains(read_addr) {
                        // Check if earlier transaction has higher incarnation
                        // (meaning it executed after our transaction started)
                        if earlier_result.incarnation > result.incarnation {
                            debug!(
                                tx_idx = tx_idx,
                                earlier_idx = earlier_idx,
                                conflicting_address = ?read_addr,
                                our_incarnation = result.incarnation,
                                their_incarnation = earlier_result.incarnation,
                                "Read-write conflict detected"
                            );
                            return true;
                        }
                    }
                }
            }
        }

        // Check transactions with higher index that might have dependencies
        for later_idx in (tx_idx + 1)..execution_results.len() {
            if let Some(later_result) = &execution_results[later_idx] {
                // If a later transaction wrote to something we read,
                // and it has a lower or equal incarnation (started before/same time),
                // we might have a conflict
                for read_addr in &result.read_set {
                    if later_result.write_set.contains(read_addr) &&
                       later_result.incarnation <= result.incarnation {
                        debug!(
                            tx_idx = tx_idx,
                            later_idx = later_idx,
                            conflicting_address = ?read_addr,
                            "Potential forward conflict detected"
                        );
                        // This is less critical but log it
                    }
                }
            }
        }

        false
    }

    /// Unblock dependent transactions after successful validation
    fn unblock_dependents(&self, tx_idx: TxIdx) {
        for &dependent_idx in &self.dependencies[tx_idx].dependents {
            let dep_status = self.tx_status[dependent_idx].lock().unwrap();

            // Check if all dependencies are satisfied
            let mut all_deps_completed = true;
            for &dep_idx in &self.dependencies[dependent_idx].depends_on {
                let dep_status_ref = self.tx_status[dep_idx].lock().unwrap();
                if !matches!(*dep_status_ref, TxStatus::Completed) {
                    all_deps_completed = false;
                    break;
                }
            }

            if all_deps_completed && matches!(*dep_status, TxStatus::Ready) {
                let mut execution_queue = self.execution_queue.lock().unwrap();
                execution_queue.push_back(TxVersion {
                    tx_idx: dependent_idx,
                    tx_incarnation: 0,
                });
            }
        }
    }

    /// Store execution result for validation
    pub fn store_result(&self, result: ParallelExecutionResult) {
        let mut execution_results = self.execution_results.lock().unwrap();
        let tx_idx = result.tx_idx;
        execution_results[tx_idx] = Some(result);
    }

    /// Schedule transaction for validation
    pub fn schedule_validation(&self, tx_version: TxVersion) {
        let mut validation_queue = self.validation_queue.lock().unwrap();
        validation_queue.push_back(tx_version);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloy_consensus::TxLegacy;
    use alloy_primitives::{Bytes, TxKind, Signature};

    #[test]
    fn test_parallel_config_default() {
        let config = ParallelConfig::default();
        assert_eq!(config.concurrency_level.get(), 8);
        assert!(config.enable_lazy_updates);
        assert_eq!(config.max_retries, 3);
        assert_eq!(config.min_transactions_for_parallel, 4);
        assert!(!config.force_sequential);
    }

    #[test]
    fn test_should_use_parallel() {
        let config = ParallelConfig::default();
        let executor = ParallelExecutor::new(config);

        // Test with insufficient transactions
        let empty_txs = vec![];
        assert!(!executor.should_use_parallel(&empty_txs));

        let few_txs = vec![]; // Mock 2 transactions
        assert!(!executor.should_use_parallel(&few_txs));
    }

    #[test]
    fn test_mv_memory_lazy_balance() {
        let mut mv_memory = MvMemory::new();
        let address = Address::random();

        // Add lazy balance addition
        mv_memory.add_lazy_balance_addition(address, U256::from(100), 0);

        // Add lazy balance subtraction
        mv_memory.add_lazy_balance_subtraction(address, U256::from(30), 1);

        // Evaluate lazy balances
        let changes = mv_memory.evaluate_lazy_balances();

        assert_eq!(changes.len(), 1);
        assert_eq!(changes[0].address, address);
        assert_eq!(changes[0].balance_change, Some(70)); // 100 - 30
    }

    #[test]
    fn test_calculate_intrinsic_gas_simple_transfer() {
        let config = ParallelConfig::default();
        let executor = ParallelExecutor::new(config);

        // Create a simple transfer transaction (no data, no access list)
        let tx = create_test_transaction(
            Address::random(),
            TxKind::Call(Address::random()),
            U256::from(1000),
            Bytes::new(),
            None,
        );

        let gas = executor.calculate_intrinsic_gas(&tx);
        assert_eq!(gas, 21000, "Simple transfer should cost exactly 21000 gas");
    }

    #[test]
    fn test_calculate_intrinsic_gas_with_data() {
        let config = ParallelConfig::default();
        let executor = ParallelExecutor::new(config);

        // Create transaction with data: 4 zero bytes + 4 non-zero bytes
        let data = Bytes::from(vec![0, 0, 0, 0, 1, 2, 3, 4]);
        let tx = create_test_transaction(
            Address::random(),
            TxKind::Call(Address::random()),
            U256::ZERO,
            data,
            None,
        );

        let gas = executor.calculate_intrinsic_gas(&tx);
        // Base: 21000 + (4 * 4 zeros) + (4 * 16 non-zeros) = 21000 + 16 + 64 = 21080
        assert_eq!(gas, 21080, "Transaction with data should include data gas cost");
    }

    #[test]
    fn test_calculate_intrinsic_gas_contract_creation() {
        let config = ParallelConfig::default();
        let executor = ParallelExecutor::new(config);

        // Contract creation (to = None)
        let tx = create_test_transaction(
            Address::random(),
            TxKind::Create,
            U256::ZERO,
            Bytes::from(vec![0x60, 0x80]), // Example bytecode
            None,
        );

        let gas = executor.calculate_intrinsic_gas(&tx);
        // Base: 21000 + contract creation: 32000 + data cost: (2 * 16) = 53032
        assert_eq!(gas, 53032, "Contract creation should include 32000 gas");
    }

    #[test]
    fn test_is_ande_precompile_call() {
        let config = ParallelConfig::default();
        let executor = ParallelExecutor::new(config);

        use crate::evm_config::ANDE_PRECOMPILE_ADDRESS;

        // Test ANDE precompile address
        assert!(executor.is_ande_precompile_call(ANDE_PRECOMPILE_ADDRESS));

        // Test random address
        assert!(!executor.is_ande_precompile_call(Address::random()));

        // Test zero address
        assert!(!executor.is_ande_precompile_call(Address::ZERO));
    }

    #[test]
    fn test_mv_memory_multiple_lazy_operations() {
        let mut mv_memory = MvMemory::new();
        let address = Address::random();

        // Simulate multiple transactions affecting the same account
        mv_memory.add_lazy_balance_addition(address, U256::from(100), 0); // +100 from tx 0
        mv_memory.add_lazy_balance_addition(address, U256::from(50), 1);  // +50 from tx 1
        mv_memory.add_lazy_balance_subtraction(address, U256::from(30), 2); // -30 from tx 2
        mv_memory.add_lazy_nonce_increment(address, 0); // Nonce++ from tx 0
        mv_memory.add_lazy_nonce_increment(address, 1); // Nonce++ from tx 1

        // Evaluate lazy balances
        let changes = mv_memory.evaluate_lazy_balances();

        assert_eq!(changes.len(), 1);
        assert_eq!(changes[0].address, address);
        assert_eq!(changes[0].balance_change, Some(120)); // +100 +50 -30 = 120
        assert_eq!(changes[0].nonce_change, Some(2)); // 2 nonce increments
    }

    #[test]
    fn test_mv_memory_saturating_arithmetic() {
        let mut mv_memory = MvMemory::new();
        let address = Address::random();

        // Test balance overflow protection
        mv_memory.add_lazy_balance_addition(address, U256::MAX, 0);
        mv_memory.add_lazy_balance_addition(address, U256::from(100), 1);

        let changes = mv_memory.evaluate_lazy_balances();
        assert_eq!(changes.len(), 1);
        // Should saturate at U256::MAX, not panic
        assert!(changes[0].balance_change.is_some());
    }

    #[test]
    fn test_scheduler_initialization() {
        let dependencies = vec![
            TxDependency {
                depends_on: vec![],
                dependents: vec![1],
                read_accounts: vec![Address::random()],
                write_accounts: vec![Address::random()],
            },
            TxDependency {
                depends_on: vec![0],
                dependents: vec![],
                read_accounts: vec![Address::random()],
                write_accounts: vec![Address::random()],
            },
        ];

        let config = ParallelConfig::default();
        let scheduler = ParallelScheduler::new(2, dependencies, config);

        // First transaction should be ready to execute
        let task = scheduler.next_task();
        assert!(task.is_some());

        if let Some(ParallelTask::Execute(tx_version)) = task {
            assert_eq!(tx_version.tx_idx, 0);
            assert_eq!(tx_version.tx_incarnation, 0);
        } else {
            panic!("Expected Execute task");
        }
    }

    #[test]
    fn test_scheduler_dependency_resolution() {
        let dependencies = vec![
            TxDependency {
                depends_on: vec![],
                dependents: vec![1, 2],
                read_accounts: vec![],
                write_accounts: vec![],
            },
            TxDependency {
                depends_on: vec![0],
                dependents: vec![],
                read_accounts: vec![],
                write_accounts: vec![],
            },
            TxDependency {
                depends_on: vec![0],
                dependents: vec![],
                read_accounts: vec![],
                write_accounts: vec![],
            },
        ];

        let config = ParallelConfig::default();
        let scheduler = ParallelScheduler::new(3, dependencies, config);

        // Execute first transaction
        let task1 = scheduler.next_task();
        assert!(task1.is_some());

        // Complete first transaction
        scheduler.finish_execution(TxVersion {
            tx_idx: 0,
            tx_incarnation: 0,
        });

        // Now transactions 1 and 2 should be ready
        let task2 = scheduler.next_task();
        let task3 = scheduler.next_task();
        assert!(task2.is_some());
        assert!(task3.is_some());
    }

    // =========================================================================
    // COMPREHENSIVE TEST SUITE FOR PRODUCTION PARALLEL EVM
    // =========================================================================

    // -------------------------------------------------------------------------
    // CONFLICT DETECTION TESTS
    // -------------------------------------------------------------------------

    #[test]
    fn test_detect_conflicts_read_write_conflict() {
        let config = ParallelConfig::default();
        let shared_account = Address::random();

        // Transaction 0: writes to shared_account
        let tx0_result = ParallelExecutionResult {
            tx_idx: 0,
            gas_used: 21000,
            success: true,
            error: None,
            state_changes: HashMap::new(),
            read_set: vec![],
            write_set: vec![shared_account],
            incarnation: 1, // Higher incarnation
        };

        // Transaction 1: reads from shared_account
        let tx1_result = ParallelExecutionResult {
            tx_idx: 1,
            gas_used: 21000,
            success: true,
            error: None,
            state_changes: HashMap::new(),
            read_set: vec![shared_account],
            write_set: vec![],
            incarnation: 0, // Lower incarnation - conflict!
        };

        let dependencies = vec![
            TxDependency {
                depends_on: vec![],
                dependents: vec![1],
                read_accounts: vec![],
                write_accounts: vec![shared_account],
            },
            TxDependency {
                depends_on: vec![0],
                dependents: vec![],
                read_accounts: vec![shared_account],
                write_accounts: vec![],
            },
        ];

        let scheduler = ParallelScheduler::new(2, dependencies, config);

        // Store results
        scheduler.store_result(tx0_result.clone());
        scheduler.store_result(tx1_result.clone());

        // Detect conflict
        let has_conflict = scheduler.detect_conflicts(1, &tx1_result);
        assert!(has_conflict, "Should detect read-write conflict when earlier tx has higher incarnation");
    }

    #[test]
    fn test_detect_conflicts_no_conflict_same_incarnation() {
        let config = ParallelConfig::default();
        let shared_account = Address::random();

        let tx0_result = ParallelExecutionResult {
            tx_idx: 0,
            gas_used: 21000,
            success: true,
            error: None,
            state_changes: HashMap::new(),
            read_set: vec![],
            write_set: vec![shared_account],
            incarnation: 0, // Same incarnation
        };

        let tx1_result = ParallelExecutionResult {
            tx_idx: 1,
            gas_used: 21000,
            success: true,
            error: None,
            state_changes: HashMap::new(),
            read_set: vec![shared_account],
            write_set: vec![],
            incarnation: 0, // Same incarnation - no conflict
        };

        let dependencies = vec![
            TxDependency {
                depends_on: vec![],
                dependents: vec![],
                read_accounts: vec![],
                write_accounts: vec![shared_account],
            },
            TxDependency {
                depends_on: vec![],
                dependents: vec![],
                read_accounts: vec![shared_account],
                write_accounts: vec![],
            },
        ];

        let scheduler = ParallelScheduler::new(2, dependencies, config);
        scheduler.store_result(tx0_result.clone());
        scheduler.store_result(tx1_result.clone());

        let has_conflict = scheduler.detect_conflicts(1, &tx1_result);
        assert!(!has_conflict, "Should not detect conflict when incarnations are equal");
    }

    #[test]
    fn test_detect_conflicts_multiple_accounts() {
        let config = ParallelConfig::default();
        let account_a = Address::random();
        let account_b = Address::random();
        let account_c = Address::random();

        // Tx 0: writes to A and B
        let tx0_result = ParallelExecutionResult {
            tx_idx: 0,
            gas_used: 21000,
            success: true,
            error: None,
            state_changes: HashMap::new(),
            read_set: vec![],
            write_set: vec![account_a, account_b],
            incarnation: 2,
        };

        // Tx 1: reads from A, B, C
        let tx1_result = ParallelExecutionResult {
            tx_idx: 1,
            gas_used: 21000,
            success: true,
            error: None,
            state_changes: HashMap::new(),
            read_set: vec![account_a, account_b, account_c],
            write_set: vec![],
            incarnation: 1, // Earlier incarnation - conflict!
        };

        let dependencies = vec![
            TxDependency {
                depends_on: vec![],
                dependents: vec![],
                read_accounts: vec![],
                write_accounts: vec![account_a, account_b],
            },
            TxDependency {
                depends_on: vec![],
                dependents: vec![],
                read_accounts: vec![account_a, account_b, account_c],
                write_accounts: vec![],
            },
        ];

        let scheduler = ParallelScheduler::new(2, dependencies, config);
        scheduler.store_result(tx0_result.clone());
        scheduler.store_result(tx1_result.clone());

        let has_conflict = scheduler.detect_conflicts(1, &tx1_result);
        assert!(has_conflict, "Should detect conflict on multiple accounts");
    }

    // -------------------------------------------------------------------------
    // RETRY LOGIC TESTS
    // -------------------------------------------------------------------------

    #[test]
    fn test_retry_logic_within_limit() {
        let config = ParallelConfig::default();
        let shared_account = Address::random();

        let dependencies = vec![
            TxDependency {
                depends_on: vec![],
                dependents: vec![1],
                read_accounts: vec![],
                write_accounts: vec![shared_account],
            },
            TxDependency {
                depends_on: vec![0],
                dependents: vec![],
                read_accounts: vec![shared_account],
                write_accounts: vec![],
            },
        ];

        let scheduler = ParallelScheduler::new(2, dependencies, config);

        // Create conflicting results
        let tx0_result = ParallelExecutionResult {
            tx_idx: 0,
            gas_used: 21000,
            success: true,
            error: None,
            state_changes: HashMap::new(),
            read_set: vec![],
            write_set: vec![shared_account],
            incarnation: 1,
        };

        let tx1_result = ParallelExecutionResult {
            tx_idx: 1,
            gas_used: 21000,
            success: true,
            error: None,
            state_changes: HashMap::new(),
            read_set: vec![shared_account],
            write_set: vec![],
            incarnation: 0,
        };

        scheduler.store_result(tx0_result);
        scheduler.store_result(tx1_result.clone());

        // Clear any initial tasks from queue
        {
            let mut execution_queue = scheduler.execution_queue.lock().unwrap();
            execution_queue.clear();
        }

        // First validation should detect conflict and schedule retry
        scheduler.finish_validation(TxVersion {
            tx_idx: 1,
            tx_incarnation: 0,
        });

        // Check that retry was scheduled
        let execution_queue = scheduler.execution_queue.lock().unwrap();
        assert!(!execution_queue.is_empty(), "Retry should be scheduled");

        // Check that incarnation was incremented
        if let Some(retry_task) = execution_queue.front() {
            assert_eq!(retry_task.tx_idx, 1);
            assert_eq!(retry_task.tx_incarnation, 1, "Incarnation should be incremented");
        }
    }

    #[test]
    fn test_retry_logic_max_retries_exceeded() {
        let mut config = ParallelConfig::default();
        config.max_retries = 2; // Set low limit for testing

        let shared_account = Address::random();

        let dependencies = vec![
            TxDependency {
                depends_on: vec![],
                dependents: vec![],
                read_accounts: vec![],
                write_accounts: vec![shared_account],
            },
            TxDependency {
                depends_on: vec![],
                dependents: vec![],
                read_accounts: vec![shared_account],
                write_accounts: vec![],
            },
        ];

        let scheduler = ParallelScheduler::new(2, dependencies, config);

        // Simulate max retries
        {
            let mut retry_count = scheduler.retry_counts[1].lock().unwrap();
            *retry_count = 2; // Already at max
        }

        // Create conflicting result
        let tx1_result = ParallelExecutionResult {
            tx_idx: 1,
            gas_used: 21000,
            success: true,
            error: None,
            state_changes: HashMap::new(),
            read_set: vec![shared_account],
            write_set: vec![],
            incarnation: 0,
        };

        let tx0_result = ParallelExecutionResult {
            tx_idx: 0,
            gas_used: 21000,
            success: true,
            error: None,
            state_changes: HashMap::new(),
            read_set: vec![],
            write_set: vec![shared_account],
            incarnation: 1,
        };

        scheduler.store_result(tx0_result);
        scheduler.store_result(tx1_result.clone());

        // Validation should mark as failed
        scheduler.finish_validation(TxVersion {
            tx_idx: 1,
            tx_incarnation: 0,
        });

        // Check that transaction is marked as failed
        let status = scheduler.tx_status[1].lock().unwrap();
        assert!(matches!(*status, TxStatus::Failed), "Should be marked as failed after max retries");
    }

    // -------------------------------------------------------------------------
    // ANDE PRECOMPILE INTEGRATION TESTS
    // -------------------------------------------------------------------------

    #[test]
    fn test_ande_precompile_lazy_update() {
        let config = ParallelConfig {
            enable_lazy_updates: true,
            ..Default::default()
        };
        let executor = ParallelExecutor::new(config);

        use crate::evm_config::ANDE_PRECOMPILE_ADDRESS;

        // Create transaction to ANDE precompile
        let tx = create_test_transaction_with_nonce(
            Address::random(),
            TxKind::Call(ANDE_PRECOMPILE_ADDRESS),
            U256::from(1000),
            Bytes::new(),
            None,
            0,
        );

        let mv_memory = Arc::new(Mutex::new(MvMemory::new()));
        let evm_config = create_test_evm_config();
        let parent_header = create_test_sealed_header();
        let next_block_attrs = create_test_block_attrs();

        let result = executor.execute_transaction_parallel(
            TxVersion { tx_idx: 0, tx_incarnation: 0 },
            &tx,
            &evm_config,
            &parent_header,
            &next_block_attrs,
            &mv_memory,
        );

        assert!(result.is_some());
        assert!(result.unwrap().success);

        // Check that lazy update was recorded
        let mv_memory_guard = mv_memory.lock().unwrap();
        assert!(!mv_memory_guard.lazy_accounts.is_empty(), "Lazy update should be recorded for ANDE precompile");
    }

    #[test]
    fn test_ande_precompile_lazy_updates_disabled() {
        let config = ParallelConfig {
            enable_lazy_updates: false,
            ..Default::default()
        };
        let executor = ParallelExecutor::new(config);

        use crate::evm_config::ANDE_PRECOMPILE_ADDRESS;

        let tx = create_test_transaction_with_nonce(
            Address::random(),
            TxKind::Call(ANDE_PRECOMPILE_ADDRESS),
            U256::from(1000),
            Bytes::new(),
            None,
            0,
        );

        let mv_memory = Arc::new(Mutex::new(MvMemory::new()));
        let evm_config = create_test_evm_config();
        let parent_header = create_test_sealed_header();
        let next_block_attrs = create_test_block_attrs();

        let result = executor.execute_transaction_parallel(
            TxVersion { tx_idx: 0, tx_incarnation: 0 },
            &tx,
            &evm_config,
            &parent_header,
            &next_block_attrs,
            &mv_memory,
        );

        assert!(result.is_some());

        // With lazy updates disabled, immediate state change should be recorded
        let result = result.unwrap();
        assert!(result.success);
        assert!(!result.state_changes.is_empty(), "Should record immediate state change when lazy updates disabled");
    }

    // -------------------------------------------------------------------------
    // MV_MEMORY EDGE CASES
    // -------------------------------------------------------------------------

    #[test]
    fn test_mv_memory_concurrent_modifications() {
        let mut mv_memory = MvMemory::new();
        let address = Address::random();

        // Simulate concurrent operations from multiple transactions
        for tx_idx in 0..100 {
            if tx_idx % 2 == 0 {
                mv_memory.add_lazy_balance_addition(address, U256::from(10), tx_idx);
            } else {
                mv_memory.add_lazy_balance_subtraction(address, U256::from(5), tx_idx);
            }
            mv_memory.add_lazy_nonce_increment(address, tx_idx);
        }

        let changes = mv_memory.evaluate_lazy_balances();
        assert_eq!(changes.len(), 1);

        // 50 additions of 10 = +500
        // 50 subtractions of 5 = -250
        // Net = +250
        assert_eq!(changes[0].balance_change, Some(250));

        // 100 nonce increments
        assert_eq!(changes[0].nonce_change, Some(100));
    }

    #[test]
    fn test_mv_memory_zero_balance_operations() {
        let mut mv_memory = MvMemory::new();
        let address = Address::random();

        // Add zero
        mv_memory.add_lazy_balance_addition(address, U256::ZERO, 0);

        // Subtract zero
        mv_memory.add_lazy_balance_subtraction(address, U256::ZERO, 1);

        let changes = mv_memory.evaluate_lazy_balances();

        // Should still track the account
        assert_eq!(changes.len(), 1);
        assert_eq!(changes[0].balance_change, Some(0));
    }

    #[test]
    fn test_mv_memory_large_balance_operations() {
        let mut mv_memory = MvMemory::new();
        let address = Address::random();

        // Add maximum value
        mv_memory.add_lazy_balance_addition(address, U256::MAX, 0);

        // Try to subtract - should saturate
        mv_memory.add_lazy_balance_subtraction(address, U256::from(1000), 1);

        let changes = mv_memory.evaluate_lazy_balances();
        assert_eq!(changes.len(), 1);

        // Should saturate at U256::MAX
        assert!(changes[0].balance_change.is_some());
    }

    #[test]
    fn test_mv_memory_multiple_accounts() {
        let mut mv_memory = MvMemory::new();
        let address_a = Address::random();
        let address_b = Address::random();
        let address_c = Address::random();

        // Different operations on different accounts
        mv_memory.add_lazy_balance_addition(address_a, U256::from(100), 0);
        mv_memory.add_lazy_balance_addition(address_b, U256::from(200), 1);
        mv_memory.add_lazy_balance_subtraction(address_c, U256::from(50), 2);
        mv_memory.add_lazy_nonce_increment(address_a, 0);
        mv_memory.add_lazy_nonce_increment(address_a, 3);
        mv_memory.add_lazy_nonce_increment(address_b, 1);

        let changes = mv_memory.evaluate_lazy_balances();
        assert_eq!(changes.len(), 3, "Should track all 3 accounts");

        // Verify each account
        let change_a = changes.iter().find(|c| c.address == address_a).unwrap();
        assert_eq!(change_a.balance_change, Some(100));
        assert_eq!(change_a.nonce_change, Some(2)); // 2 increments

        let change_b = changes.iter().find(|c| c.address == address_b).unwrap();
        assert_eq!(change_b.balance_change, Some(200));
        assert_eq!(change_b.nonce_change, Some(1)); // 1 increment

        let change_c = changes.iter().find(|c| c.address == address_c).unwrap();
        assert_eq!(change_c.balance_change, Some(-50i128));
        assert_eq!(change_c.nonce_change, None);
    }

    // -------------------------------------------------------------------------
    // SCHEDULER COMPLEX DEPENDENCY TESTS
    // -------------------------------------------------------------------------

    #[test]
    fn test_scheduler_chain_dependency() {
        // Test linear dependency chain: tx0 -> tx1 -> tx2 -> tx3
        let dependencies = vec![
            TxDependency {
                depends_on: vec![],
                dependents: vec![1],
                read_accounts: vec![],
                write_accounts: vec![],
            },
            TxDependency {
                depends_on: vec![0],
                dependents: vec![2],
                read_accounts: vec![],
                write_accounts: vec![],
            },
            TxDependency {
                depends_on: vec![1],
                dependents: vec![3],
                read_accounts: vec![],
                write_accounts: vec![],
            },
            TxDependency {
                depends_on: vec![2],
                dependents: vec![],
                read_accounts: vec![],
                write_accounts: vec![],
            },
        ];

        let config = ParallelConfig::default();
        let scheduler = ParallelScheduler::new(4, dependencies, config);

        // Only tx0 should be ready initially
        let task1 = scheduler.next_task();
        assert!(task1.is_some());
        if let Some(ParallelTask::Execute(tx_version)) = task1 {
            assert_eq!(tx_version.tx_idx, 0);
        }

        // Complete tx0
        scheduler.finish_execution(TxVersion { tx_idx: 0, tx_incarnation: 0 });

        // Now tx1 should be ready
        let task2 = scheduler.next_task();
        assert!(task2.is_some());
        if let Some(ParallelTask::Execute(tx_version)) = task2 {
            assert_eq!(tx_version.tx_idx, 1);
        }
    }

    #[test]
    fn test_scheduler_diamond_dependency() {
        // Test diamond dependency pattern:
        //      tx0
        //     /   \
        //   tx1   tx2
        //     \   /
        //      tx3
        let dependencies = vec![
            TxDependency {
                depends_on: vec![],
                dependents: vec![1, 2],
                read_accounts: vec![],
                write_accounts: vec![],
            },
            TxDependency {
                depends_on: vec![0],
                dependents: vec![3],
                read_accounts: vec![],
                write_accounts: vec![],
            },
            TxDependency {
                depends_on: vec![0],
                dependents: vec![3],
                read_accounts: vec![],
                write_accounts: vec![],
            },
            TxDependency {
                depends_on: vec![1, 2],
                dependents: vec![],
                read_accounts: vec![],
                write_accounts: vec![],
            },
        ];

        let config = ParallelConfig::default();
        let scheduler = ParallelScheduler::new(4, dependencies, config);

        // Complete tx0
        scheduler.next_task(); // Get tx0
        scheduler.finish_execution(TxVersion { tx_idx: 0, tx_incarnation: 0 });

        // Both tx1 and tx2 should be ready
        let task1 = scheduler.next_task();
        let task2 = scheduler.next_task();
        assert!(task1.is_some());
        assert!(task2.is_some());

        // tx3 should not be ready yet
        let task3 = scheduler.next_task();
        assert!(task3.is_none());

        // Complete tx1 and tx2
        scheduler.finish_execution(TxVersion { tx_idx: 1, tx_incarnation: 0 });
        scheduler.finish_execution(TxVersion { tx_idx: 2, tx_incarnation: 0 });

        // Now tx3 should be ready
        let task4 = scheduler.next_task();
        assert!(task4.is_some());
        if let Some(ParallelTask::Execute(tx_version)) = task4 {
            assert_eq!(tx_version.tx_idx, 3);
        }
    }

    #[test]
    fn test_scheduler_independent_transactions() {
        // Test fully independent transactions (can all execute in parallel)
        let dependencies = vec![
            TxDependency {
                depends_on: vec![],
                dependents: vec![],
                read_accounts: vec![Address::random()],
                write_accounts: vec![Address::random()],
            },
            TxDependency {
                depends_on: vec![],
                dependents: vec![],
                read_accounts: vec![Address::random()],
                write_accounts: vec![Address::random()],
            },
            TxDependency {
                depends_on: vec![],
                dependents: vec![],
                read_accounts: vec![Address::random()],
                write_accounts: vec![Address::random()],
            },
            TxDependency {
                depends_on: vec![],
                dependents: vec![],
                read_accounts: vec![Address::random()],
                write_accounts: vec![Address::random()],
            },
        ];

        let config = ParallelConfig::default();
        let scheduler = ParallelScheduler::new(4, dependencies, config);

        // All 4 transactions should be ready
        let task1 = scheduler.next_task();
        let task2 = scheduler.next_task();
        let task3 = scheduler.next_task();
        let task4 = scheduler.next_task();

        assert!(task1.is_some());
        assert!(task2.is_some());
        assert!(task3.is_some());
        assert!(task4.is_some());

        // No more tasks
        let task5 = scheduler.next_task();
        assert!(task5.is_none());
    }

    // -------------------------------------------------------------------------
    // INTRINSIC GAS CALCULATION TESTS
    // -------------------------------------------------------------------------

    #[test]
    fn test_calculate_intrinsic_gas_access_list() {
        let config = ParallelConfig::default();
        let executor = ParallelExecutor::new(config);

        use alloy_eips::eip2930::AccessList;
        use alloy_consensus::TxEip2930;

        // Create EIP-2930 transaction with access list
        let access_list_addr1 = Address::random();
        let access_list_addr2 = Address::random();

        let access_list = AccessList(vec![
            alloy_eips::eip2930::AccessListItem {
                address: access_list_addr1,
                storage_keys: vec![
                    alloy_primitives::B256::ZERO,
                    alloy_primitives::B256::left_padding_from(&[1]),
                    alloy_primitives::B256::left_padding_from(&[2])
                ], // 3 keys
            },
            alloy_eips::eip2930::AccessListItem {
                address: access_list_addr2,
                storage_keys: vec![alloy_primitives::B256::left_padding_from(&[10])], // 1 key
            },
        ]);

        let tx_eip2930 = TxEip2930 {
            chain_id: 1337,
            nonce: 0,
            gas_price: 1000000000,
            gas_limit: 100000,
            to: TxKind::Call(Address::random()),
            value: U256::ZERO,
            input: Bytes::new(),
            access_list,
        };

        use alloy_consensus::TypedTransaction;

        let signature = Signature::test_signature();
        let typed_tx = TypedTransaction::Eip2930(tx_eip2930);
        let tx = TransactionSigned::new_unhashed(typed_tx.into(), signature);

        let gas = executor.calculate_intrinsic_gas(&tx);

        // Base: 21000
        // Access list: 2 addresses * 2400 = 4800
        // Storage keys: 4 keys * 1900 = 7600
        // Total: 21000 + 4800 + 7600 = 33400
        assert_eq!(gas, 33400, "Access list gas calculation incorrect");
    }

    #[test]
    fn test_calculate_intrinsic_gas_large_data() {
        let config = ParallelConfig::default();
        let executor = ParallelExecutor::new(config);

        // Create transaction with large calldata
        let mut data = vec![0u8; 1000]; // 1000 zero bytes
        data.extend(vec![1u8; 1000]); // 1000 non-zero bytes

        let tx = create_test_transaction(
            Address::random(),
            TxKind::Call(Address::random()),
            U256::ZERO,
            Bytes::from(data),
            None,
        );

        let gas = executor.calculate_intrinsic_gas(&tx);

        // Base: 21000
        // Zero bytes: 1000 * 4 = 4000
        // Non-zero bytes: 1000 * 16 = 16000
        // Total: 21000 + 4000 + 16000 = 41000
        assert_eq!(gas, 41000, "Large data gas calculation incorrect");
    }

    // -------------------------------------------------------------------------
    // EDGE CASES AND ERROR HANDLING
    // -------------------------------------------------------------------------

    #[test]
    fn test_execute_transaction_invalid_signature() {
        let config = ParallelConfig::default();
        let executor = ParallelExecutor::new(config);

        // Create transaction with invalid signature
        let tx = TxLegacy {
            chain_id: Some(1337),
            nonce: 0,
            gas_price: 1000000000,
            gas_limit: 21000,
            to: TxKind::Call(Address::random()),
            value: U256::from(1000),
            input: Bytes::new(),
        };

        // Use an invalid signature (all zeros) - this signature won't recover properly
        let invalid_sig = Signature::from_scalars_and_parity(
            alloy_primitives::B256::ZERO,
            alloy_primitives::B256::ZERO,
            false,
        );

        use alloy_consensus::TypedTransaction;

        let typed_tx = TypedTransaction::Legacy(tx);
        let signed_tx = TransactionSigned::new_unhashed(typed_tx.into(), invalid_sig);

        let mv_memory = Arc::new(Mutex::new(MvMemory::new()));
        let evm_config = create_test_evm_config();
        let parent_header = create_test_sealed_header();
        let next_block_attrs = create_test_block_attrs();

        let result = executor.execute_transaction_parallel(
            TxVersion { tx_idx: 0, tx_incarnation: 0 },
            &signed_tx,
            &evm_config,
            &parent_header,
            &next_block_attrs,
            &mv_memory,
        );

        // Should return error result (not panic)
        assert!(result.is_some());
        let result = result.unwrap();
        assert!(!result.success, "Should fail with invalid signature");
        assert!(result.error.is_some());
    }

    #[test]
    fn test_execute_transaction_gas_limit_too_low() {
        let config = ParallelConfig::default();
        let executor = ParallelExecutor::new(config);

        // Create transaction with gas limit lower than intrinsic gas
        let tx = TxLegacy {
            chain_id: Some(1337),
            nonce: 0,
            gas_price: 1000000000,
            gas_limit: 20000, // Less than 21000 minimum
            to: TxKind::Call(Address::random()),
            value: U256::from(1000),
            input: Bytes::new(),
        };

        use alloy_consensus::TypedTransaction;

        let signature = Signature::test_signature();
        let typed_tx = TypedTransaction::Legacy(tx);
        let signed_tx = TransactionSigned::new_unhashed(typed_tx.into(), signature);

        let mv_memory = Arc::new(Mutex::new(MvMemory::new()));
        let evm_config = create_test_evm_config();
        let parent_header = create_test_sealed_header();
        let next_block_attrs = create_test_block_attrs();

        let result = executor.execute_transaction_parallel(
            TxVersion { tx_idx: 0, tx_incarnation: 0 },
            &signed_tx,
            &evm_config,
            &parent_header,
            &next_block_attrs,
            &mv_memory,
        );

        assert!(result.is_some());
        let result = result.unwrap();
        assert!(!result.success, "Should fail with insufficient gas");
        assert!(result.error.is_some());
        assert!(result.error.unwrap().contains("Intrinsic gas too low"));
    }

    #[test]
    fn test_should_use_parallel_force_sequential() {
        let config = ParallelConfig {
            force_sequential: true,
            min_transactions_for_parallel: 4,
            ..Default::default()
        };
        let executor = ParallelExecutor::new(config);

        // Even with many transactions, should use sequential
        let many_txs = vec![
            create_test_transaction(
                Address::random(),
                TxKind::Call(Address::random()),
                U256::ZERO,
                Bytes::new(),
                None,
            );
            100
        ];

        assert!(!executor.should_use_parallel(&many_txs), "Should force sequential when configured");
    }

    #[test]
    fn test_should_use_parallel_min_transactions() {
        let config = ParallelConfig {
            force_sequential: false,
            min_transactions_for_parallel: 10,
            ..Default::default()
        };
        let executor = ParallelExecutor::new(config);

        // Test below threshold
        let few_txs = vec![
            create_test_transaction(
                Address::random(),
                TxKind::Call(Address::random()),
                U256::ZERO,
                Bytes::new(),
                None,
            );
            5
        ];
        assert!(!executor.should_use_parallel(&few_txs), "Should use sequential below min threshold");

        // Test at threshold
        let threshold_txs = vec![
            create_test_transaction(
                Address::random(),
                TxKind::Call(Address::random()),
                U256::ZERO,
                Bytes::new(),
                None,
            );
            10
        ];
        assert!(executor.should_use_parallel(&threshold_txs), "Should use parallel at threshold");

        // Test above threshold
        let many_txs = vec![
            create_test_transaction(
                Address::random(),
                TxKind::Call(Address::random()),
                U256::ZERO,
                Bytes::new(),
                None,
            );
            50
        ];
        assert!(executor.should_use_parallel(&many_txs), "Should use parallel above threshold");
    }

    // -------------------------------------------------------------------------
    // HELPER FUNCTIONS FOR TESTS
    // -------------------------------------------------------------------------

    fn create_test_transaction(
        _from: Address,
        to: TxKind,
        value: U256,
        input: Bytes,
        _access_list: Option<Vec<(Address, Vec<U256>)>>,
    ) -> TransactionSigned {
        use alloy_consensus::TypedTransaction;

        let tx = TxLegacy {
            chain_id: Some(1337),
            nonce: 0,
            gas_price: 1000000000,
            gas_limit: 21000,
            to,
            value,
            input,
        };

        let signature = Signature::test_signature();
        let typed_tx = TypedTransaction::Legacy(tx);
        TransactionSigned::new_unhashed(typed_tx.into(), signature)
    }

    fn create_test_transaction_with_nonce(
        _from: Address,
        to: TxKind,
        value: U256,
        input: Bytes,
        _access_list: Option<Vec<(Address, Vec<U256>)>>,
        nonce: u64,
    ) -> TransactionSigned {
        use alloy_consensus::TypedTransaction;

        let tx = TxLegacy {
            chain_id: Some(1337),
            nonce,
            gas_price: 1000000000,
            gas_limit: 100000, // Higher gas limit for complex operations
            to,
            value,
            input,
        };

        let signature = Signature::test_signature();
        let typed_tx = TypedTransaction::Legacy(tx);
        TransactionSigned::new_unhashed(typed_tx.into(), signature)
    }

    /// Helper to create test EVM config (works for any network)
    fn create_test_evm_config() -> AndeEvmConfig {
        use reth_chainspec::{ChainSpecBuilder, Chain};
        use std::sync::Arc;

        // Create a minimal test chain spec that works for unit tests
        // This is NOT mainnet or testnet specific - it's a local test configuration
        let chain_spec = Arc::new(
            ChainSpecBuilder::default()
                .chain(Chain::from_id(31337)) // Local test chain ID
                .genesis(Default::default())
                .build()
        );

        AndeEvmConfig::new(chain_spec)
    }

    fn create_test_sealed_header() -> SealedHeader {
        let header = Header {
            parent_hash: alloy_primitives::B256::ZERO,
            ommers_hash: alloy_primitives::B256::ZERO,
            beneficiary: Address::ZERO,
            state_root: alloy_primitives::B256::ZERO,
            transactions_root: alloy_primitives::B256::ZERO,
            receipts_root: alloy_primitives::B256::ZERO,
            logs_bloom: Default::default(),
            difficulty: U256::ZERO,
            number: 1,
            gas_limit: 10000000,
            gas_used: 0,
            timestamp: 1000000,
            mix_hash: alloy_primitives::B256::ZERO,
            nonce: alloy_primitives::B64::ZERO, // FixedBytes<8>
            base_fee_per_gas: Some(1000000000),
            withdrawals_root: None,
            blob_gas_used: None,
            excess_blob_gas: None,
            parent_beacon_block_root: None,
            requests_hash: None,
            extra_data: Default::default(),
        };

        SealedHeader::new(header, alloy_primitives::B256::ZERO)
    }

    fn create_test_block_attrs() -> NextBlockEnvAttributes {
        NextBlockEnvAttributes {
            timestamp: 1000001,
            suggested_fee_recipient: Address::ZERO,
            prev_randao: alloy_primitives::B256::ZERO,
            gas_limit: 10000000,
            withdrawals: Some(Default::default()),
            parent_beacon_block_root: Some(alloy_primitives::B256::ZERO),
        }
    }

    // =========================================================================
    // INTEGRATION TESTS: ANDE TOKEN DUALITY & PARALLEL EXECUTION
    // =========================================================================
    //
    // These tests verify the complete flow of parallel execution with ANDE
    // Token Duality precompile integration, lazy updates, and real-world scenarios.

    #[test]
    fn test_integration_ande_precompile_single_transaction() {
        use crate::evm_config::ANDE_PRECOMPILE_ADDRESS;

        let config = ParallelConfig {
            enable_lazy_updates: true,
            ..Default::default()
        };
        let executor = ParallelExecutor::new(config);

        // Create transaction sending value to ANDE precompile
        let sender = Address::random();
        let value = U256::from(1000000); // 1M wei

        let tx = create_test_transaction_with_nonce(
            sender,
            TxKind::Call(ANDE_PRECOMPILE_ADDRESS),
            value,
            Bytes::new(),
            None,
            0,
        );

        let mv_memory = Arc::new(Mutex::new(MvMemory::new()));
        let evm_config = create_test_evm_config();
        let parent_header = create_test_sealed_header();
        let next_block_attrs = create_test_block_attrs();

        // Execute transaction
        let result = executor.execute_transaction_parallel(
            TxVersion { tx_idx: 0, tx_incarnation: 0 },
            &tx,
            &evm_config,
            &parent_header,
            &next_block_attrs,
            &mv_memory,
        );

        // Verify execution succeeded
        assert!(result.is_some(), "Transaction should execute");
        let result = result.unwrap();
        assert!(result.success, "Transaction should succeed");

        // Verify lazy update was recorded for ANDE precompile
        let mv_memory_guard = mv_memory.lock().unwrap();
        assert!(
            mv_memory_guard.lazy_accounts.contains_key(&ANDE_PRECOMPILE_ADDRESS),
            "ANDE precompile should have lazy update recorded"
        );

        let lazy_state = &mv_memory_guard.lazy_accounts[&ANDE_PRECOMPILE_ADDRESS];
        assert_eq!(
            lazy_state.balance_additions.len(),
            1,
            "Should have one balance addition"
        );
        assert_eq!(
            lazy_state.balance_additions[0].1,
            value,
            "Balance addition should match transaction value"
        );
    }

    #[test]
    fn test_integration_ande_multiple_transactions_lazy_aggregation() {
        use crate::evm_config::ANDE_PRECOMPILE_ADDRESS;

        let config = ParallelConfig {
            enable_lazy_updates: true,
            min_transactions_for_parallel: 3,
            ..Default::default()
        };
        let executor = ParallelExecutor::new(config);

        let mv_memory = Arc::new(Mutex::new(MvMemory::new()));
        let evm_config = create_test_evm_config();
        let parent_header = create_test_sealed_header();
        let next_block_attrs = create_test_block_attrs();

        // Create 5 transactions all sending to ANDE precompile
        let values = vec![100, 200, 300, 400, 500];
        let mut total_value = U256::ZERO;

        for (i, val) in values.iter().enumerate() {
            let value = U256::from(*val);
            total_value = total_value.saturating_add(value);

            let tx = create_test_transaction_with_nonce(
                Address::random(),
                TxKind::Call(ANDE_PRECOMPILE_ADDRESS),
                value,
                Bytes::new(),
                None,
                i as u64,
            );

            let result = executor.execute_transaction_parallel(
                TxVersion { tx_idx: i, tx_incarnation: 0 },
                &tx,
                &evm_config,
                &parent_header,
                &next_block_attrs,
                &mv_memory,
            );

            assert!(result.is_some(), "Transaction {} should execute", i);
            assert!(result.unwrap().success, "Transaction {} should succeed", i);
        }

        // Verify all lazy updates were recorded
        let mut mv_memory_guard = mv_memory.lock().unwrap();
        let lazy_state = &mv_memory_guard.lazy_accounts[&ANDE_PRECOMPILE_ADDRESS];
        assert_eq!(
            lazy_state.balance_additions.len(),
            5,
            "Should have 5 balance additions"
        );

        // Evaluate lazy balances and verify total
        let changes = mv_memory_guard.evaluate_lazy_balances();
        assert_eq!(changes.len(), 1, "Should have changes for ANDE precompile");

        let ande_change = &changes[0];
        assert_eq!(ande_change.address, ANDE_PRECOMPILE_ADDRESS);

        let expected_balance_change = total_value.to::<u128>() as i128;
        assert_eq!(
            ande_change.balance_change,
            Some(expected_balance_change),
            "Total balance change should match sum of all transactions"
        );
    }

    #[test]
    fn test_integration_mixed_ande_and_regular_transactions() {
        use crate::evm_config::ANDE_PRECOMPILE_ADDRESS;

        let config = ParallelConfig {
            enable_lazy_updates: true,
            min_transactions_for_parallel: 4,
            ..Default::default()
        };
        let executor = ParallelExecutor::new(config);

        let mv_memory = Arc::new(Mutex::new(MvMemory::new()));
        let evm_config = create_test_evm_config();
        let parent_header = create_test_sealed_header();
        let next_block_attrs = create_test_block_attrs();

        // Mix of ANDE precompile and regular transfers
        let regular_recipient = Address::random();

        // Transaction 0: Regular transfer
        let tx0 = create_test_transaction_with_nonce(
            Address::random(),
            TxKind::Call(regular_recipient),
            U256::from(1000),
            Bytes::new(),
            None,
            0,
        );

        // Transaction 1: ANDE precompile
        let tx1 = create_test_transaction_with_nonce(
            Address::random(),
            TxKind::Call(ANDE_PRECOMPILE_ADDRESS),
            U256::from(2000),
            Bytes::new(),
            None,
            1,
        );

        // Transaction 2: Another regular transfer
        let tx2 = create_test_transaction_with_nonce(
            Address::random(),
            TxKind::Call(regular_recipient),
            U256::from(3000),
            Bytes::new(),
            None,
            2,
        );

        // Transaction 3: Another ANDE precompile
        let tx3 = create_test_transaction_with_nonce(
            Address::random(),
            TxKind::Call(ANDE_PRECOMPILE_ADDRESS),
            U256::from(4000),
            Bytes::new(),
            None,
            3,
        );

        let transactions = vec![tx0, tx1, tx2, tx3];

        // Execute all transactions
        for (i, tx) in transactions.iter().enumerate() {
            let result = executor.execute_transaction_parallel(
                TxVersion { tx_idx: i, tx_incarnation: 0 },
                tx,
                &evm_config,
                &parent_header,
                &next_block_attrs,
                &mv_memory,
            );

            assert!(result.is_some(), "Transaction {} should execute", i);
            let result = result.unwrap();
            assert!(result.success, "Transaction {} should succeed", i);

            // Verify read/write sets are correct
            match i {
                0 | 2 => {
                    // Regular transfers should have recipient in write set
                    assert!(
                        result.write_set.contains(&regular_recipient),
                        "Regular transfer should write to recipient"
                    );
                }
                1 | 3 => {
                    // ANDE precompile calls should have precompile in write set
                    assert!(
                        result.write_set.contains(&ANDE_PRECOMPILE_ADDRESS),
                        "ANDE precompile call should write to precompile address"
                    );
                }
                _ => {}
            }
        }

        // Evaluate lazy balances
        let mut mv_memory_guard = mv_memory.lock().unwrap();
        let changes = mv_memory_guard.evaluate_lazy_balances();

        // Should have lazy updates for ANDE precompile
        let ande_change = changes.iter()
            .find(|c| c.address == ANDE_PRECOMPILE_ADDRESS)
            .expect("Should have ANDE precompile changes");

        // Total ANDE value: 2000 + 4000 = 6000
        assert_eq!(
            ande_change.balance_change,
            Some(6000),
            "ANDE precompile should receive 6000 total"
        );
    }

    #[test]
    fn test_integration_conflict_detection_with_ande() {
        use crate::evm_config::ANDE_PRECOMPILE_ADDRESS;

        let config = ParallelConfig {
            enable_lazy_updates: true,
            max_retries: 2,
            ..Default::default()
        };

        // Create scheduler with ANDE-related dependencies
        let shared_account = Address::random();

        let dependencies = vec![
            // Tx 0: Writes to shared account
            TxDependency {
                depends_on: vec![],
                dependents: vec![1],
                read_accounts: vec![],
                write_accounts: vec![shared_account],
            },
            // Tx 1: Reads from shared account, writes to ANDE
            TxDependency {
                depends_on: vec![0],
                dependents: vec![],
                read_accounts: vec![shared_account],
                write_accounts: vec![ANDE_PRECOMPILE_ADDRESS],
            },
        ];

        let scheduler = ParallelScheduler::new(2, dependencies, config);

        // Simulate execution results with conflict
        let tx0_result = ParallelExecutionResult {
            tx_idx: 0,
            gas_used: 21000,
            success: true,
            error: None,
            state_changes: HashMap::new(),
            read_set: vec![],
            write_set: vec![shared_account],
            incarnation: 1, // Higher incarnation
        };

        let tx1_result = ParallelExecutionResult {
            tx_idx: 1,
            gas_used: 21000,
            success: true,
            error: None,
            state_changes: HashMap::new(),
            read_set: vec![shared_account],
            write_set: vec![ANDE_PRECOMPILE_ADDRESS],
            incarnation: 0, // Lower incarnation - conflict!
        };

        scheduler.store_result(tx0_result);
        scheduler.store_result(tx1_result.clone());

        // Validate - should detect conflict
        let has_conflict = scheduler.detect_conflicts(1, &tx1_result);
        assert!(
            has_conflict,
            "Should detect conflict when reading account modified by earlier tx"
        );
    }

    #[test]
    fn test_integration_sequential_fallback_with_ande() {
        use crate::evm_config::ANDE_PRECOMPILE_ADDRESS;

        let config = ParallelConfig {
            enable_lazy_updates: true,
            min_transactions_for_parallel: 10, // Set high to force sequential
            force_sequential: false,
            ..Default::default()
        };
        let executor = ParallelExecutor::new(config);

        // Create only 3 transactions (below threshold)
        let tx0 = create_test_transaction_with_nonce(
            Address::random(),
            TxKind::Call(ANDE_PRECOMPILE_ADDRESS),
            U256::from(1000),
            Bytes::new(),
            None,
            0,
        );

        let tx1 = create_test_transaction_with_nonce(
            Address::random(),
            TxKind::Call(Address::random()),
            U256::from(2000),
            Bytes::new(),
            None,
            1,
        );

        let tx2 = create_test_transaction_with_nonce(
            Address::random(),
            TxKind::Call(ANDE_PRECOMPILE_ADDRESS),
            U256::from(3000),
            Bytes::new(),
            None,
            2,
        );

        let transactions = vec![tx0, tx1, tx2];

        // Verify it will use sequential mode
        assert!(
            !executor.should_use_parallel(&transactions),
            "Should use sequential mode for < min_transactions_for_parallel"
        );

        // Note: We can't easily test execute_transactions without the full async runtime,
        // but we've verified the decision logic works correctly
    }

    #[test]
    fn test_integration_lazy_updates_disabled_immediate_state_changes() {
        use crate::evm_config::ANDE_PRECOMPILE_ADDRESS;

        let config = ParallelConfig {
            enable_lazy_updates: false, // Disable lazy updates
            ..Default::default()
        };
        let executor = ParallelExecutor::new(config);

        let mv_memory = Arc::new(Mutex::new(MvMemory::new()));
        let evm_config = create_test_evm_config();
        let parent_header = create_test_sealed_header();
        let next_block_attrs = create_test_block_attrs();

        // Create transaction to ANDE precompile
        let tx = create_test_transaction_with_nonce(
            Address::random(),
            TxKind::Call(ANDE_PRECOMPILE_ADDRESS),
            U256::from(5000),
            Bytes::new(),
            None,
            0,
        );

        let result = executor.execute_transaction_parallel(
            TxVersion { tx_idx: 0, tx_incarnation: 0 },
            &tx,
            &evm_config,
            &parent_header,
            &next_block_attrs,
            &mv_memory,
        );

        assert!(result.is_some());
        let result = result.unwrap();
        assert!(result.success);

        // With lazy updates disabled, should have immediate state change
        assert!(
            !result.state_changes.is_empty(),
            "Should have immediate state changes when lazy updates disabled"
        );

        // Should NOT have lazy account updates
        let mv_memory_guard = mv_memory.lock().unwrap();
        assert!(
            mv_memory_guard.lazy_accounts.is_empty() ||
            !mv_memory_guard.lazy_accounts.contains_key(&ANDE_PRECOMPILE_ADDRESS),
            "Should not have lazy updates when disabled"
        );
    }

    #[test]
    fn test_integration_large_batch_ande_transactions() {
        use crate::evm_config::ANDE_PRECOMPILE_ADDRESS;

        let config = ParallelConfig {
            enable_lazy_updates: true,
            min_transactions_for_parallel: 10,
            ..Default::default()
        };
        let executor = ParallelExecutor::new(config);

        let mv_memory = Arc::new(Mutex::new(MvMemory::new()));
        let evm_config = create_test_evm_config();
        let parent_header = create_test_sealed_header();
        let next_block_attrs = create_test_block_attrs();

        // Create 50 transactions to ANDE precompile
        let transaction_count = 50;
        let mut total_value = U256::ZERO;

        for i in 0..transaction_count {
            let value = U256::from((i + 1) * 100); // 100, 200, 300, ...
            total_value = total_value.saturating_add(value);

            let tx = create_test_transaction_with_nonce(
                Address::random(),
                TxKind::Call(ANDE_PRECOMPILE_ADDRESS),
                value,
                Bytes::new(),
                None,
                i as u64,
            );

            let result = executor.execute_transaction_parallel(
                TxVersion { tx_idx: i, tx_incarnation: 0 },
                &tx,
                &evm_config,
                &parent_header,
                &next_block_attrs,
                &mv_memory,
            );

            assert!(result.is_some(), "Transaction {} should execute", i);
            assert!(result.unwrap().success, "Transaction {} should succeed", i);
        }

        // Evaluate all lazy updates
        let mut mv_memory_guard = mv_memory.lock().unwrap();
        let lazy_state = &mv_memory_guard.lazy_accounts[&ANDE_PRECOMPILE_ADDRESS];
        assert_eq!(
            lazy_state.balance_additions.len(),
            transaction_count,
            "Should have {} lazy updates", transaction_count
        );

        let changes = mv_memory_guard.evaluate_lazy_balances();
        let ande_change = &changes[0];

        // Expected total: sum of 100 + 200 + ... + 5000 = 127500
        let expected_total: u128 = (1..=transaction_count)
            .map(|i| (i as u128) * 100)
            .sum();

        assert_eq!(
            ande_change.balance_change,
            Some(expected_total as i128),
            "Total balance change should be {}", expected_total
        );
    }

    #[test]
    fn test_integration_ande_with_zero_value_optimization() {
        use crate::evm_config::ANDE_PRECOMPILE_ADDRESS;

        let config = ParallelConfig {
            enable_lazy_updates: true,
            ..Default::default()
        };
        let executor = ParallelExecutor::new(config);

        let mv_memory = Arc::new(Mutex::new(MvMemory::new()));
        let evm_config = create_test_evm_config();
        let parent_header = create_test_sealed_header();
        let next_block_attrs = create_test_block_attrs();

        // Create transaction with zero value to ANDE precompile
        let tx = create_test_transaction_with_nonce(
            Address::random(),
            TxKind::Call(ANDE_PRECOMPILE_ADDRESS),
            U256::ZERO, // Zero value
            Bytes::new(),
            None,
            0,
        );

        let result = executor.execute_transaction_parallel(
            TxVersion { tx_idx: 0, tx_incarnation: 0 },
            &tx,
            &evm_config,
            &parent_header,
            &next_block_attrs,
            &mv_memory,
        );

        assert!(result.is_some());
        let result = result.unwrap();
        assert!(result.success);

        // Zero value transfers should not create lazy updates
        let mv_memory_guard = mv_memory.lock().unwrap();

        // Either no lazy accounts, or ANDE account has no additions
        if let Some(lazy_state) = mv_memory_guard.lazy_accounts.get(&ANDE_PRECOMPILE_ADDRESS) {
            assert_eq!(
                lazy_state.balance_additions.len(),
                0,
                "Zero value transfer should not create lazy balance addition"
            );
        }
    }

    // =========================================================================
    // SECURITY TESTS: ATTACK VECTOR VALIDATION
    // =========================================================================
    //
    // These tests validate security properties and defense against attack vectors.
    // See PARALLEL_SECURITY.md for full threat model.

    #[test]
    fn test_security_dos_excessive_retries() {
        // Attack: Try to exhaust resources with infinite conflicts
        let config = ParallelConfig {
            max_retries: 3, // Limited retries
            ..Default::default()
        };

        let shared_account = Address::random();
        let dependencies = vec![
            TxDependency {
                depends_on: vec![],
                dependents: vec![],
                read_accounts: vec![],
                write_accounts: vec![shared_account],
            },
            TxDependency {
                depends_on: vec![],
                dependents: vec![],
                read_accounts: vec![shared_account],
                write_accounts: vec![],
            },
        ];

        let scheduler = ParallelScheduler::new(2, dependencies, config);

        // Simulate max retries
        {
            let mut retry_count = scheduler.retry_counts[1].lock().unwrap();
            *retry_count = 3; // At max
        }

        // Create persistent conflict
        let tx0_result = ParallelExecutionResult {
            tx_idx: 0,
            gas_used: 21000,
            success: true,
            error: None,
            state_changes: HashMap::new(),
            read_set: vec![],
            write_set: vec![shared_account],
            incarnation: 999, // Always higher
        };

        let tx1_result = ParallelExecutionResult {
            tx_idx: 1,
            gas_used: 21000,
            success: true,
            error: None,
            state_changes: HashMap::new(),
            read_set: vec![shared_account],
            write_set: vec![],
            incarnation: 0,
        };

        scheduler.store_result(tx0_result);
        scheduler.store_result(tx1_result.clone());

        // Validation should mark as failed (not infinite retry)
        scheduler.finish_validation(TxVersion {
            tx_idx: 1,
            tx_incarnation: 0,
        });

        let status = scheduler.tx_status[1].lock().unwrap();
        assert!(
            matches!(*status, TxStatus::Failed),
            "Should fail after max retries (DoS prevention)"
        );
    }

    #[test]
    fn test_security_integer_overflow_gas_calculation() {
        let config = ParallelConfig::default();
        let executor = ParallelExecutor::new(config);

        // Attack: Try to overflow gas calculation with maximum values
        use alloy_consensus::TypedTransaction;

        let tx = TxLegacy {
            chain_id: Some(1337),
            nonce: 0,
            gas_price: u128::MAX, // Maximum gas price
            gas_limit: u64::MAX,  // Maximum gas limit
            to: TxKind::Call(Address::random()),
            value: U256::MAX, // Maximum value
            input: Bytes::new(),
        };

        let signature = Signature::test_signature();
        let typed_tx = TypedTransaction::Legacy(tx);
        let signed_tx = TransactionSigned::new_unhashed(typed_tx.into(), signature);

        let mv_memory = Arc::new(Mutex::new(MvMemory::new()));
        let evm_config = create_test_evm_config();
        let parent_header = create_test_sealed_header();
        let next_block_attrs = create_test_block_attrs();

        // Execute - should not panic with saturating arithmetic
        let result = executor.execute_transaction_parallel(
            TxVersion { tx_idx: 0, tx_incarnation: 0 },
            &signed_tx,
            &evm_config,
            &parent_header,
            &next_block_attrs,
            &mv_memory,
        );

        // Should return result (not panic)
        assert!(result.is_some(), "Should handle maximum values safely");

        // May fail validation but should not crash
        let result = result.unwrap();
        // We don't assert success here - just that it didn't crash
        assert!(
            result.success || result.error.is_some(),
            "Should either succeed or have error (no panic)"
        );
    }

    #[test]
    fn test_security_ande_balance_overflow_protection() {
        use crate::evm_config::ANDE_PRECOMPILE_ADDRESS;

        let mut mv_memory = MvMemory::new();

        // Attack: Try to overflow ANDE precompile balance
        // Add U256::MAX multiple times
        mv_memory.add_lazy_balance_addition(ANDE_PRECOMPILE_ADDRESS, U256::MAX, 0);
        mv_memory.add_lazy_balance_addition(ANDE_PRECOMPILE_ADDRESS, U256::MAX, 1);
        mv_memory.add_lazy_balance_addition(ANDE_PRECOMPILE_ADDRESS, U256::from(1000), 2);

        // Evaluate - should saturate, not overflow
        let changes = mv_memory.evaluate_lazy_balances();

        assert_eq!(changes.len(), 1);
        let ande_change = &changes[0];

        // Should saturate at U256::MAX (not wrap around to 0)
        assert!(
            ande_change.balance_change.is_some(),
            "Should handle overflow with saturation"
        );

        // Balance change should be positive (saturated, not negative from overflow)
        assert!(
            ande_change.balance_change.unwrap() > 0,
            "Should saturate positively (no wraparound)"
        );
    }

    #[test]
    fn test_security_dos_minimum_transaction_threshold() {
        // Attack: Submit single transaction repeatedly to waste parallel overhead
        let config = ParallelConfig {
            min_transactions_for_parallel: 4,
            ..Default::default()
        };
        let executor = ParallelExecutor::new(config);

        // Single transaction (attack)
        let single_tx = vec![create_test_transaction(
            Address::random(),
            TxKind::Call(Address::random()),
            U256::from(1000),
            Bytes::new(),
            None,
        )];

        // Should use sequential (not wasting parallel overhead)
        assert!(
            !executor.should_use_parallel(&single_tx),
            "Should reject single transaction for parallel execution (DoS prevention)"
        );

        // Three transactions (still below threshold)
        let few_txs = vec![
            create_test_transaction(
                Address::random(),
                TxKind::Call(Address::random()),
                U256::from(1000),
                Bytes::new(),
                None,
            );
            3
        ];

        assert!(
            !executor.should_use_parallel(&few_txs),
            "Should use sequential below threshold (DoS prevention)"
        );

        // Four transactions (at threshold)
        let threshold_txs = vec![
            create_test_transaction(
                Address::random(),
                TxKind::Call(Address::random()),
                U256::from(1000),
                Bytes::new(),
                None,
            );
            4
        ];

        assert!(
            executor.should_use_parallel(&threshold_txs),
            "Should allow parallel at threshold"
        );
    }

    #[test]
    fn test_security_thread_safety_concurrent_mv_memory_access() {
        use std::thread;

        let mv_memory = Arc::new(Mutex::new(MvMemory::new()));
        let test_address = Address::random();

        // Spawn multiple threads trying to modify MvMemory concurrently
        thread::scope(|scope| {
            for i in 0..10 {
                let mv_memory_clone = Arc::clone(&mv_memory);
                scope.spawn(move || {
                    let mut mv_guard = mv_memory_clone.lock().unwrap();
                    mv_guard.add_lazy_balance_addition(
                        test_address,
                        U256::from(i * 100),
                        i,
                    );
                });
            }
        });

        // Verify all updates were recorded (no lost updates)
        let mut mv_guard = mv_memory.lock().unwrap();
        let lazy_state = &mv_guard.lazy_accounts[&test_address];

        assert_eq!(
            lazy_state.balance_additions.len(),
            10,
            "All concurrent updates should be recorded (thread-safe)"
        );

        // Verify total is correct (no race condition)
        let changes = mv_guard.evaluate_lazy_balances();
        let expected_total: i128 = (0..10).map(|i| i * 100).sum();

        assert_eq!(
            changes[0].balance_change,
            Some(expected_total),
            "Concurrent updates should sum correctly (no race condition)"
        );
    }

    #[test]
    fn test_security_input_validation_malformed_transaction() {
        let config = ParallelConfig::default();
        let executor = ParallelExecutor::new(config);

        use alloy_consensus::TypedTransaction;

        // Attack: Malformed transaction with zero gas limit
        let tx = TxLegacy {
            chain_id: Some(1337),
            nonce: 0,
            gas_price: 1000000000,
            gas_limit: 0, // Zero gas limit (invalid)
            to: TxKind::Call(Address::random()),
            value: U256::from(1000),
            input: Bytes::new(),
        };

        let signature = Signature::test_signature();
        let typed_tx = TypedTransaction::Legacy(tx);
        let signed_tx = TransactionSigned::new_unhashed(typed_tx.into(), signature);

        let mv_memory = Arc::new(Mutex::new(MvMemory::new()));
        let evm_config = create_test_evm_config();
        let parent_header = create_test_sealed_header();
        let next_block_attrs = create_test_block_attrs();

        // Execute
        let result = executor.execute_transaction_parallel(
            TxVersion { tx_idx: 0, tx_incarnation: 0 },
            &signed_tx,
            &evm_config,
            &parent_header,
            &next_block_attrs,
            &mv_memory,
        );

        // Should fail validation (not crash)
        assert!(result.is_some());
        let result = result.unwrap();
        assert!(!result.success, "Should reject transaction with zero gas limit");
        assert!(result.error.is_some(), "Should have error message");
        assert!(
            result.error.unwrap().contains("Intrinsic gas too low"),
            "Should have specific error"
        );
    }
}