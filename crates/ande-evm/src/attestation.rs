//! Block Attestation Module
//!
//! Handles signing and attesting blocks to the AndeConsensusV2 contract.

use alloy::{
    primitives::{keccak256, Bytes, B256},
    signers::local::PrivateKeySigner,
};
use eyre::Result;
use std::sync::Arc;
use tracing::{debug, info};

use crate::consensus_client::AndeConsensusClient;

/// Block attester for signing and submitting blocks to consensus contract
pub struct BlockAttester {
    /// Signer for creating ECDSA signatures
    signer: PrivateKeySigner,
    /// Consensus client for submitting proposals
    consensus_client: Arc<AndeConsensusClient>,
}

impl BlockAttester {
    /// Create a new block attester
    ///
    /// # Arguments
    /// * `signer` - Private key signer for attestations
    /// * `consensus_client` - Consensus contract client
    pub fn new(signer: PrivateKeySigner, consensus_client: Arc<AndeConsensusClient>) -> Self {
        info!(
            "BlockAttester initialized with signer address: {:?}",
            signer.address()
        );
        Self {
            signer,
            consensus_client,
        }
    }

    /// Attest a block by signing and submitting to consensus contract
    ///
    /// # Arguments
    /// * `block_number` - Block number to attest
    /// * `block_hash` - Hash of the block
    ///
    /// # Returns
    /// Transaction hash of the attestation
    pub async fn attest_block(&self, block_number: u64, block_hash: B256) -> Result<B256> {
        debug!(
            "Attesting block {} with hash {:?}",
            block_number, block_hash
        );

        // 1. Create message to sign (blockNumber || blockHash)
        let message = Self::create_attestation_message(block_number, block_hash);
        debug!("Attestation message hash: {:?}", message);

        // 2. Sign the message
        let signature = self.sign_message(message).await?;
        debug!("Signature created: {} bytes", signature.len());

        // 3. Submit to consensus contract
        let tx_hash = self
            .consensus_client
            .propose_block(block_number, block_hash, signature)
            .await?;

        info!(
            "Block {} attested successfully, tx: {:?}",
            block_number, tx_hash
        );

        Ok(tx_hash)
    }

    /// Create attestation message hash
    ///
    /// Message format: keccak256(abi.encodePacked(blockNumber, blockHash))
    fn create_attestation_message(block_number: u64, block_hash: B256) -> B256 {
        let mut message = Vec::new();
        message.extend_from_slice(&block_number.to_be_bytes());
        message.extend_from_slice(block_hash.as_slice());

        keccak256(&message)
    }

    /// Sign a message using eth_sign format
    ///
    /// This creates an Ethereum signed message hash and signs it.
    async fn sign_message(&self, message_hash: B256) -> Result<Bytes> {
        // Ethereum signed message: "\x19Ethereum Signed Message:\n32" + message_hash
        let eth_message = Self::eth_signed_message_hash(message_hash);

        // Sign the message
        let signature = self.signer.sign_hash(&eth_message).await?;

        // Convert to bytes (r + s + v format)
        let mut sig_bytes = Vec::new();
        sig_bytes.extend_from_slice(signature.r().as_le_slice());
        sig_bytes.extend_from_slice(signature.s().as_le_slice());
        sig_bytes.push(signature.v().y_parity_byte());

        Ok(Bytes::from(sig_bytes))
    }

    /// Create Ethereum signed message hash
    ///
    /// Format: keccak256("\x19Ethereum Signed Message:\n32" + message_hash)
    fn eth_signed_message_hash(message_hash: B256) -> B256 {
        let mut eth_message = Vec::new();
        eth_message.extend_from_slice(b"\x19Ethereum Signed Message:\n32");
        eth_message.extend_from_slice(message_hash.as_slice());

        keccak256(&eth_message)
    }

    /// Get the signer's address
    pub fn address(&self) -> alloy::primitives::Address {
        self.signer.address()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloy::primitives::address;

    #[test]
    fn test_attestation_message() {
        let block_number = 12345u64;
        let block_hash = B256::from([1u8; 32]);

        let message = BlockAttester::create_attestation_message(block_number, block_hash);

        // Verify message is deterministic
        let message2 = BlockAttester::create_attestation_message(block_number, block_hash);
        assert_eq!(message, message2);

        // Verify different inputs produce different hashes
        let message3 = BlockAttester::create_attestation_message(block_number + 1, block_hash);
        assert_ne!(message, message3);
    }

    #[test]
    fn test_eth_signed_message_hash() {
        let message_hash = B256::from([1u8; 32]);
        let eth_hash = BlockAttester::eth_signed_message_hash(message_hash);

        // Verify deterministic
        let eth_hash2 = BlockAttester::eth_signed_message_hash(message_hash);
        assert_eq!(eth_hash, eth_hash2);

        // Verify format
        assert_ne!(eth_hash, message_hash); // Should be different after wrapping
    }

    #[tokio::test]
    async fn test_signer_address() {
        // Create a test private key
        let signer = PrivateKeySigner::random();
        let expected_address = signer.address();

        // Mock consensus client (won't actually be used in this test)
        // In real tests, you'd use a mock or testnet

        // Just verify the address is accessible
        assert_ne!(expected_address, address!("0000000000000000000000000000000000000000"));
    }
}
