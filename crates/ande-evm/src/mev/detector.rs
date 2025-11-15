//! MEV Detection Module
//!
//! Detects MEV opportunities in transaction flows including:
//! - Arbitrage opportunities
//! - Sandwich attacks
//! - Liquidations
//! - Front-running opportunities

use alloy_primitives::{Address, U256, B256};
use alloy_consensus::Transaction;
use alloy_consensus::transaction::SignerRecoverable;
use reth_primitives::TransactionSigned;
use std::collections::{HashMap, HashSet};
use tracing::{debug, info};

/// Type of MEV opportunity detected
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MevType {
    /// Arbitrage opportunity between DEXes
    Arbitrage,
    /// Sandwich attack (front-run + back-run)
    Sandwich,
    /// Liquidation opportunity
    Liquidation,
    /// Front-running opportunity
    FrontRun,
    /// Back-running opportunity
    BackRun,
    /// JIT (Just-In-Time) liquidity
    JitLiquidity,
    /// Other MEV type
    Other,
}

impl MevType {
    /// Get human-readable name
    pub fn name(&self) -> &'static str {
        match self {
            MevType::Arbitrage => "Arbitrage",
            MevType::Sandwich => "Sandwich",
            MevType::Liquidation => "Liquidation",
            MevType::FrontRun => "Front-Run",
            MevType::BackRun => "Back-Run",
            MevType::JitLiquidity => "JIT Liquidity",
            MevType::Other => "Other",
        }
    }
}

/// Detected MEV opportunity
#[derive(Debug, Clone)]
pub struct MevOpportunity {
    /// Type of MEV
    pub mev_type: MevType,
    /// Transaction hash
    pub tx_hash: B256,
    /// Estimated MEV value (in wei)
    pub value: U256,
    /// Addresses involved
    pub addresses: Vec<Address>,
    /// Block number where detected
    pub block_number: u64,
    /// Additional metadata
    pub metadata: HashMap<String, String>,
}

impl MevOpportunity {
    /// Create new MEV opportunity
    pub fn new(mev_type: MevType, tx_hash: B256, value: U256, block_number: u64) -> Self {
        Self {
            mev_type,
            tx_hash,
            value,
            addresses: Vec::new(),
            block_number,
            metadata: HashMap::new(),
        }
    }
    
    /// Add address to opportunity
    pub fn add_address(&mut self, address: Address) {
        self.addresses.push(address);
    }
    
    /// Add metadata
    pub fn add_metadata(&mut self, key: String, value: String) {
        self.metadata.insert(key, value);
    }
}

/// MEV Detector configuration
#[derive(Debug, Clone)]
pub struct DetectorConfig {
    /// Enable arbitrage detection
    pub detect_arbitrage: bool,
    /// Enable sandwich detection
    pub detect_sandwich: bool,
    /// Enable liquidation detection
    pub detect_liquidation: bool,
    /// Minimum value to report (in wei)
    pub min_value: U256,
    /// Known DEX router addresses
    pub dex_routers: HashSet<Address>,
    /// Known lending protocol addresses
    pub lending_protocols: HashSet<Address>,
}

impl Default for DetectorConfig {
    fn default() -> Self {
        Self {
            detect_arbitrage: true,
            detect_sandwich: true,
            detect_liquidation: true,
            min_value: U256::from(100_000_000_000_000_000u64), // 0.1 ANDE
            dex_routers: HashSet::new(),
            lending_protocols: HashSet::new(),
        }
    }
}

/// MEV Detector
#[derive(Debug)]
pub struct MevDetector {
    /// Configuration
    config: DetectorConfig,
    /// Recent transactions for pattern matching
    recent_txs: Vec<(B256, TransactionInfo)>,
    /// Maximum number of recent transactions to track
    max_recent_txs: usize,
}

/// Transaction information for MEV detection
#[derive(Debug, Clone)]
struct TransactionInfo {
    /// Transaction hash
    hash: B256,
    /// From address
    from: Address,
    /// To address (if any)
    to: Option<Address>,
    /// Value transferred
    value: U256,
    /// Gas price
    gas_price: U256,
    /// Block number
    block_number: u64,
}

impl MevDetector {
    /// Create new MEV detector
    pub fn new(config: DetectorConfig) -> Self {
        Self {
            config,
            recent_txs: Vec::new(),
            max_recent_txs: 1000,
        }
    }
    
    /// Create detector with default configuration
    pub fn default() -> Self {
        Self::new(DetectorConfig::default())
    }
    
    /// Analyze a transaction for MEV opportunities
    pub fn analyze_transaction(
        &mut self,
        tx: &TransactionSigned,
        block_number: u64,
    ) -> Vec<MevOpportunity> {
        let mut opportunities = Vec::new();
        
        // Extract transaction info
        let tx_info = self.extract_tx_info(tx, block_number);
        
        // Detect different types of MEV
        if self.config.detect_arbitrage {
            if let Some(opp) = self.detect_arbitrage(&tx_info) {
                opportunities.push(opp);
            }
        }
        
        if self.config.detect_sandwich {
            if let Some(opp) = self.detect_sandwich(&tx_info) {
                opportunities.push(opp);
            }
        }
        
        if self.config.detect_liquidation {
            if let Some(opp) = self.detect_liquidation(&tx_info) {
                opportunities.push(opp);
            }
        }
        
        // Store transaction for future pattern matching
        self.add_recent_tx(tx_info);
        
        // Filter by minimum value
        opportunities.retain(|opp| opp.value >= self.config.min_value);
        
        // Log detected opportunities
        for opp in &opportunities {
            info!(
                "MEV detected: type={}, value={}, tx={}",
                opp.mev_type.name(),
                opp.value,
                opp.tx_hash
            );
        }
        
        opportunities
    }
    
    /// Analyze a block of transactions
    pub fn analyze_block(
        &mut self,
        transactions: &[TransactionSigned],
        block_number: u64,
    ) -> Vec<MevOpportunity> {
        let mut all_opportunities = Vec::new();
        
        for tx in transactions {
            let opportunities = self.analyze_transaction(tx, block_number);
            all_opportunities.extend(opportunities);
        }
        
        // Look for cross-transaction patterns (e.g., sandwich attacks)
        let cross_tx_opportunities = self.detect_cross_transaction_mev(transactions, block_number);
        all_opportunities.extend(cross_tx_opportunities);
        
        all_opportunities
    }
    
    /// Extract transaction information
    fn extract_tx_info(&self, tx: &TransactionSigned, block_number: u64) -> TransactionInfo {
        let from = tx.recover_signer().unwrap_or_default();
        let to = tx.to();
        let value = tx.value();
        
        // Get gas price (handle different transaction types)
        let gas_price = {
            let max_fee = tx.max_fee_per_gas();
            if max_fee > 0 {
                U256::from(max_fee)
            } else if let Some(gp) = tx.gas_price() {
                U256::from(gp)
            } else {
                U256::ZERO
            }
        };
        
        TransactionInfo {
            hash: *tx.hash(),
            from,
            to,
            value,
            gas_price,
            block_number,
        }
    }
    
    /// Detect arbitrage opportunities
    fn detect_arbitrage(&self, tx_info: &TransactionInfo) -> Option<MevOpportunity> {
        // Check if transaction interacts with DEX routers
        if let Some(to) = tx_info.to {
            if self.config.dex_routers.contains(&to) {
                // Heuristic: High gas price + interaction with DEX = possible arbitrage
                let base_gas_price = U256::from(50_000_000_000u64); // 50 gwei
                
                if tx_info.gas_price > base_gas_price * U256::from(2) {
                    // Estimate MEV value based on gas price premium
                    let gas_premium = tx_info.gas_price - base_gas_price;
                    let estimated_value = gas_premium * U256::from(100_000); // Rough estimate
                    
                    let mut opp = MevOpportunity::new(
                        MevType::Arbitrage,
                        tx_info.hash,
                        estimated_value,
                        tx_info.block_number,
                    );
                    opp.add_address(to);
                    opp.add_address(tx_info.from);
                    
                    debug!("Potential arbitrage detected: tx={}", tx_info.hash);
                    return Some(opp);
                }
            }
        }
        
        None
    }
    
    /// Detect sandwich attacks
    fn detect_sandwich(&self, tx_info: &TransactionInfo) -> Option<MevOpportunity> {
        // Look for pattern: high gas price + similar recent transaction
        // This is a simplified heuristic
        
        let base_gas_price = U256::from(50_000_000_000u64);
        if tx_info.gas_price > base_gas_price * U256::from(3) {
            // Check if there's a similar pending transaction (victim)
            for (_, recent_tx) in &self.recent_txs {
                if recent_tx.to == tx_info.to 
                    && recent_tx.from != tx_info.from
                    && recent_tx.block_number == tx_info.block_number {
                    
                    // Potential sandwich attack detected
                    let estimated_value = tx_info.gas_price * U256::from(50_000);
                    
                    let mut opp = MevOpportunity::new(
                        MevType::Sandwich,
                        tx_info.hash,
                        estimated_value,
                        tx_info.block_number,
                    );
                    opp.add_address(tx_info.from);
                    if let Some(to) = tx_info.to {
                        opp.add_address(to);
                    }
                    
                    debug!("Potential sandwich attack detected: tx={}", tx_info.hash);
                    return Some(opp);
                }
            }
        }
        
        None
    }
    
    /// Detect liquidation opportunities
    fn detect_liquidation(&self, tx_info: &TransactionInfo) -> Option<MevOpportunity> {
        // Check if transaction interacts with lending protocols
        if let Some(to) = tx_info.to {
            if self.config.lending_protocols.contains(&to) {
                // Heuristic: High value + lending protocol = possible liquidation
                let min_liquidation_value = U256::from(1_000_000_000_000_000_000u64); // 1 ANDE
                
                if tx_info.value > min_liquidation_value {
                    let estimated_value = tx_info.value / U256::from(10); // 10% estimate
                    
                    let mut opp = MevOpportunity::new(
                        MevType::Liquidation,
                        tx_info.hash,
                        estimated_value,
                        tx_info.block_number,
                    );
                    opp.add_address(to);
                    opp.add_address(tx_info.from);
                    
                    debug!("Potential liquidation detected: tx={}", tx_info.hash);
                    return Some(opp);
                }
            }
        }
        
        None
    }
    
    /// Detect cross-transaction MEV patterns
    fn detect_cross_transaction_mev(
        &self,
        transactions: &[TransactionSigned],
        block_number: u64,
    ) -> Vec<MevOpportunity> {
        let mut opportunities = Vec::new();
        
        // Look for sandwich patterns: front-run + victim + back-run
        if transactions.len() >= 3 {
            for i in 0..transactions.len() - 2 {
                let tx1 = &transactions[i];
                let tx2 = &transactions[i + 1];
                let tx3 = &transactions[i + 2];
                
                // Extract signers
                let signer1 = tx1.recover_signer().unwrap_or_default();
                let signer2 = tx2.recover_signer().unwrap_or_default();
                let signer3 = tx3.recover_signer().unwrap_or_default();
                
                // Check if tx1 and tx3 are from same address (potential sandwich)
                if signer1 == signer3 && signer1 != signer2 && tx1.to() == tx3.to() {
                    // Estimate MEV value from gas price difference
                    let gas_price1 = tx1.gas_price().unwrap_or(0);
                    let gas_price2 = tx2.gas_price().unwrap_or(0);
                    
                    if gas_price1 > gas_price2 {
                        let gas_diff = U256::from(gas_price1 - gas_price2);
                        let estimated_value = gas_diff * U256::from(100_000);
                        
                        let mut opp = MevOpportunity::new(
                            MevType::Sandwich,
                            *tx2.hash(),
                            estimated_value,
                            block_number,
                        );
                        opp.add_address(signer1);
                        opp.add_address(signer2);
                        opp.add_metadata("sandwich_type".to_string(), "detected".to_string());
                        opp.add_metadata("front_run_tx".to_string(), format!("{:?}", *tx1.hash()));
                        opp.add_metadata("back_run_tx".to_string(), format!("{:?}", *tx3.hash()));
                        
                        opportunities.push(opp);
                    }
                }
            }
        }
        
        opportunities
    }
    
    /// Add transaction to recent history
    fn add_recent_tx(&mut self, tx_info: TransactionInfo) {
        self.recent_txs.push((tx_info.hash, tx_info));
        
        // Keep only recent transactions
        if self.recent_txs.len() > self.max_recent_txs {
            self.recent_txs.remove(0);
        }
    }
    
    /// Clear old transactions from history
    pub fn cleanup(&mut self, current_block: u64, blocks_to_keep: u64) {
        let cutoff_block = current_block.saturating_sub(blocks_to_keep);
        self.recent_txs.retain(|(_, tx)| tx.block_number >= cutoff_block);
    }
    
    /// Add DEX router address
    pub fn add_dex_router(&mut self, address: Address) {
        self.config.dex_routers.insert(address);
    }
    
    /// Add lending protocol address
    pub fn add_lending_protocol(&mut self, address: Address) {
        self.config.lending_protocols.insert(address);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mev_type_name() {
        assert_eq!(MevType::Arbitrage.name(), "Arbitrage");
        assert_eq!(MevType::Sandwich.name(), "Sandwich");
        assert_eq!(MevType::Liquidation.name(), "Liquidation");
    }

    #[test]
    fn test_mev_opportunity_creation() {
        let hash = B256::random();
        let value = U256::from(1000);
        let mut opp = MevOpportunity::new(MevType::Arbitrage, hash, value, 100);
        
        assert_eq!(opp.mev_type, MevType::Arbitrage);
        assert_eq!(opp.value, value);
        assert_eq!(opp.block_number, 100);
        
        let addr = Address::random();
        opp.add_address(addr);
        assert_eq!(opp.addresses.len(), 1);
        assert_eq!(opp.addresses[0], addr);
    }

    #[test]
    fn test_detector_creation() {
        let config = DetectorConfig::default();
        let detector = MevDetector::new(config);
        
        assert_eq!(detector.recent_txs.len(), 0);
        assert_eq!(detector.max_recent_txs, 1000);
    }

    #[test]
    fn test_detector_config() {
        let mut config = DetectorConfig::default();
        assert!(config.detect_arbitrage);
        assert!(config.detect_sandwich);
        assert!(config.detect_liquidation);
        
        let dex_addr = Address::random();
        config.dex_routers.insert(dex_addr);
        assert!(config.dex_routers.contains(&dex_addr));
    }
}
