//! ANDE Token Duality Precompile
//!
//! Production-grade precompile based on evstack's MintPrecompile with enhanced security.
//!
//! ## Features
//! - ✅ Direct balance manipulation via EvmInternals
//! - ✅ Admin-based authorization
//! - ✅ Storage-based allowlist
//! - ✅ Per-call transfer caps
//! - ✅ Per-block transfer caps with automatic tracking
//! - ✅ Environment-based configuration
//! - ✅ ABI interface for clean integration
//!
//! ## Architecture
//! Based on evstack/ev-reth MintPrecompile pattern with ANDE-specific enhancements.

use alloy_evm::{
    precompiles::{Precompile, PrecompileInput},
    revm::precompile::{PrecompileError, PrecompileId, PrecompileResult},
    EvmInternals, EvmInternalsError,
};
use alloy_primitives::{address, Address, Bytes, U256};
use revm::{bytecode::Bytecode, precompile::PrecompileOutput};
use std::sync::{Arc, OnceLock, RwLock};

/// Function selectors for ANDE Token Duality interface
pub mod selectors {
    /// transfer(address,address,uint256) - 0xbeabacc8
    pub const TRANSFER: [u8; 4] = [0xbe, 0xab, 0xac, 0xc8];
    /// addToAllowList(address) - 0xe43252d7
    pub const ADD_TO_ALLOWLIST: [u8; 4] = [0xe4, 0x32, 0x52, 0xd7];
    /// removeFromAllowList(address) - 0xb8 0x2b 0x84 0x2f
    pub const REMOVE_FROM_ALLOWLIST: [u8; 4] = [0xb8, 0x2b, 0x84, 0x2f];
    /// allowlist(address) - 0x43 0xd7 0x26 0xd6
    pub const ALLOWLIST: [u8; 4] = [0x43, 0xd7, 0x26, 0xd6];
    /// transferredThisBlock() - 0x1c 0x4e 0x59 0x2f
    pub const TRANSFERRED_THIS_BLOCK: [u8; 4] = [0x1c, 0x4e, 0x59, 0x2f];
}

/// ANDE Token Duality Precompile Address: 0x00..fd
pub const ANDE_PRECOMPILE_ADDRESS: Address = address!("00000000000000000000000000000000000000fd");

/// Address of the ANDEToken contract authorized to call this precompile
/// Configured via genesis or environment variable ANDE_TOKEN_ADDRESS
pub const ANDE_TOKEN_ADDRESS: Address = Address::ZERO;

/// Default per-call cap: 1 million ANDE (with 18 decimals)
const DEFAULT_PER_CALL_CAP: u128 = 1_000_000;

/// Default per-block cap: 10 million ANDE (with 18 decimals)
const DEFAULT_PER_BLOCK_CAP: u128 = 10_000_000;

/// Configuration for ANDE Token Duality Precompile
#[derive(Clone, Debug)]
pub struct AndePrecompileConfig {
    /// Admin address (can manage allowlist)
    pub admin: Address,
    
    /// Maximum amount per single transfer
    pub per_call_cap: U256,
    
    /// Maximum total amount per block
    pub per_block_cap: U256,
    
    /// Enable strict validation
    pub strict_validation: bool,
}

impl Default for AndePrecompileConfig {
    fn default() -> Self {
        Self {
            admin: Address::ZERO,
            per_call_cap: U256::from(DEFAULT_PER_CALL_CAP) * U256::from(10u64).pow(U256::from(18)),
            per_block_cap: U256::from(DEFAULT_PER_BLOCK_CAP) * U256::from(10u64).pow(U256::from(18)),
            strict_validation: true,
        }
    }
}

impl AndePrecompileConfig {
    /// Load configuration from environment variables
    pub fn from_env() -> Self {
        let mut config = Self::default();
        
        if let Ok(admin) = std::env::var("ANDE_ADMIN") {
            if let Ok(addr) = admin.parse() {
                config.admin = addr;
            }
        }
        
        if let Ok(cap) = std::env::var("ANDE_PER_CALL_CAP") {
            if let Ok(value) = cap.parse::<u64>() {
                config.per_call_cap = U256::from(value);
            }
        }
        
        if let Ok(cap) = std::env::var("ANDE_PER_BLOCK_CAP") {
            if let Ok(value) = cap.parse::<u64>() {
                config.per_block_cap = U256::from(value);
            }
        }
        
        config
    }
}

/// Per-block transfer tracking
#[derive(Clone, Debug, Default)]
struct BlockTransferTracker {
    block_number: u64,
    total_transferred: U256,
}

/// ANDE Token Duality Precompile
///
/// Allows ANDE tokens to function as both native gas token and ERC-20-like token
#[derive(Clone, Debug)]
pub struct AndeTokenDualityPrecompile {
    config: AndePrecompileConfig,
    block_tracker: Arc<RwLock<BlockTransferTracker>>,
}

impl AndeTokenDualityPrecompile {
    /// Lazily-initialized precompile ID
    pub fn id() -> &'static PrecompileId {
        static ID: OnceLock<PrecompileId> = OnceLock::new();
        ID.get_or_init(|| PrecompileId::custom("ande_token_duality"))
    }
    
    /// Bytecode marker for the precompile account
    fn bytecode() -> &'static Bytecode {
        static BYTECODE: OnceLock<Bytecode> = OnceLock::new();
        BYTECODE.get_or_init(|| Bytecode::new_raw(Bytes::from_static(&[0xFE])))
    }
    
    /// Create new precompile with config
    pub fn new(config: AndePrecompileConfig) -> Self {
        Self {
            config,
            block_tracker: Arc::new(RwLock::new(BlockTransferTracker::default())),
        }
    }
    
    /// Create from environment variables
    pub fn from_env() -> Self {
        Self::new(AndePrecompileConfig::from_env())
    }
    
    // === Helper functions from evstack MintPrecompile ===
    
    fn map_internals_error(err: EvmInternalsError) -> PrecompileError {
        PrecompileError::Other(err.to_string())
    }
    
    fn ensure_account_created(
        internals: &mut EvmInternals<'_>,
        addr: Address,
    ) -> Result<(), PrecompileError> {
        let mut account = internals
            .load_account(addr)
            .map_err(Self::map_internals_error)?;
        
        if account.is_loaded_as_not_existing() {
            if addr == ANDE_PRECOMPILE_ADDRESS {
                account.info.set_code(Self::bytecode().clone());
                account.info.set_nonce(1);
            }
            account.mark_created();
            internals.touch_account(addr);
        }
        
        Ok(())
    }
    
    fn add_balance(
        internals: &mut EvmInternals<'_>,
        addr: Address,
        amount: U256,
    ) -> Result<(), PrecompileError> {
        let mut account = internals
            .load_account(addr)
            .map_err(Self::map_internals_error)?;
        let new_balance = account
            .info
            .balance
            .checked_add(amount)
            .ok_or_else(|| PrecompileError::Other("balance overflow".to_string()))?;
        account.info.set_balance(new_balance);
        Ok(())
    }
    
    fn sub_balance(
        internals: &mut EvmInternals<'_>,
        addr: Address,
        amount: U256,
    ) -> Result<(), PrecompileError> {
        let mut account = internals
            .load_account(addr)
            .map_err(Self::map_internals_error)?;
        let new_balance = account
            .info
            .balance
            .checked_sub(amount)
            .ok_or_else(|| PrecompileError::Other("insufficient balance".to_string()))?;
        account.info.set_balance(new_balance);
        Ok(())
    }
    
    // === Authorization ===
    
    fn ensure_admin(&self, caller: Address) -> Result<(), PrecompileError> {
        if caller == self.config.admin {
            Ok(())
        } else {
            Err(PrecompileError::Other("unauthorized: not admin".to_string()))
        }
    }
    
    fn ensure_authorized(
        &self,
        internals: &mut EvmInternals<'_>,
        caller: Address,
    ) -> Result<(), PrecompileError> {
        if caller == self.config.admin {
            tracing::debug!(target: "ande_precompile", ?caller, "✅ authorized: admin");
            return Ok(());
        }
        
        let allowlisted = Self::is_allowlisted(internals, caller)?;
        if allowlisted {
            tracing::debug!(target: "ande_precompile", ?caller, "✅ authorized: allowlist");
            Ok(())
        } else {
            tracing::warn!(target: "ande_precompile", ?caller, "❌ unauthorized");
            Err(PrecompileError::Other("unauthorized: not in allowlist".to_string()))
        }
    }
    
    // === Allowlist storage ===
    
    fn is_allowlisted(
        internals: &mut EvmInternals<'_>,
        addr: Address,
    ) -> Result<bool, PrecompileError> {
        Self::ensure_account_created(internals, ANDE_PRECOMPILE_ADDRESS)?;
        let key = Self::allowlist_key(addr);
        let value = internals
            .sload(ANDE_PRECOMPILE_ADDRESS, key)
            .map_err(Self::map_internals_error)?;
        Ok(!value.is_zero())
    }
    
    fn set_allowlisted(
        internals: &mut EvmInternals<'_>,
        addr: Address,
        allowed: bool,
    ) -> Result<(), PrecompileError> {
        Self::ensure_account_created(internals, ANDE_PRECOMPILE_ADDRESS)?;
        let value = if allowed { U256::from(1) } else { U256::ZERO };
        internals
            .sstore(ANDE_PRECOMPILE_ADDRESS, Self::allowlist_key(addr), value)
            .map_err(Self::map_internals_error)?;
        internals.touch_account(ANDE_PRECOMPILE_ADDRESS);
        Ok(())
    }
    
    fn allowlist_key(addr: Address) -> U256 {
        U256::from_be_bytes(addr.into_word().into())
    }
    
    // === Transfer caps validation ===
    
    fn validate_transfer_caps(
        &self,
        amount: U256,
        block_number: u64,
    ) -> Result<(), PrecompileError> {
        // Per-call cap
        if amount > self.config.per_call_cap {
            return Err(PrecompileError::Other(format!(
                "transfer exceeds per-call cap: {} > {}",
                amount, self.config.per_call_cap
            )));
        }
        
        // Per-block cap
        let mut tracker = self.block_tracker.write().unwrap();
        
        // Reset if new block
        if tracker.block_number != block_number {
            tracker.block_number = block_number;
            tracker.total_transferred = U256::ZERO;
        }
        
        let new_total = tracker
            .total_transferred
            .checked_add(amount)
            .ok_or_else(|| PrecompileError::Other("block transfer overflow".to_string()))?;
        
        if new_total > self.config.per_block_cap {
            return Err(PrecompileError::Other(format!(
                "transfer exceeds per-block cap: {} > {}",
                new_total, self.config.per_block_cap
            )));
        }
        
        tracker.total_transferred = new_total;
        Ok(())
    }
    
    // === Main transfer logic ===
    
    fn execute_transfer(
        &self,
        internals: &mut EvmInternals<'_>,
        from: Address,
        to: Address,
        amount: U256,
        block_number: u64,
    ) -> Result<(), PrecompileError> {
        // Validate zero address
        if to.is_zero() {
            return Err(PrecompileError::Other("cannot transfer to zero address".to_string()));
        }
        
        // Validate caps
        self.validate_transfer_caps(amount, block_number)?;
        
        // Skip zero transfers
        if amount.is_zero() {
            return Ok(());
        }
        
        tracing::info!(
            target: "ande_precompile",
            ?from, ?to, ?amount, block_number,
            "🔄 executing ANDE transfer"
        );
        
        // Ensure accounts exist
        Self::ensure_account_created(internals, from)?;
        Self::ensure_account_created(internals, to)?;
        
        // Execute native balance transfer
        Self::sub_balance(internals, from, amount)?;
        Self::add_balance(internals, to, amount)?;
        
        // Touch accounts
        internals.touch_account(from);
        internals.touch_account(to);
        
        tracing::info!(target: "ande_precompile", "✅ transfer successful");
        Ok(())
    }
}

impl Precompile for AndeTokenDualityPrecompile {
    fn precompile_id(&self) -> &PrecompileId {
        Self::id()
    }
    
    fn call(&self, mut input: PrecompileInput<'_>) -> PrecompileResult {
        let caller = input.caller;
        let gas_limit = input.gas;
        let data = input.data;
        
        tracing::info!(
            target: "ande_precompile",
            ?caller,
            gas = gas_limit,
            calldata_len = data.len(),
            "📞 ANDE Token Duality precompile called"
        );
        
        // Check minimum length (selector = 4 bytes)
        if data.len() < 4 {
            return Err(PrecompileError::Other("calldata too short".to_string()));
        }
        
        let selector = &data[0..4];
        let internals = input.internals_mut();
        
        // Get block number from block environment
        let block_number = internals.block_number().to::<u64>();
        
        // Dispatch based on selector
        match selector {
            s if s == selectors::TRANSFER => {
                // transfer(address from, address to, uint256 amount)
                if data.len() != 100 { // 4 + 32 + 32 + 32
                    return Err(PrecompileError::Other("invalid calldata length for transfer".to_string()));
                }
                
                self.ensure_authorized(internals, caller)?;
                
                let from = Address::from_slice(&data[16..36]); // skip padding
                let to = Address::from_slice(&data[48..68]);
                let amount = U256::from_be_slice(&data[68..100]);
                
                self.execute_transfer(internals, from, to, amount, block_number)?;
                Ok(PrecompileOutput::new(0, Bytes::new()))
            }
            s if s == selectors::ADD_TO_ALLOWLIST => {
                // addToAllowList(address account)
                if data.len() != 36 { // 4 + 32
                    return Err(PrecompileError::Other("invalid calldata length for addToAllowList".to_string()));
                }
                
                self.ensure_admin(caller)?;
                let account = Address::from_slice(&data[16..36]);
                Self::set_allowlisted(internals, account, true)?;
                
                tracing::info!(target: "ande_precompile", ?account, "✅ added to allowlist");
                Ok(PrecompileOutput::new(0, Bytes::new()))
            }
            s if s == selectors::REMOVE_FROM_ALLOWLIST => {
                // removeFromAllowList(address account)
                if data.len() != 36 { // 4 + 32
                    return Err(PrecompileError::Other("invalid calldata length for removeFromAllowList".to_string()));
                }
                
                self.ensure_admin(caller)?;
                let account = Address::from_slice(&data[16..36]);
                Self::set_allowlisted(internals, account, false)?;
                
                tracing::info!(target: "ande_precompile", ?account, "✅ removed from allowlist");
                Ok(PrecompileOutput::new(0, Bytes::new()))
            }
            s if s == selectors::ALLOWLIST => {
                // allowlist(address account) returns (bool)
                if data.len() != 36 { // 4 + 32
                    return Err(PrecompileError::Other("invalid calldata length for allowlist".to_string()));
                }
                
                let account = Address::from_slice(&data[16..36]);
                let is_allowed = Self::is_allowlisted(internals, account)?;
                
                // Encode bool as uint256 (0 or 1)
                let mut result = vec![0u8; 32];
                if is_allowed {
                    result[31] = 1;
                }
                
                Ok(PrecompileOutput::new(0, Bytes::from(result)))
            }
            s if s == selectors::TRANSFERRED_THIS_BLOCK => {
                // transferredThisBlock() returns (uint256)
                let tracker = self.block_tracker.read().unwrap();
                let result = tracker.total_transferred.to_be_bytes::<32>();
                Ok(PrecompileOutput::new(0, Bytes::copy_from_slice(&result)))
            }
            _ => {
                tracing::warn!(target: "ande_precompile", selector = ?selector, "❌ unknown function selector");
                Err(PrecompileError::Other("unknown function selector".to_string()))
            }
        }
    }
    
    fn is_pure(&self) -> bool {
        false // Stateful precompile
    }
}
