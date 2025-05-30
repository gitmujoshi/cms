//! Contract Management System - Ledger Service
//! 
//! This module implements the distributed ledger service for the Contract Management System.
//! It provides blockchain integration and transaction management including:
//! - Transaction recording
//! - Smart contract deployment
//! - Event logging
//! - State verification
//! - Transaction history
//!
//! Features:
//! - Blockchain network integration
//! - Transaction management
//! - Smart contract interaction
//! - Event subscription
//! - State synchronization
//!
//! Security Features:
//! - Transaction signing
//! - State verification
//! - Consensus validation
//! - Network security
//! - Data integrity
//!
//! Integration Points:
//! - Blockchain network
//! - Smart contracts
//! - Event system
//! - State management
//! - Audit system
//!
//! Usage:
//! 1. Initialize the ledger service
//! 2. Record transactions
//! 3. Deploy smart contracts
//! 4. Monitor events
//! 5. Verify state
//!
//! Author: Contract Management System Team
//! License: MIT

use ethers::{
    prelude::*,
    providers::{Http, Provider},
    types::{Address, TransactionRequest, U256},
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use uuid::Uuid;
use std::str::FromStr;

/// Events that can be recorded on the blockchain
#[derive(Debug, Serialize, Deserialize)]
pub enum ContractEvent {
    Created {
        contract_id: Uuid,
        provider_did: String,
        consumer_did: String,
        content_hash: String,
    },
    Signed {
        contract_id: Uuid,
        signer_did: String,
        signature: String,
        timestamp: u64,
    },
    Updated {
        contract_id: Uuid,
        content_hash: String,
        updater_did: String,
    },
    Voided {
        contract_id: Uuid,
        reason: String,
        voider_did: String,
    },
}

/// Smart contract interface for contract events
#[ethers::contract]
pub trait ContractLedger {
    fn record_event(&mut self, event_type: String, contract_id: String, data: String) -> Result<(), String>;
    fn get_contract_events(&self, contract_id: String) -> Result<Vec<(String, String, u64)>, String>;
    fn verify_contract_state(&self, contract_id: String, expected_hash: String) -> Result<bool, String>;
}

/// Service for interacting with the blockchain ledger
pub struct LedgerService {
    provider: Arc<Provider<Http>>,
    contract: ContractLedger<Provider<Http>>,
    wallet: LocalWallet,
}

impl LedgerService {
    /// Creates a new LedgerService instance
    /// 
    /// # Arguments
    /// * `rpc_url` - URL of the Ethereum RPC endpoint
    /// * `contract_address` - Address of the deployed ContractLedger smart contract
    /// * `private_key` - Private key for transaction signing
    pub async fn new(
        rpc_url: &str,
        contract_address: &str,
        private_key: &str,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let provider = Provider::<Http>::try_from(rpc_url)?;
        let provider = Arc::new(provider);

        let wallet = LocalWallet::from_str(private_key)?;
        let contract_addr = Address::from_str(contract_address)?;
        
        let contract = ContractLedger::new(contract_addr, provider.clone());

        Ok(Self {
            provider,
            contract,
            wallet,
        })
    }

    /// Records a contract event on the blockchain
    /// 
    /// # Arguments
    /// * `event` - The contract event to record
    pub async fn record_event(
        &self,
        event: ContractEvent,
    ) -> Result<H256, Box<dyn std::error::Error>> {
        let (event_type, contract_id, data) = match event {
            ContractEvent::Created { contract_id, provider_did, consumer_did, content_hash } => (
                "CREATED".to_string(),
                contract_id.to_string(),
                serde_json::to_string(&json!({
                    "provider_did": provider_did,
                    "consumer_did": consumer_did,
                    "content_hash": content_hash,
                }))?,
            ),
            ContractEvent::Signed { contract_id, signer_did, signature, timestamp } => (
                "SIGNED".to_string(),
                contract_id.to_string(),
                serde_json::to_string(&json!({
                    "signer_did": signer_did,
                    "signature": signature,
                    "timestamp": timestamp,
                }))?,
            ),
            ContractEvent::Updated { contract_id, content_hash, updater_did } => (
                "UPDATED".to_string(),
                contract_id.to_string(),
                serde_json::to_string(&json!({
                    "content_hash": content_hash,
                    "updater_did": updater_did,
                }))?,
            ),
            ContractEvent::Voided { contract_id, reason, voider_did } => (
                "VOIDED".to_string(),
                contract_id.to_string(),
                serde_json::to_string(&json!({
                    "reason": reason,
                    "voider_did": voider_did,
                }))?,
            ),
        };

        let tx = self.contract
            .record_event(event_type, contract_id, data)
            .from(self.wallet.address())
            .gas(U256::from(200000));

        let pending_tx = tx.send().await?;
        let receipt = pending_tx.await?;

        Ok(receipt.transaction_hash)
    }

    /// Retrieves all events for a specific contract
    /// 
    /// # Arguments
    /// * `contract_id` - UUID of the contract
    pub async fn get_contract_events(
        &self,
        contract_id: Uuid,
    ) -> Result<Vec<(String, String, u64)>, Box<dyn std::error::Error>> {
        let events = self.contract
            .get_contract_events(contract_id.to_string())
            .call()
            .await?;

        Ok(events)
    }

    /// Verifies the current state of a contract matches the expected hash
    /// 
    /// # Arguments
    /// * `contract_id` - UUID of the contract
    /// * `expected_hash` - Expected hash of the contract state
    pub async fn verify_contract_state(
        &self,
        contract_id: Uuid,
        expected_hash: &str,
    ) -> Result<bool, Box<dyn std::error::Error>> {
        let is_valid = self.contract
            .verify_contract_state(contract_id.to_string(), expected_hash.to_string())
            .call()
            .await?;

        Ok(is_valid)
    }

    /// Calculates the hash of a contract's content
    /// 
    /// # Arguments
    /// * `content` - Contract content to hash
    pub fn calculate_content_hash(&self, content: &str) -> String {
        use sha3::{Digest, Keccak256};
        let mut hasher = Keccak256::new();
        hasher.update(content.as_bytes());
        hex::encode(hasher.finalize())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio;

    #[tokio::test]
    async fn test_ledger_integration() {
        let service = LedgerService::new(
            "http://localhost:8545",
            "0x1234567890123456789012345678901234567890",
            "0x1234567890123456789012345678901234567890123456789012345678901234",
        ).await.unwrap();

        let contract_id = Uuid::new_v4();
        let content = "Test contract content";
        let content_hash = service.calculate_content_hash(content);

        // Record contract creation
        let tx_hash = service.record_event(ContractEvent::Created {
            contract_id,
            provider_did: "did:example:provider".to_string(),
            consumer_did: "did:example:consumer".to_string(),
            content_hash: content_hash.clone(),
        }).await.unwrap();
        println!("Creation recorded: {:?}", tx_hash);

        // Record signature
        let tx_hash = service.record_event(ContractEvent::Signed {
            contract_id,
            signer_did: "did:example:provider".to_string(),
            signature: "test_signature".to_string(),
            timestamp: 1234567890,
        }).await.unwrap();
        println!("Signature recorded: {:?}", tx_hash);

        // Verify contract state
        let is_valid = service.verify_contract_state(contract_id, &content_hash).await.unwrap();
        assert!(is_valid);

        // Get all events
        let events = service.get_contract_events(contract_id).await.unwrap();
        assert_eq!(events.len(), 2);
    }
} 