//! MEV Fee Redistribution for ANDE Chain
//!
//! Implements fair MEV distribution by redirecting base fees and detected MEV profits
//! to a distribution contract that shares revenue among all validators.
//!
//! ## Architecture
//!
//! ```text
//! Transaction → AndeHandler → AndeMevRedirect
//!   ├─ Detect MEV type (sandwich, arbitrage, liquidation)
//!   ├─ Calculate MEV profit
//!   └─ Redirect to distribution contract
//!       └─ Fair share among validators based on stake weight
//! ```

use alloy_primitives::{Address, U256};
use reth_revm::revm::{
    context_interface::{journaled_state::JournalTr, Block, ContextTr},
    database_interface::Database,
};
use thiserror::Error;

/// MEV type classification for analytics and distribution
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MevType {
    /// Sandwich attack (frontrun + backrun)
    Sandwich,
    /// Arbitrage opportunity
    Arbitrage,
    /// Liquidation
    Liquidation,
    /// Base fee only (no MEV detected)
    BaseFeeOnly,
}

/// MEV detection result with profit amount
#[derive(Debug, Clone, Copy)]
pub struct MevDetection {
    /// Type of MEV detected
    pub mev_type: MevType,
    /// Estimated MEV profit in wei
    pub profit: U256,
    /// Gas used by the transaction
    pub gas_used: u64,
}

/// Encapsulates the policy of redirecting MEV profits to a fair distribution contract.
///
/// Similar to evstack's `BaseFeeRedirect` but with enhanced MEV detection and tracking.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AndeMevRedirect {
    /// Address of the MEV distribution contract
    mev_sink: Address,
    /// Minimum profit threshold to consider as MEV (in wei)
    min_mev_threshold: U256,
}

impl AndeMevRedirect {
    /// Creates a new MEV redirect policy.
    ///
    /// # Arguments
    ///
    /// * `mev_sink` - Distribution contract address (must not be zero address)
    /// * `min_mev_threshold` - Minimum profit to classify as MEV (default: 0.001 ETH)
    ///
    /// # Panics
    ///
    /// Panics if `mev_sink` is the zero address. Use `try_new()` for Result-based validation.
    ///
    /// # Security
    ///
    /// M-2 FIX: Validates MEV sink address to prevent loss of funds
    pub fn new(mev_sink: Address, min_mev_threshold: U256) -> Self {
        // SECURITY (M-2): Validate sink is not zero address
        assert!(!mev_sink.is_zero(), "MEV sink cannot be zero address");

        Self {
            mev_sink,
            min_mev_threshold,
        }
    }

    /// Creates a new MEV redirect policy with validation.
    ///
    /// # Arguments
    ///
    /// * `mev_sink` - Distribution contract address
    /// * `min_mev_threshold` - Minimum profit to classify as MEV
    ///
    /// # Errors
    ///
    /// Returns `MevValidationError::ZeroAddress` if sink is zero address.
    ///
    /// # Security
    ///
    /// M-2 FIX: Validates MEV sink address to prevent loss of funds
    pub fn try_new(mev_sink: Address, min_mev_threshold: U256) -> Result<Self, MevValidationError> {
        if mev_sink.is_zero() {
            return Err(MevValidationError::ZeroAddress);
        }

        Ok(Self {
            mev_sink,
            min_mev_threshold,
        })
    }

    /// Default threshold: 0.001 ETH = 1_000_000_000_000_000 wei
    pub const DEFAULT_MIN_MEV_THRESHOLD: U256 = U256::from_limbs([1_000_000_000_000_000u64, 0, 0, 0]);

    /// Creates a new redirect with default threshold
    ///
    /// # Security
    ///
    /// M-2 FIX: Validates MEV sink address to prevent loss of funds
    pub fn with_default_threshold(mev_sink: Address) -> Self {
        Self::new(mev_sink, Self::DEFAULT_MIN_MEV_THRESHOLD)
    }

    /// Returns the configured sink address
    pub const fn mev_sink(&self) -> Address {
        self.mev_sink
    }

    /// Returns the minimum MEV threshold
    pub const fn min_threshold(&self) -> U256 {
        self.min_mev_threshold
    }

    /// Applies the MEV redirect by crediting the distribution contract.
    ///
    /// This function:
    /// 1. Calculates base fee portion
    /// 2. Detects MEV type and profit
    /// 3. Credits total amount to mev_sink
    /// 4. Returns detection info for logging
    ///
    /// # Returns
    ///
    /// `MevDetection` containing MEV type, profit, and gas used
    pub fn apply<CTX>(
        &self,
        ctx: &mut CTX,
        gas_used: u64,
    ) -> Result<MevDetection, MevRedirectError<<CTX::Db as Database>::Error>>
    where
        CTX: ContextTr,
        CTX::Journal: JournalTr<Database = CTX::Db>,
        CTX::Db: Database,
        <CTX::Db as Database>::Error: std::error::Error,
    {
        // Get base fee from block context
        let base_fee = ctx.block().basefee();
        
        if gas_used == 0 {
            return Ok(MevDetection {
                mev_type: MevType::BaseFeeOnly,
                profit: U256::ZERO,
                gas_used: 0,
            });
        }

        // Calculate base fee portion
        let base_fee_amount = U256::from(base_fee) * U256::from(gas_used);

        // TODO: Implement MEV detection logic
        // For now, we only redirect base fees
        // Future: Analyze transaction patterns to detect sandwich, arbitrage, liquidations
        let mev_type = MevType::BaseFeeOnly;
        let total_amount = base_fee_amount;

        if total_amount.is_zero() {
            return Ok(MevDetection {
                mev_type,
                profit: U256::ZERO,
                gas_used,
            });
        }

        // Credit the MEV sink account
        let journal = ctx.journal_mut();
        journal
            .load_account(self.mev_sink)
            .map_err(MevRedirectError::Database)?;
        journal
            .balance_incr(self.mev_sink, total_amount)
            .map_err(MevRedirectError::Database)?;

        Ok(MevDetection {
            mev_type,
            profit: total_amount,
            gas_used,
        })
    }

    /// Detects MEV type based on transaction context.
    ///
    /// This is a placeholder for future MEV detection logic.
    /// Future implementation will analyze:
    /// - Transaction ordering patterns
    /// - Price impacts
    /// - DEX pool states
    /// - Flash loan usage
    #[allow(dead_code)]
    fn detect_mev_type<CTX>(&self, _ctx: &CTX) -> MevType
    where
        CTX: ContextTr,
    {
        // TODO: Implement actual MEV detection
        // For now, return BaseFeeOnly
        MevType::BaseFeeOnly
    }

    /// Calculates MEV profit for detected MEV type.
    ///
    /// Future implementation will calculate actual profit based on:
    /// - Price changes in DEX pools
    /// - Liquidation bonuses
    /// - Arbitrage spreads
    #[allow(dead_code)]
    fn calculate_mev_profit<CTX>(
        &self,
        _ctx: &CTX,
        _mev_type: MevType,
    ) -> U256
    where
        CTX: ContextTr,
    {
        // TODO: Implement MEV profit calculation
        U256::ZERO
    }
}

impl From<Address> for AndeMevRedirect {
    fn from(value: Address) -> Self {
        Self::with_default_threshold(value)
    }
}

/// Errors that can occur during MEV redirect validation
#[derive(Debug, Error, Clone, Copy, PartialEq, Eq)]
pub enum MevValidationError {
    /// MEV sink address cannot be zero
    #[error("MEV sink address cannot be zero address")]
    ZeroAddress,
}

/// Errors that can occur when applying MEV redirect
#[derive(Debug, Error)]
pub enum MevRedirectError<DbError> {
    /// Database error from journal operations
    #[error("failed to update MEV sink account: {0}")]
    Database(#[from] DbError),
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloy_primitives::address;
    use reth_revm::revm::{
        context::Context,
        context::{BlockEnv, CfgEnv, TxEnv},
        database::EmptyDB,
        primitives::hardfork::SpecId,
    };

    type TestContext = Context<BlockEnv, TxEnv, CfgEnv<SpecId>, EmptyDB>;

    const BASE_FEE: u64 = 100;
    const GAS_USED: u64 = 21_000;

    #[test]
    fn test_apply_redirects_base_fee() {
        let sink = address!("0x0000000000000000000000000000000000000001");
        let redirect = AndeMevRedirect::with_default_threshold(sink);
        
        let mut ctx = setup_context(BASE_FEE, sink);
        
        let result = redirect.apply(&mut ctx, GAS_USED).expect("apply succeeds");
        
        assert_eq!(result.mev_type, MevType::BaseFeeOnly);
        assert_eq!(result.gas_used, GAS_USED);
        
        let expected_amount = U256::from(BASE_FEE) * U256::from(GAS_USED);
        assert_eq!(result.profit, expected_amount);
        
        let sink_balance = ctx.journal().account(sink).info.balance;
        assert_eq!(sink_balance, expected_amount);
    }

    #[test]
    fn test_apply_skips_when_no_gas() {
        let sink = address!("0x0000000000000000000000000000000000000002");
        let redirect = AndeMevRedirect::with_default_threshold(sink);
        
        let mut ctx = setup_context(BASE_FEE, sink);
        
        let result = redirect.apply(&mut ctx, 0).expect("apply succeeds");
        
        assert_eq!(result.gas_used, 0);
        assert_eq!(result.profit, U256::ZERO);
        
        let sink_balance = ctx.journal().account(sink).info.balance;
        assert_eq!(sink_balance, U256::ZERO);
    }

    #[test]
    fn test_default_threshold() {
        assert_eq!(
            AndeMevRedirect::DEFAULT_MIN_MEV_THRESHOLD,
            U256::from(1_000_000_000_000_000u64)
        );
    }

    // M-2 SECURITY FIX TESTS: MEV Sink Validation

    #[test]
    #[should_panic(expected = "MEV sink cannot be zero address")]
    fn test_new_rejects_zero_address() {
        let _ = AndeMevRedirect::new(Address::ZERO, U256::from(1000));
    }

    #[test]
    fn test_try_new_rejects_zero_address() {
        let result = AndeMevRedirect::try_new(Address::ZERO, U256::from(1000));
        assert_eq!(result.unwrap_err(), MevValidationError::ZeroAddress);
    }

    #[test]
    fn test_try_new_accepts_valid_address() {
        let sink = address!("0x1234567890123456789012345678901234567890");
        let result = AndeMevRedirect::try_new(sink, U256::from(1000));
        assert!(result.is_ok());
        let redirect = result.unwrap();
        assert_eq!(redirect.mev_sink(), sink);
    }

    #[test]
    fn test_with_default_threshold_validates() {
        let sink = address!("0x1234567890123456789012345678901234567890");
        let redirect = AndeMevRedirect::with_default_threshold(sink);
        assert_eq!(redirect.mev_sink(), sink);
        assert_eq!(redirect.min_threshold(), AndeMevRedirect::DEFAULT_MIN_MEV_THRESHOLD);
    }

    fn setup_context(base_fee: u64, sink: Address) -> TestContext {
        let mut ctx: TestContext = Context::new(EmptyDB::default(), SpecId::CANCUN);
        ctx.block.basefee = base_fee;
        ctx.cfg.spec = SpecId::CANCUN;

        // Load sink account
        ctx.journal_mut().load_account(sink).unwrap();

        ctx
    }
}
