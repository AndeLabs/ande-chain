//! Ethereum contract client for consensus on-chain interactions

use crate::{
    error::{ConsensusError, Result},
    types::ValidatorInfo,
};
use alloy_primitives::{Address, B256};
use ethers::{
    contract::abigen,
    providers::{Http, Middleware, Provider, Ws},
    types::H160,
};
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, error, info};

// Generate contract bindings
abigen!(
    AndeConsensus,
    r#"[
        function getActiveValidators() external view returns (address[])
        function getValidatorInfo(address) external view returns (tuple(address,bytes32,string,uint256,uint256,int256,uint256,uint256,uint256,uint256,uint256,bool,bool,bool))
        function getCurrentProposer() external view returns (address)
        function isValidator(address) external view returns (bool)
        function currentEpoch() external view returns (uint256)
        function currentBlockNumber() external view returns (uint256)
        function totalVotingPower() external view returns (uint256)
        event ValidatorSetUpdated(uint256 indexed epoch, address[] validators, uint256[] powers, uint256 totalPower)
        event BlockProposed(uint256 indexed blockNumber, bytes32 indexed blockHash, address indexed producer, uint256 timestamp)
        event BlockFinalized(uint256 indexed blockNumber, bytes32 indexed blockHash, uint256 totalPower, uint256 threshold)
    ]"#
);

abigen!(
    AndeSequencerCoordinator,
    r#"[
        function currentLeader() external view returns (address)
        function getActiveSequencers() external view returns (address[])
        function isTimeoutReached() external view returns (bool)
        function checkTimeout() external
        function recordBlockProduced(address,uint256,uint256) external
        event LeaderRotated(uint256 indexed rotationNumber, address indexed oldLeader, address indexed newLeader, uint256 blockNumber, string reason)
        event TimeoutDetected(address indexed sequencer, uint256 missedBlocks, uint256 slashedAmount)
    ]"#
);

/// Client for interacting with consensus contracts
pub struct ContractClient {
    /// HTTP provider for queries
    provider: Arc<Provider<Http>>,

    /// WebSocket provider for events
    ws_provider: Option<Arc<Provider<Ws>>>,

    /// AndeConsensus contract
    consensus: AndeConsensus<Provider<Http>>,

    /// AndeSequencerCoordinator contract
    coordinator: AndeSequencerCoordinator<Provider<Http>>,

    /// Last synced block number
    last_synced_block: Arc<RwLock<u64>>,
}

impl ContractClient {
    /// Create new contract client
    ///
    /// # Errors
    ///
    /// Returns error if provider connection fails
    pub async fn new(
        rpc_url: &str,
        ws_url: Option<&str>,
        consensus_address: Address,
        coordinator_address: Address,
    ) -> Result<Self> {
        // Connect HTTP provider
        let provider = Provider::<Http>::try_from(rpc_url)
            .map_err(|e| ConsensusError::RpcError(e.to_string()))?;
        let provider = Arc::new(provider);

        // Connect WebSocket provider for events (optional)
        let ws_provider = if let Some(url) = ws_url {
            match Provider::<Ws>::connect(url).await {
                Ok(ws) => Some(Arc::new(ws)),
                Err(e) => {
                    error!(error = %e, "Failed to connect WebSocket provider, events disabled");
                    None
                }
            }
        } else {
            None
        };

        // Create contract instances
        let consensus_addr: H160 = H160::from_slice(consensus_address.as_slice());
        let coordinator_addr: H160 = H160::from_slice(coordinator_address.as_slice());

        let consensus = AndeConsensus::new(consensus_addr, provider.clone());
        let coordinator = AndeSequencerCoordinator::new(coordinator_addr, provider.clone());

        info!(
            consensus = ?consensus_address,
            coordinator = ?coordinator_address,
            "Contract client initialized"
        );

        Ok(Self {
            provider,
            ws_provider,
            consensus,
            coordinator,
            last_synced_block: Arc::new(RwLock::new(0)),
        })
    }

    /// Get list of active validator addresses
    pub async fn get_active_validators(&self) -> Result<Vec<Address>> {
        let addresses = self
            .consensus
            .get_active_validators()
            .call()
            .await
            .map_err(|e| ConsensusError::ContractError(e.to_string()))?;

        let validators: Vec<Address> = addresses
            .into_iter()
            .map(|addr| Address::from_slice(addr.as_bytes()))
            .collect();

        debug!(count = validators.len(), "Fetched active validators");
        Ok(validators)
    }

    /// Get detailed validator information
    pub async fn get_validator_info(&self, address: Address) -> Result<ValidatorInfo> {
        let addr: H160 = H160::from_slice(address.as_slice());

        let info = self
            .consensus
            .get_validator_info(addr)
            .call()
            .await
            .map_err(|e| ConsensusError::ContractError(e.to_string()))?;

        // Convert from contract tuple to ValidatorInfo
        // Note: Solidity returns I256 for accumulated_priority, convert to i64
        let accumulated_priority = if info.5 >= ethers::types::I256::zero() {
            info.5.as_u128() as i64
        } else {
            -((-info.5).as_u128() as i64)
        };

        Ok(ValidatorInfo {
            validator: Address::from_slice(info.0.as_bytes()),
            p2p_peer_id: B256::from_slice(&info.1),
            rpc_endpoint: info.2,
            stake: alloy_primitives::U256::from_limbs(info.3.0),
            power: info.4.as_u64(),
            accumulated_priority,
            total_blocks_produced: info.6.as_u64(),
            total_blocks_missed: info.7.as_u64(),
            uptime: info.8.as_u64() as u16,
            last_block_produced: info.9.as_u64(),
            registered_at: info.10.as_u64(),
            jailed: info.11,
            active: info.12,
            is_permanent: info.13,
        })
    }

    /// Get all active validators with full info
    pub async fn get_all_validators_info(&self) -> Result<Vec<ValidatorInfo>> {
        let addresses = self.get_active_validators().await?;

        let mut validators = Vec::with_capacity(addresses.len());
        for address in addresses {
            match self.get_validator_info(address).await {
                Ok(info) => validators.push(info),
                Err(e) => {
                    error!(validator = ?address, error = %e, "Failed to fetch validator info");
                }
            }
        }

        Ok(validators)
    }

    /// Get current proposer address
    pub async fn get_current_proposer(&self) -> Result<Address> {
        let proposer = self
            .consensus
            .get_current_proposer()
            .call()
            .await
            .map_err(|e| ConsensusError::ContractError(e.to_string()))?;

        Ok(Address::from_slice(proposer.as_bytes()))
    }

    /// Check if address is a validator
    pub async fn is_validator(&self, address: Address) -> Result<bool> {
        let addr: H160 = H160::from_slice(address.as_slice());

        let is_val = self
            .consensus
            .is_validator(addr)
            .call()
            .await
            .map_err(|e| ConsensusError::ContractError(e.to_string()))?;

        Ok(is_val)
    }

    /// Get current epoch number
    pub async fn get_current_epoch(&self) -> Result<u64> {
        let epoch = self
            .consensus
            .current_epoch()
            .call()
            .await
            .map_err(|e| ConsensusError::ContractError(e.to_string()))?;

        Ok(epoch.as_u64())
    }

    /// Get total voting power
    pub async fn get_total_voting_power(&self) -> Result<u64> {
        let power = self
            .consensus
            .total_voting_power()
            .call()
            .await
            .map_err(|e| ConsensusError::ContractError(e.to_string()))?;

        Ok(power.as_u64())
    }

    /// Get current leader from sequencer coordinator
    pub async fn get_current_leader(&self) -> Result<Address> {
        let leader = self
            .coordinator
            .current_leader()
            .call()
            .await
            .map_err(|e| ConsensusError::ContractError(e.to_string()))?;

        Ok(Address::from_slice(leader.as_bytes()))
    }

    /// Check if timeout is reached for current leader
    pub async fn is_timeout_reached(&self) -> Result<bool> {
        let timeout = self
            .coordinator
            .is_timeout_reached()
            .call()
            .await
            .map_err(|e| ConsensusError::ContractError(e.to_string()))?;

        Ok(timeout)
    }

    /// Get current block number from provider
    pub async fn get_block_number(&self) -> Result<u64> {
        let block = self
            .provider
            .get_block_number()
            .await
            .map_err(|e| ConsensusError::RpcError(e.to_string()))?;

        Ok(block.as_u64())
    }

    /// Subscribe to ValidatorSetUpdated events
    ///
    /// # Errors
    ///
    /// Returns error if WebSocket provider is not available
    ///
    /// TODO: Implement event subscription with proper type annotations
    /// Currently disabled pending ethers-rs type compatibility fixes
    #[allow(dead_code)]
    pub async fn subscribe_validator_set_updates(
        &self,
    ) -> Result<()> {
        Err(ConsensusError::Internal(
            "Event subscription not yet implemented".to_string(),
        ))
    }

    /// Get the last synced block number
    pub async fn last_synced_block(&self) -> u64 {
        *self.last_synced_block.read().await
    }

    /// Update the last synced block number
    pub async fn update_last_synced_block(&self, block: u64) {
        *self.last_synced_block.write().await = block;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    #[ignore] // Requires running node
    async fn test_contract_client_creation() {
        let client = ContractClient::new(
            "http://localhost:8545",
            None,
            Address::ZERO,
            Address::ZERO,
        )
        .await;

        // Should succeed even with zero addresses (contracts might not exist)
        assert!(client.is_ok());
    }
}
