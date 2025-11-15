//! Parallel Task Scheduler
//!
//! Coordinates execution and validation tasks among worker threads,
//! managing transaction dependencies and execution order.

use crate::parallel::{ParallelTask, TxVersion, TxStatus, ParallelConfig, TxIdx};
use std::{
    collections::VecDeque,
    sync::{Arc, Mutex},
};

/// Transaction dependency information
#[derive(Debug, Clone)]
pub struct TxDependency {
    /// Transactions that this transaction depends on
    pub depends_on: Vec<TxIdx>,
    /// Transactions that depend on this one
    pub dependents: Vec<TxIdx>,
    /// Accounts this transaction reads from
    pub read_accounts: Vec<alloy_primitives::Address>,
    /// Accounts this transaction writes to
    pub write_accounts: Vec<alloy_primitives::Address>,
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
    /// Configuration
    #[allow(dead_code)]
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
    pub fn finish_validation(&self, _tx_version: TxVersion) {
        // TODO: Implement validation completion logic
    }

    /// Add a dependency between transactions
    pub fn add_dependency(&self, _tx_idx: TxIdx, _blocking_tx_idx: TxIdx) -> bool {
        // Implementation for adding runtime dependencies
        true
    }
}