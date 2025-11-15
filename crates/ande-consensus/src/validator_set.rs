//! Validator set management with CometBFT-style proposer selection

use crate::{
    error::{ConsensusError, Result},
    types::ValidatorInfo,
};
use alloy_primitives::Address;
use std::collections::HashMap;
use tracing::{debug, info, warn};

/// Manages the active validator set and proposer selection
#[derive(Debug, Clone)]
pub struct ValidatorSet {
    /// Map of validator address to validator info
    validators: HashMap<Address, ValidatorInfo>,

    /// Ordered list of active validator addresses
    active_validators: Vec<Address>,

    /// Total voting power of all active validators
    total_voting_power: u64,

    /// Current proposer address
    current_proposer: Option<Address>,

    /// Index of current proposer in active_validators
    proposer_index: usize,

    /// Last block number where validator set was updated
    last_update_block: u64,
}

impl ValidatorSet {
    /// Create a new empty validator set
    pub fn new() -> Self {
        Self {
            validators: HashMap::new(),
            active_validators: Vec::new(),
            total_voting_power: 0,
            current_proposer: None,
            proposer_index: 0,
            last_update_block: 0,
        }
    }

    /// Update the validator set from on-chain data
    pub fn update_from_chain(
        &mut self,
        validators: Vec<ValidatorInfo>,
        block_number: u64,
    ) -> Result<()> {
        info!(
            count = validators.len(),
            block = block_number,
            "Updating validator set from chain"
        );

        // Clear existing validators
        self.validators.clear();
        self.active_validators.clear();
        self.total_voting_power = 0;

        // Add new validators
        for validator in validators {
            if validator.can_propose() {
                self.total_voting_power += validator.power;
                self.active_validators.push(validator.validator);
            }
            self.validators.insert(validator.validator, validator);
        }

        self.last_update_block = block_number;

        // Reset proposer selection if validator set changed
        if !self.active_validators.is_empty() {
            self.proposer_index = 0;
            self.select_next_proposer()?;
        }

        info!(
            active_count = self.active_validators.len(),
            total_power = self.total_voting_power,
            "Validator set updated successfully"
        );

        Ok(())
    }

    /// Get validator info by address
    pub fn get_validator(&self, address: &Address) -> Option<&ValidatorInfo> {
        self.validators.get(address)
    }

    /// Get mutable validator info by address
    pub fn get_validator_mut(&mut self, address: &Address) -> Option<&mut ValidatorInfo> {
        self.validators.get_mut(address)
    }

    /// Check if an address is an active validator
    pub fn is_active_validator(&self, address: &Address) -> bool {
        self.validators
            .get(address)
            .map_or(false, ValidatorInfo::can_propose)
    }

    /// Get current proposer address
    pub fn current_proposer(&self) -> Option<Address> {
        self.current_proposer
    }

    /// Get total voting power
    pub const fn total_voting_power(&self) -> u64 {
        self.total_voting_power
    }

    /// Get number of active validators
    pub fn active_count(&self) -> usize {
        self.active_validators.len()
    }

    /// Get all active validators
    pub fn active_validators(&self) -> &[Address] {
        &self.active_validators
    }

    /// Calculate BFT threshold (2/3 + 1)
    pub const fn bft_threshold(&self) -> u64 {
        (self.total_voting_power * 2) / 3 + 1
    }

    /// Select next proposer using weighted round-robin (CometBFT algorithm)
    ///
    /// # Algorithm
    ///
    /// 1. For each validator, increment accumulated_priority by voting_power
    /// 2. Select validator with highest accumulated_priority
    /// 3. Decrement selected validator's priority by total_voting_power
    ///
    /// This ensures validators are selected proportionally to their voting power
    pub fn select_next_proposer(&mut self) -> Result<Address> {
        if self.active_validators.is_empty() {
            return Err(ConsensusError::Internal(
                "No active validators available".to_string(),
            ));
        }

        // Simple round-robin for single validator
        if self.active_validators.len() == 1 {
            let proposer = self.active_validators[0];
            self.current_proposer = Some(proposer);
            debug!(proposer = ?proposer, "Single validator, selected as proposer");
            return Ok(proposer);
        }

        // CometBFT weighted round-robin algorithm
        let mut max_priority = i64::MIN;
        let mut selected_addr = Address::ZERO;
        let mut selected_idx = 0;

        // Step 1 & 2: Increment priorities and find maximum
        for (idx, &addr) in self.active_validators.iter().enumerate() {
            if let Some(validator) = self.validators.get_mut(&addr) {
                if !validator.can_propose() {
                    continue;
                }

                // Increment priority by voting power
                validator.accumulated_priority += validator.power as i64;

                debug!(
                    validator = ?addr,
                    power = validator.power,
                    priority = validator.accumulated_priority,
                    "Updated validator priority"
                );

                // Track maximum
                if validator.accumulated_priority > max_priority {
                    max_priority = validator.accumulated_priority;
                    selected_addr = addr;
                    selected_idx = idx;
                }
            }
        }

        if selected_addr == Address::ZERO {
            return Err(ConsensusError::Internal(
                "No eligible proposer found".to_string(),
            ));
        }

        // Step 3: Decrement selected validator's priority
        if let Some(validator) = self.validators.get_mut(&selected_addr) {
            validator.accumulated_priority -= self.total_voting_power as i64;

            info!(
                proposer = ?selected_addr,
                power = validator.power,
                new_priority = validator.accumulated_priority,
                "Selected new proposer"
            );
        }

        self.current_proposer = Some(selected_addr);
        self.proposer_index = selected_idx;

        Ok(selected_addr)
    }

    /// Force rotation to next validator (used on timeout)
    pub fn force_next_proposer(&mut self) -> Result<Address> {
        if self.active_validators.is_empty() {
            return Err(ConsensusError::Internal(
                "No active validators for rotation".to_string(),
            ));
        }

        // Round-robin to next validator
        self.proposer_index = (self.proposer_index + 1) % self.active_validators.len();
        let next_proposer = self.active_validators[self.proposer_index];

        // Skip jailed validators
        let mut attempts = 0;
        while attempts < self.active_validators.len() {
            if let Some(validator) = self.validators.get(&next_proposer) {
                if validator.can_propose() {
                    self.current_proposer = Some(next_proposer);
                    warn!(
                        proposer = ?next_proposer,
                        reason = "force rotation",
                        "Forced proposer rotation"
                    );
                    return Ok(next_proposer);
                }
            }

            // Try next validator
            self.proposer_index = (self.proposer_index + 1) % self.active_validators.len();
            attempts += 1;
        }

        Err(ConsensusError::Internal(
            "No eligible validator found during force rotation".to_string(),
        ))
    }

    /// Update validator statistics after block production
    pub fn record_block_produced(&mut self, producer: &Address, block_number: u64) -> Result<()> {
        let validator = self
            .validators
            .get_mut(producer)
            .ok_or_else(|| ConsensusError::ValidatorNotFound(*producer))?;

        validator.total_blocks_produced += 1;
        validator.last_block_produced = block_number;

        // Update uptime
        let total_blocks = validator.total_blocks_produced + validator.total_blocks_missed;
        if total_blocks > 0 {
            validator.uptime =
                ((validator.total_blocks_produced * 10000) / total_blocks) as u16;
        }

        debug!(
            validator = ?producer,
            total_produced = validator.total_blocks_produced,
            uptime = validator.uptime,
            "Updated validator block production stats"
        );

        Ok(())
    }

    /// Record missed block
    pub fn record_block_missed(&mut self, validator_addr: &Address) -> Result<()> {
        let validator = self
            .validators
            .get_mut(validator_addr)
            .ok_or_else(|| ConsensusError::ValidatorNotFound(*validator_addr))?;

        validator.total_blocks_missed += 1;

        // Update uptime
        let total_blocks = validator.total_blocks_produced + validator.total_blocks_missed;
        if total_blocks > 0 {
            validator.uptime =
                ((validator.total_blocks_produced * 10000) / total_blocks) as u16;
        }

        warn!(
            validator = ?validator_addr,
            total_missed = validator.total_blocks_missed,
            uptime = validator.uptime,
            "Recorded missed block"
        );

        Ok(())
    }

    /// Verify that the given address is the expected proposer
    pub fn verify_proposer(&self, actual: Address) -> Result<()> {
        let expected = self
            .current_proposer
            .ok_or_else(|| ConsensusError::Internal("No proposer selected".to_string()))?;

        if actual != expected {
            return Err(ConsensusError::InvalidProposer { expected, actual });
        }

        Ok(())
    }

    /// Get validator set statistics
    pub fn stats(&self) -> ValidatorSetStats {
        let active_count = self.active_validators.len();
        let jailed_count = self
            .validators
            .values()
            .filter(|v| v.jailed)
            .count();
        let inactive_count = self
            .validators
            .values()
            .filter(|v| !v.active)
            .count();

        let avg_uptime = if active_count > 0 {
            let total_uptime: u64 = self
                .active_validators
                .iter()
                .filter_map(|addr| self.validators.get(addr))
                .map(|v| u64::from(v.uptime))
                .sum();
            total_uptime / active_count as u64
        } else {
            0
        };

        ValidatorSetStats {
            total_validators: self.validators.len(),
            active_count,
            jailed_count,
            inactive_count,
            total_voting_power: self.total_voting_power,
            bft_threshold: self.bft_threshold(),
            average_uptime: avg_uptime as u16,
            last_update_block: self.last_update_block,
        }
    }
}

impl Default for ValidatorSet {
    fn default() -> Self {
        Self::new()
    }
}

/// Statistics about the validator set
#[derive(Debug, Clone)]
pub struct ValidatorSetStats {
    /// Total number of validators (active + inactive + jailed)
    pub total_validators: usize,

    /// Number of active validators
    pub active_count: usize,

    /// Number of jailed validators
    pub jailed_count: usize,

    /// Number of inactive validators
    pub inactive_count: usize,

    /// Total voting power
    pub total_voting_power: u64,

    /// BFT threshold (2/3 + 1)
    pub bft_threshold: u64,

    /// Average uptime (basis points)
    pub average_uptime: u16,

    /// Last block where set was updated
    pub last_update_block: u64,
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloy_primitives::B256;

    fn create_test_validator(addr: Address, power: u64, active: bool) -> ValidatorInfo {
        ValidatorInfo {
            validator: addr,
            p2p_peer_id: B256::ZERO,
            rpc_endpoint: "http://test".to_string(),
            stake: Default::default(),
            power,
            accumulated_priority: 0,
            total_blocks_produced: 0,
            total_blocks_missed: 0,
            uptime: 10000,
            last_block_produced: 0,
            registered_at: 0,
            jailed: false,
            active,
            is_permanent: false,
        }
    }

    #[test]
    fn test_empty_validator_set() {
        let set = ValidatorSet::new();
        assert_eq!(set.active_count(), 0);
        assert_eq!(set.total_voting_power(), 0);
        assert!(set.current_proposer().is_none());
    }

    #[test]
    fn test_single_validator() {
        let mut set = ValidatorSet::new();
        let addr = Address::from([1u8; 20]);
        let validators = vec![create_test_validator(addr, 100, true)];

        set.update_from_chain(validators, 1).unwrap();

        assert_eq!(set.active_count(), 1);
        assert_eq!(set.total_voting_power(), 100);

        let proposer = set.select_next_proposer().unwrap();
        assert_eq!(proposer, addr);
    }

    #[test]
    fn test_weighted_round_robin() {
        let mut set = ValidatorSet::new();

        // Validator A: 100 power, Validator B: 300 power
        let addr_a = Address::from([1u8; 20]);
        let addr_b = Address::from([2u8; 20]);

        let validators = vec![
            create_test_validator(addr_a, 100, true),
            create_test_validator(addr_b, 300, true),
        ];

        set.update_from_chain(validators, 1).unwrap();

        // Track selections over multiple rounds
        let mut selections_a = 0;
        let mut selections_b = 0;

        for _ in 0..400 {
            let proposer = set.select_next_proposer().unwrap();
            if proposer == addr_a {
                selections_a += 1;
            } else if proposer == addr_b {
                selections_b += 1;
            }
        }

        // B should be selected ~3x more than A (300/100 ratio)
        // Allow some variance
        let ratio = selections_b as f64 / selections_a as f64;
        assert!((2.5..3.5).contains(&ratio));
    }

    #[test]
    fn test_verify_proposer() {
        let mut set = ValidatorSet::new();
        let addr = Address::from([1u8; 20]);
        let validators = vec![create_test_validator(addr, 100, true)];

        set.update_from_chain(validators, 1).unwrap();
        set.select_next_proposer().unwrap();

        // Valid proposer
        assert!(set.verify_proposer(addr).is_ok());

        // Invalid proposer
        let wrong_addr = Address::from([2u8; 20]);
        assert!(set.verify_proposer(wrong_addr).is_err());
    }

    #[test]
    fn test_record_block_stats() {
        let mut set = ValidatorSet::new();
        let addr = Address::from([1u8; 20]);
        let validators = vec![create_test_validator(addr, 100, true)];

        set.update_from_chain(validators, 1).unwrap();

        // Record successful production
        set.record_block_produced(&addr, 100).unwrap();
        let validator = set.get_validator(&addr).unwrap();
        assert_eq!(validator.total_blocks_produced, 1);
        assert_eq!(validator.uptime, 10000); // 100%

        // Record miss
        set.record_block_missed(&addr).unwrap();
        let validator = set.get_validator(&addr).unwrap();
        assert_eq!(validator.total_blocks_missed, 1);
        assert_eq!(validator.uptime, 5000); // 50%
    }

    #[test]
    fn test_bft_threshold() {
        let mut set = ValidatorSet::new();
        let validators = vec![
            create_test_validator(Address::from([1u8; 20]), 100, true),
            create_test_validator(Address::from([2u8; 20]), 200, true),
            create_test_validator(Address::from([3u8; 20]), 300, true),
        ];

        set.update_from_chain(validators, 1).unwrap();

        // Total power = 600, threshold = 2/3 + 1 = 401
        assert_eq!(set.total_voting_power(), 600);
        assert_eq!(set.bft_threshold(), 401);
    }
}
