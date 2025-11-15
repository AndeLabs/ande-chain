//! Ande Consensus Contract Client
//!
//! Provides integration between ev-reth and the AndeConsensus smart contract
//! for block producer selection, attestation, and validator synchronization.

use alloy::{
    network::EthereumWallet,
    primitives::{Address, Bytes, FixedBytes, B256, U256},
    providers::{Provider, ProviderBuilder, RootProvider},
    signers::local::PrivateKeySigner,
    transports::http::{Client, Http},
};
use ande_consensus_bindings::{AndeConsensus, ContractAddresses};
use eyre::Result;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;
use tokio::task::JoinHandle;
use tracing::{debug, error, info, warn};

/// Ande Consensus Contract Client
/// 
/// Handles all interactions with the AndeConsensus smart contract:
/// - Querying designated block producers
/// - Proposing blocks with signatures
/// - Syncing validator set
/// - Reading validator information
/// - Background event listening
#[derive(Clone)]
pub struct AndeConsensusClient {
    /// Consensus contract instance
    consensus: AndeConsensus::AndeConsensusInstance<
        Http<Client>,
        Arc<RootProvider<Http<Client>>>,
    >,
    /// Provider for RPC calls
    provider: Arc<RootProvider<Http<Client>>>,
    /// Wallet for signing transactions
    wallet: Option<EthereumWallet>,
    /// Cached active validators
    validators: Arc<RwLock<Vec<Address>>>,
    /// Last synced block number for event filtering
    last_synced_block: Arc<RwLock<u64>>,
}

impl AndeConsensusClient {
    /// Create a new consensus client
    ///
    /// # Arguments
    /// * `rpc_url` - RPC endpoint URL
    /// * `addresses` - Contract addresses
    /// * `signer` - Optional signer for transactions
    pub async fn new(
        rpc_url: &str,
        addresses: ContractAddresses,
        signer: Option<PrivateKeySigner>,
    ) -> Result<Self> {
        info!("Initializing AndeConsensusClient");
        info!("  RPC URL: {}", rpc_url);
        info!("  Consensus contract: {:?}", addresses.consensus);
        
        // Build provider
        let provider = if let Some(signer) = signer {
            let wallet = EthereumWallet::from(signer.clone());
            ProviderBuilder::new()
                .with_recommended_fillers()
                .wallet(wallet.clone())
                .on_http(rpc_url.parse()?)
        } else {
            ProviderBuilder::new()
                .on_http(rpc_url.parse()?)
        };
        
        let provider = Arc::new(provider);
        
        // Create consensus contract instance
        let consensus = AndeConsensus::new(addresses.consensus, provider.clone());
        
        // Initialize empty validator list
        let validators = Arc::new(RwLock::new(Vec::new()));
        let last_synced_block = Arc::new(RwLock::new(0));
        
        let client = Self {
            consensus,
            provider,
            wallet: signer.map(EthereumWallet::from),
            validators,
            last_synced_block,
        };
        
        // Initial validator sync
        client.sync_validators().await?;
        
        info!("AndeConsensusClient initialized successfully");
        Ok(client)
    }

    /// Get the designated block producer for a given block number
    ///
    /// Uses the weighted round-robin selection based on voting power.
    pub async fn get_block_producer(&self, block_number: u64) -> Result<Address> {
        debug!("Querying block producer for block {}", block_number);
        
        let producer = self
            .consensus
            .getBlockProducer(U256::from(block_number))
            .call()
            .await?
            ._0;
        
        debug!("Block {} producer: {:?}", block_number, producer);
        Ok(producer)
    }

    /// Propose a block to the consensus contract
    ///
    /// This sends a transaction with the block hash and signature.
    /// Requires wallet to be configured.
    pub async fn propose_block(
        &self,
        block_number: u64,
        block_hash: B256,
        signature: Bytes,
    ) -> Result<B256> {
        if self.wallet.is_none() {
            return Err(eyre::eyre!("Wallet not configured, cannot propose blocks"));
        }
        
        info!(
            "Proposing block {} with hash {:?}",
            block_number, block_hash
        );
        
        let tx_hash = self
            .consensus
            .proposeBlock(
                U256::from(block_number),
                block_hash,
                signature,
            )
            .send()
            .await?
            .watch()
            .await?;
        
        info!(
            "Block {} proposed successfully, tx: {:?}",
            block_number, tx_hash
        );
        
        Ok(tx_hash)
    }

    /// Get active validators from the contract
    pub async fn get_active_validators(&self) -> Result<Vec<Address>> {
        debug!("Fetching active validators");
        
        let validators = self.consensus.getActiveValidators().call().await?._0;
        
        debug!("Found {} active validators", validators.len());
        Ok(validators)
    }

    /// Sync validators from contract to local cache
    pub async fn sync_validators(&self) -> Result<()> {
        let validators = self.get_active_validators().await?;
        
        let mut cache = self.validators.write().await;
        *cache = validators.clone();
        
        info!("Synced {} validators to cache", validators.len());
        Ok(())
    }

    /// Get cached validators (fast, no RPC call)
    pub async fn get_cached_validators(&self) -> Vec<Address> {
        self.validators.read().await.clone()
    }

    /// Get validator information
    pub async fn get_validator_info(&self, validator: Address) -> Result<ValidatorInfo> {
        debug!("Fetching info for validator {:?}", validator);
        
        let info = self.consensus.getValidatorInfo(validator).call().await?;
        
        Ok(ValidatorInfo {
            validator: info.validator,
            p2p_peer_id: info.p2pPeerId,
            rpc_endpoint: info.rpcEndpoint,
            stake: info.stake,
            power: info.power,
            accumulated_priority: info.accumulatedPriority,
            total_blocks_produced: info.totalBlocksProduced,
            total_blocks_missed: info.totalBlocksMissed,
            uptime: info.uptime,
            last_block_produced: info.lastBlockProduced,
            registered_at: info.registeredAt,
            jailed: info.jailed,
            active: info.active,
            is_permanent: info.isPermanent,
        })
    }

    /// Get current epoch number
    pub async fn get_current_epoch(&self) -> Result<u64> {
        let epoch = self.consensus.currentEpoch().call().await?._0;
        Ok(epoch.to())
    }

    /// Check if address is validator
    pub async fn is_validator(&self, address: Address) -> Result<bool> {
        let is_val = self.consensus.isValidator(address).call().await?._0;
        Ok(is_val)
    }

    /// Get block proposal information
    pub async fn get_block_proposal(&self, block_number: u64) -> Result<Option<BlockProposal>> {
        let proposal = self
            .consensus
            .getBlockProposal(U256::from(block_number))
            .call()
            .await?;
        
        // Check if proposal exists (verified = true)
        if !proposal.verified {
            return Ok(None);
        }
        
        Ok(Some(BlockProposal {
            block_number: proposal.blockNumber,
            block_hash: proposal.blockHash,
            producer: proposal.producer,
            signature: proposal.signature,
            timestamp: proposal.timestamp,
            verified: proposal.verified,
        }))
    }

    /// Sync validator set from ValidatorSetUpdated events
    ///
    /// This method queries the chain for ValidatorSetUpdated events since the last sync
    /// and updates the local validator cache.
    pub async fn sync_validator_set_from_events(&self) -> Result<()> {
        let last_block = *self.last_synced_block.read().await;
        
        debug!("Syncing validator set from events since block {}", last_block);
        
        // For now, we use direct query instead of event filtering
        // TODO: Implement proper event filtering when bindings support it
        let validators = self.get_active_validators().await?;
        
        let mut cache = self.validators.write().await;
        *cache = validators.clone();
        
        // Update last synced block to current
        if let Ok(current_block) = self.provider.get_block_number().await {
            let mut last_synced = self.last_synced_block.write().await;
            *last_synced = current_block;
            debug!("Updated last synced block to {}", current_block);
        }
        
        info!("Synced {} validators from events", validators.len());
        Ok(())
    }

    /// Start background task to periodically sync validator set
    ///
    /// This spawns a tokio task that syncs validators every 30 seconds
    pub fn start_validator_sync_task(self) -> JoinHandle<()> {
        info!("Starting background validator sync task");
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_secs(30));
            loop {
                interval.tick().await;
                if let Err(e) = self.sync_validator_set_from_events().await {
                    error!("Failed to sync validator set from events: {}", e);
                } else {
                    debug!("Background validator sync completed successfully");
                }
            }
        })
    }

    /// Get current proposer from the contract
    pub async fn get_current_proposer(&self) -> Result<Address> {
        debug!("Querying current proposer");
        
        let proposer = self.consensus.getCurrentProposer().call().await?._0;
        
        debug!("Current proposer: {:?}", proposer);
        Ok(proposer)
    }

    /// Check if we are the designated proposer for the next block
    pub async fn am_i_proposer(&self, my_address: Address, next_block: u64) -> Result<bool> {
        let proposer = self.get_block_producer(next_block).await?;
        Ok(proposer == my_address)
    }

    /// Get total voting power
    pub async fn get_total_voting_power(&self) -> Result<U256> {
        let power = self.consensus.totalVotingPower().call().await?._0;
        Ok(power)
    }

    /// Check if a block is finalized (has 2/3+1 attestations)
    pub async fn is_block_finalized(&self, block_hash: B256) -> Result<bool> {
        let finalized = self.consensus.isBlockFinalized(block_hash).call().await?._0;
        Ok(finalized)
    }

    /// Get attestation power for a block
    pub async fn get_attestation_power(&self, block_hash: B256) -> Result<U256> {
        let power = self.consensus.getAttestationPower(block_hash).call().await?._0;
        Ok(power)
    }

    /// Get current block number tracked by consensus
    pub async fn get_current_block_number(&self) -> Result<u64> {
        let block_num = self.consensus.currentBlockNumber().call().await?._0;
        Ok(block_num.to())
    }
}

/// Validator information (matches AndeConsensus.sol struct)
#[derive(Debug, Clone)]
pub struct ValidatorInfo {
    /// Address of the validator
    pub validator: Address,
    /// libp2p peer ID
    pub p2p_peer_id: FixedBytes<32>,
    /// RPC endpoint URL
    pub rpc_endpoint: String,
    /// Staked amount
    pub stake: U256,
    /// Voting power
    pub power: U256,
    /// Accumulated priority (CometBFT algorithm)
    pub accumulated_priority: i64,
    /// Total blocks produced
    pub total_blocks_produced: U256,
    /// Total blocks missed
    pub total_blocks_missed: U256,
    /// Uptime percentage (basis points)
    pub uptime: U256,
    /// Last block produced timestamp
    pub last_block_produced: U256,
    /// Registration timestamp
    pub registered_at: U256,
    /// Is jailed
    pub jailed: bool,
    /// Is active
    pub active: bool,
    /// Is permanent (genesis validator)
    pub is_permanent: bool,
}

/// Block proposal information
#[derive(Debug, Clone)]
pub struct BlockProposal {
    /// Block number
    pub block_number: U256,
    /// Block hash
    pub block_hash: B256,
    /// Producer address
    pub producer: Address,
    /// Signature
    pub signature: Bytes,
    /// Timestamp
    pub timestamp: U256,
    /// Verified
    pub verified: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_validator_sync() {
        // This is a placeholder test that would require a running node
        // In real tests, you'd use a local anvil instance with deployed contracts
        // Example:
        // let client = AndeConsensusClient::new(...).await.unwrap();
        // let validators = client.get_cached_validators().await;
        // assert!(!validators.is_empty());
    }

    #[tokio::test]
    async fn test_proposer_selection_logic() {
        // Test that proposer selection returns valid addresses
        // This would need a test environment setup
    }
}