//! Parallel EVM Execution Module
//!
//! This module provides parallel transaction execution capabilities for AndeChain,
//! enabling significant throughput improvements while maintaining ANDE Token Duality.

pub mod executor;
pub mod scheduler;
pub mod mv_memory;
pub mod config;

pub use executor::{
    ParallelExecutor, ParallelExecutionResult,
    ParallelTask, TxVersion, TxStatus, AccountStateChange, TxIdx
};
pub use config::ParallelConfig;
pub use scheduler::ParallelScheduler;
pub use mv_memory::MvMemory;