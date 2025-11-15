//! Multi-Version Memory for Parallel Execution
//!
//! Tracks multiple versions of state during parallel transaction execution,
//! handling conflicts and lazy updates for ANDE Token Duality.

use crate::parallel::{AccountStateChange, TxVersion};
use alloy_primitives::{Address, U256};
use std::collections::HashMap;

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
    /// Account was self-destructed
    SelfDestructed,
}

/// Lazy account state for deferred balance calculations
#[derive(Debug, Clone)]
pub struct LazyAccountState {
    /// Base balance before lazy updates
    pub base_balance: U256,
    /// Base nonce before lazy updates
    pub base_nonce: u64,
    /// Pending balance additions
    pub balance_additions: Vec<(usize, U256)>,
    /// Pending balance subtractions
    pub balance_subtractions: Vec<(usize, U256)>,
    /// Pending nonce increments
    pub nonce_increments: Vec<usize>,
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
    pub fn add_lazy_balance_addition(&mut self, address: Address, amount: U256, tx_idx: usize) {
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
    pub fn add_lazy_balance_subtraction(&mut self, address: Address, amount: U256, tx_idx: usize) {
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
    pub fn add_lazy_nonce_increment(&mut self, address: Address, tx_idx: usize) {
        let lazy_state = self.lazy_accounts.entry(address).or_insert_with(|| LazyAccountState {
            base_balance: U256::ZERO,
            base_nonce: 0,
            balance_additions: Vec::new(),
            balance_subtractions: Vec::new(),
            nonce_increments: Vec::new(),
        });
        lazy_state.nonce_increments.push(tx_idx);
    }

    /// Set base account state for lazy calculations
    pub fn set_base_account_state(&mut self, address: Address, balance: U256, nonce: u64) {
        let lazy_state = self.lazy_accounts.entry(address).or_insert_with(|| LazyAccountState {
            base_balance: balance,
            base_nonce: nonce,
            balance_additions: Vec::new(),
            balance_subtractions: Vec::new(),
            nonce_increments: Vec::new(),
        });
        lazy_state.base_balance = balance;
        lazy_state.base_nonce = nonce;
    }

    /// Evaluate lazy balances and return final state changes
    pub fn evaluate_lazy_balances(&mut self) -> Vec<AccountStateChange> {
        let mut changes = Vec::new();

        for (address, lazy_state) in &self.lazy_accounts {
            // Calculate final balance
            let mut final_balance = lazy_state.base_balance;

            // Add all pending additions
            for (_, amount) in &lazy_state.balance_additions {
                final_balance = final_balance.saturating_add(*amount);
            }

            // Subtract all pending subtractions
            for (_, amount) in &lazy_state.balance_subtractions {
                final_balance = final_balance.saturating_sub(*amount);
            }

            // Calculate final nonce
            let final_nonce = lazy_state.base_nonce + lazy_state.nonce_increments.len() as u64;

            // Create state change
            let balance_change = final_balance.saturating_sub(lazy_state.base_balance);
            let balance_change_i128 = if balance_change > U256::from(u128::MAX) {
                i128::MAX
            } else {
                balance_change.to::<u128>() as i128
            };

            let nonce_change = if final_nonce != lazy_state.base_nonce {
                Some(final_nonce)
            } else {
                None
            };

            changes.push(AccountStateChange {
                address: *address,
                balance_change: Some(balance_change_i128),
                nonce_change,
                storage_changes: HashMap::new(),
            });
        }

        changes
    }

    /// Get lazy accounts that need evaluation
    pub fn get_lazy_accounts(&self) -> impl Iterator<Item = (&Address, &LazyAccountState)> {
        self.lazy_accounts.iter()
    }

    /// Clear lazy accounts after evaluation
    pub fn clear_lazy_accounts(&mut self) {
        self.lazy_accounts.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lazy_balance_calculations() {
        let mut mv_memory = MvMemory::new();
        let address = Address::random();

        // Set base state
        mv_memory.set_base_account_state(address, U256::from(1000), 5);

        // Add lazy operations
        mv_memory.add_lazy_balance_addition(address, U256::from(200), 0);
        mv_memory.add_lazy_balance_subtraction(address, U256::from(100), 1);
        mv_memory.add_lazy_nonce_increment(address, 2);

        // Evaluate changes
        let changes = mv_memory.evaluate_lazy_balances();

        assert_eq!(changes.len(), 1);
        assert_eq!(changes[0].address, address);
        assert_eq!(changes[0].balance_change, Some(100)); // +200 -100
        assert_eq!(changes[0].nonce_change, Some(6)); // 5 + 1
    }

    #[test]
    fn test_multiple_lazy_operations() {
        let mut mv_memory = MvMemory::new();
        let address = Address::random();

        mv_memory.set_base_account_state(address, U256::from(100), 1);

        // Multiple additions and subtractions
        mv_memory.add_lazy_balance_addition(address, U256::from(50), 0);
        mv_memory.add_lazy_balance_addition(address, U256::from(30), 1);
        mv_memory.add_lazy_balance_subtraction(address, U256::from(20), 2);
        mv_memory.add_lazy_balance_subtraction(address, U256::from(10), 3);

        let changes = mv_memory.evaluate_lazy_balances();

        assert_eq!(changes[0].balance_change, Some(50)); // 100 + 50 + 30 - 20 - 10
    }
}