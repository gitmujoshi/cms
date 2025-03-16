//! Contract Management System Client
//! 
//! This module provides a high-level client interface for interacting with the Contract Management System API.
//! It handles DID-based authentication, contract operations, and blockchain interactions.
//! 
//! # Features
//! - DID registration and authentication
//! - Contract creation and management
//! - Blockchain event verification
//! - Secure communication with JWT tokens
//! 
//! # Example
//! ```rust
//! let client = ContractClient::new("https://api.example.com")?;
//! client.register_did("did:example:123", "public_key").await?;
//! client.authenticate("did:example:123", &wallet).await?;
//! 
//! let contract = ContractData {
//!     title: "Test Contract".to_string(),
//!     description: "Example contract".to_string(),
//!     consumer_did: "did:example:456".to_string(),
//!     terms: "Contract terms...".to_string(),
//!     valid_from: "2024-03-21T00:00:00Z".to_string(),
//!     valid_until: "2025-03-21T00:00:00Z".to_string(),
//! };
//! 
//! let result = client.create_contract(contract).await?;
//! ```

// External crate imports for functionality
use chrono::{DateTime, Duration, Utc};
use ed25519_dalek::{Keypair, SecretKey, Signer};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::json;
use uuid::Uuid;
use rand::rngs::OsRng;
use ethers::{
    signers::{LocalWallet, Signer},
    types::Signature,
};
use anyhow::Result;
use reqwest::header;
use hex;

/// Represents the data structure for a contract in the system
#[derive(Debug, Serialize)]
pub struct ContractData {
    /// Title of the contract
    title: String,
    /// Detailed description of the contract
    description: String,
    /// DID of the contract consumer/counterparty
    consumer_did: String,
    /// Legal terms and conditions of the contract
    terms: String,
    /// ISO 8601 timestamp when the contract becomes valid
    valid_from: String,
    /// ISO 8601 timestamp when the contract expires
    valid_until: String,
}

/// Response type for contract-related API endpoints
/// Represents the structure of a contract in the system
#[derive(Debug, Deserialize)]
struct ContractResponse {
    /// Unique identifier for the contract
    id: Uuid,
    /// Title of the contract
    title: String,
    /// DID of the service provider
    provider_did: String,
    /// DID of the service consumer
    consumer_did: String,
    /// Current status of the contract (e.g., "Draft", "PendingSignatures", "Active")
    status: String,
    /// List of signatures applied to the contract
    signatures: Vec<SignatureResponse>,
}

/// Represents a signature applied to a contract
#[derive(Debug, Deserialize)]
struct SignatureResponse {
    /// DID of the entity that signed the contract
    signer_did: String,
    /// Timestamp when the signature was applied
    signed_at: DateTime<Utc>,
}

/// Response type for successful authentication
#[derive(Debug, Deserialize)]
struct AuthResponse {
    /// JWT token for subsequent authenticated requests
    token: String,
}

/// Manages DID-based keypairs for authentication and signing
/// Uses Ed25519 for cryptographic operations
struct DIDKeyPair {
    /// The DID identifier derived from the public key
    did: String,
    /// Ed25519 keypair for signing operations
    keypair: Keypair,
}

impl DIDKeyPair {
    /// Creates a new DID keypair using Ed25519
    /// 
    /// # Returns
    /// A new DIDKeyPair instance with a randomly generated keypair and derived DID
    fn new() -> Self {
        let mut csprng = OsRng;
        let keypair = Keypair::generate(&mut csprng);
        // Create a did:key identifier using the public key
        let did = format!("did:key:ed25519:{}", bs58::encode(keypair.public.as_bytes()).into_string());
        
        Self { did, keypair }
    }

    /// Signs a message using the Ed25519 keypair
    /// 
    /// # Arguments
    /// * `message` - The message bytes to sign
    /// 
    /// # Returns
    /// Base58-encoded signature string
    fn sign(&self, message: &[u8]) -> String {
        let signature = self.keypair.sign(message);
        bs58::encode(signature.to_bytes()).into_string()
    }
}

/// Main client for interacting with the Contract Management System API
/// Handles authentication, contract operations, and signature verification
pub struct ContractClient {
    /// HTTP client for making API requests
    client: Client,
    /// Base URL of the API
    base_url: String,
    /// JWT token for authenticated requests
    auth_token: Option<String>,
    /// DID keypair for authentication and signing
    keypair: DIDKeyPair,
}

impl ContractClient {
    /// Creates a new ContractClient instance
    /// 
    /// # Arguments
    /// * `base_url` - Base URL of the Contract Management System API
    pub fn new(base_url: &str) -> Result<Self> {
        Ok(Self {
            client: Client::new(),
            base_url: base_url.to_string(),
            auth_token: None,
            keypair: DIDKeyPair::new(),
        })
    }

    /// Registers a new DID with the system
    /// 
    /// # Arguments
    /// * `did` - Decentralized Identifier to register
    /// * `public_key` - Associated public key for the DID
    /// 
    /// # Returns
    /// * `Result<()>` - Success or error
    pub async fn register_did(&self, did: &str, public_key: &str) -> Result<()> {
        self.client
            .post(&format!("{}/api/v1/auth/register", self.base_url))
            .json(&serde_json::json!({
                "did": did,
                "public_key": public_key,
            }))
            .send()
            .await?;
        Ok(())
    }

    /// Authenticates with the system using DID and wallet
    /// 
    /// This method performs the challenge-response authentication flow:
    /// 1. Requests a challenge from the server
    /// 2. Signs the challenge with the provided wallet
    /// 3. Submits the signature for verification
    /// 4. Stores the JWT token for subsequent requests
    /// 
    /// # Arguments
    /// * `did` - DID to authenticate with
    /// * `wallet` - Ethereum wallet for signing the challenge
    /// 
    /// # Returns
    /// * `Result<()>` - Success or error
    pub async fn authenticate(&mut self, did: &str, wallet: &LocalWallet) -> Result<()> {
        // Get challenge from server
        let challenge_resp = self.client
            .post(&format!("{}/api/v1/auth/challenge", self.base_url))
            .json(&serde_json::json!({ "did": did }))
            .send()
            .await?;
        
        let challenge: String = challenge_resp.json::<serde_json::Value>().await?
            .get("challenge")
            .and_then(|v| v.as_str())
            .unwrap()
            .to_string();

        // Sign challenge with wallet
        let signature: Signature = wallet.sign_message(challenge.as_bytes()).await?;

        // Submit signature for verification
        let verify_resp = self.client
            .post(&format!("{}/api/v1/auth/verify", self.base_url))
            .json(&serde_json::json!({
                "did": did,
                "challenge": challenge,
                "signature": format!("0x{}", hex::encode(signature.to_vec())),
            }))
            .send()
            .await?;

        // Store JWT token
        let auth_response: AuthResponse = verify_resp.json().await?;
        self.auth_token = Some(auth_response.token);
        Ok(())
    }

    /// Creates a new contract in the system
    /// 
    /// # Arguments
    /// * `contract_data` - Contract details to create
    /// 
    /// # Returns
    /// * `Result<serde_json::Value>` - Created contract data or error
    /// 
    /// # Errors
    /// Returns error if not authenticated or if contract creation fails
    pub async fn create_contract(&self, contract_data: ContractData) -> Result<serde_json::Value> {
        let token = self.auth_token.as_ref()
            .ok_or_else(|| anyhow::anyhow!("Not authenticated"))?;

        let mut headers = header::HeaderMap::new();
        headers.insert(
            header::AUTHORIZATION,
            header::HeaderValue::from_str(&format!("Bearer {}", token))?,
        );

        let response = self.client
            .post(&format!("{}/api/v1/contracts", self.base_url))
            .headers(headers)
            .json(&contract_data)
            .send()
            .await?;

        Ok(response.json().await?)
    }

    /// Retrieves contract details by ID
    /// 
    /// # Arguments
    /// * `contract_id` - Unique identifier of the contract
    /// 
    /// # Returns
    /// * `Result<serde_json::Value>` - Contract details or error
    /// 
    /// # Errors
    /// Returns error if not authenticated or if contract retrieval fails
    pub async fn get_contract(&self, contract_id: &str) -> Result<serde_json::Value> {
        let token = self.auth_token.as_ref()
            .ok_or_else(|| anyhow::anyhow!("Not authenticated"))?;

        let mut headers = header::HeaderMap::new();
        headers.insert(
            header::AUTHORIZATION,
            header::HeaderValue::from_str(&format!("Bearer {}", token))?,
        );

        let response = self.client
            .get(&format!("{}/api/v1/contracts/{}", self.base_url, contract_id))
            .headers(headers)
            .send()
            .await?;

        Ok(response.json().await?)
    }

    /// Signs a contract using the client's DID
    /// 
    /// The signing process:
    /// 1. Retrieves the contract details
    /// 2. Generates a signing message including contract metadata
    /// 3. Signs the message using the DID keypair
    /// 4. Submits the signature to the server
    /// 
    /// # Arguments
    /// * `contract_id` - UUID of the contract to sign
    /// 
    /// # Returns
    /// Updated ContractResponse with the new signature
    async fn sign_contract(
        &self,
        contract_id: Uuid,
    ) -> Result<ContractResponse, Box<dyn std::error::Error>> {
        // Get contract details for signing message
        let contract = self.get_contract(contract_id.to_string().as_str()).await?;

        // Generate standardized signing message
        let message = format!(
            "Contract Signature Request\n\
             Contract ID: {}\n\
             Title: {}\n\
             Provider DID: {}\n\
             Consumer DID: {}\n",
            contract["id"], contract["title"], contract["provider_did"], contract["consumer_did"]
        );

        // Sign the message
        let signature = self.keypair.sign(message.as_bytes());

        // Submit signature to server
        let response = self.client
            .post(format!("{}/contracts/{}/sign", self.base_url, contract["id"]))
            .bearer_auth(self.auth_token.as_ref().unwrap())
            .json(&json!({
                "signer_did": self.keypair.did,
                "signature": signature,
                "verification_method": format!("{}#keys-1", self.keypair.did)
            }))
            .send()
            .await?
            .json::<ContractResponse>()
            .await?;

        Ok(response)
    }

    /// Lists all contracts associated with the client's DID
    /// 
    /// # Returns
    /// Vector of ContractResponse objects
    async fn list_contracts(&self) -> Result<Vec<ContractResponse>, Box<dyn std::error::Error>> {
        let response = self.client
            .get(format!("{}/contracts", self.base_url))
            .bearer_auth(self.auth_token.as_ref().unwrap())
            .query(&[("signer_did", &self.keypair.did)])
            .send()
            .await?
            .json::<Vec<ContractResponse>>()
            .await?;

        Ok(response)
    }

    /// Verifies all signatures on a contract
    /// 
    /// # Arguments
    /// * `contract_id` - UUID of the contract to verify
    /// 
    /// # Returns
    /// Boolean indicating whether all signatures are valid
    async fn verify_signatures(
        &self,
        contract_id: Uuid,
    ) -> Result<bool, Box<dyn std::error::Error>> {
        let response = self.client
            .get(format!("{}/contracts/{}/verify", self.base_url, contract_id))
            .bearer_auth(self.auth_token.as_ref().unwrap())
            .send()
            .await?
            .json::<serde_json::Value>()
            .await?;

        Ok(response["valid"].as_bool().unwrap_or(false))
    }
}

/// Example usage of the ContractClient
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize client with API endpoint
    let mut client = ContractClient::new("http://localhost:8080")?;

    // Perform DID-based authentication
    client.authenticate("did:example:123", &LocalWallet::new(&mut OsRng)).await?;
    println!("Authenticated successfully!");

    // Create a new contract
    let contract = ContractData {
        title: "Test Contract".to_string(),
        description: "A test contract between two parties".to_string(),
        consumer_did: "did:example:consumer123".to_string(),
        terms: "1. First party agrees to...\n2. Second party agrees to...".to_string(),
        valid_from: Utc::now().to_rfc3339(),
        valid_until: (Utc::now() + Duration::days(365)).to_rfc3339(),
    };

    let result = client.create_contract(contract).await?;
    println!("Created contract: {:?}", result);

    // Sign the contract
    let signed_contract = client.sign_contract(Uuid::new_v4()).await?;
    println!("Signed contract: {:?}", signed_contract);

    // List all contracts
    let contracts = client.list_contracts().await?;
    println!("All contracts: {:?}", contracts);

    // Verify signatures
    let valid = client.verify_signatures(Uuid::new_v4()).await?;
    println!("Signatures valid: {}", valid);

    Ok(())
}

/// Integration tests for the ContractClient
#[cfg(test)]
mod tests {
    use super::*;
    use mockito::mock;

    /// Tests a complete contract workflow including:
    /// - Provider and consumer authentication
    /// - Contract creation
    /// - Multiple party signing
    /// - Signature verification
    async fn test_contract_workflow() -> Result<(), Box<dyn std::error::Error>> {
        // Set up provider and consumer clients
        let mut provider = ContractClient::new("http://localhost:8080")?;
        provider.authenticate("did:example:provider123", &LocalWallet::new(&mut OsRng)).await?;

        let mut consumer = ContractClient::new("http://localhost:8080")?;
        consumer.authenticate("did:example:consumer123", &LocalWallet::new(&mut OsRng)).await?;

        // Create contract as provider
        let contract = provider.create_contract(ContractData {
            title: "Service Agreement".to_string(),
            description: "Agreement for providing consulting services".to_string(),
            consumer_did: consumer.keypair.did,
            terms: "Terms and conditions...".to_string(),
            valid_from: Utc::now().to_rfc3339(),
            valid_until: (Utc::now() + Duration::days(365)).to_rfc3339(),
        }).await?;

        // Provider signs first
        let contract = provider.sign_contract(Uuid::new_v4()).await?;
        assert_eq!(contract.signatures.len(), 1);
        assert_eq!(contract.status, "PendingSignatures");

        // Consumer signs second
        let contract = consumer.sign_contract(Uuid::new_v4()).await?;
        assert_eq!(contract.signatures.len(), 2);
        assert_eq!(contract.status, "Active");

        // Verify all signatures
        let valid = provider.verify_signatures(Uuid::new_v4()).await?;
        assert!(valid);

        Ok(())
    }

    #[tokio::test]
    async fn test_full_workflow() {
        test_contract_workflow().await.unwrap();
    }

    #[tokio::test]
    async fn test_register_did() {
        let mock_server = mock("POST", "/api/v1/auth/register")
            .with_status(200)
            .create();

        let client = ContractClient::new("http://localhost:8080")?;
        let result = client.register_did("did:example:123", "test-key").await;
        
        assert!(result.is_ok());
        mock_server.assert();
    }
} 