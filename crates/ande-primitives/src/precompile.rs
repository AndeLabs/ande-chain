//! Precompile definitions

/// Token duality precompile functions
pub mod token_duality {
    
    /// Function selector for balance_of
    pub const BALANCE_OF_SELECTOR: [u8; 4] = [0x70, 0xa0, 0x82, 0x31];
    
    /// Function selector for transfer
    pub const TRANSFER_SELECTOR: [u8; 4] = [0xa9, 0x05, 0x9c, 0xbb];
}
