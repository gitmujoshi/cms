// External crate imports for functionality
use chrono::{DateTime, Duration, Utc};
use ed25519_dalek::{Keypair, SecretKey, Signer};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::json;
use uuid::Uuid;
use rand::rngs::OsRng;

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
struct ContractClient {
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
    fn new(base_url: String) -> Self {
        Self {
            client: Client::new(),
            base_url,
            auth_token: None,
            keypair: DIDKeyPair::new(),
        }
    }

    /// Performs DID-based authentication using challenge-response
    /// 
    /// The authentication flow:
    /// 1. Request a challenge from the server
    /// 2. Sign the challenge using the DID keypair
    /// 3. Submit the signature for verification
    /// 4. Receive and store the JWT token
    /// 
    /// # Returns
    /// Result indicating success or failure of authentication
    async fn authenticate(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        // Request authentication challenge
        let challenge_resp = self.client
            .post(format!("{}/auth/challenge", self.base_url))
            .json(&json!({
                "did": self.keypair.did
            }))
            .send()
            .await?
            .json::<serde_json::Value>()
            .await?;

        let challenge = challenge_resp["challenge"].as_str().unwrap();

        // Sign the challenge using our DID keypair
        let signature = self.keypair.sign(challenge.as_bytes());

        // Verify signature and obtain JWT token
        let auth_resp = self.client
            .post(format!("{}/auth/verify", self.base_url))
            .json(&json!({
                "did": self.keypair.did,
                "challenge": challenge,
                "signature": signature,
                "verification_method": format!("{}#keys-1", self.keypair.did)
            }))
            .send()
            .await?
            .json::<AuthResponse>()
            .await?;

        self.auth_token = Some(auth_resp.token);
        Ok(())
    }

    /// Creates a new contract in the system
    /// 
    /// # Arguments
    /// * `title` - Title of the contract
    /// * `description` - Description of the contract
    /// * `consumer_did` - DID of the consumer party
    /// * `terms` - Terms and conditions of the contract
    /// 
    /// # Returns
    /// ContractResponse containing the created contract details
    async fn create_contract(
        &self,
        title: &str,
        description: &str,
        consumer_did: &str,
        terms: &str,
    ) -> Result<ContractResponse, Box<dyn std::error::Error>> {
        let response = self.client
            .post(format!("{}/contracts", self.base_url))
            .bearer_auth(self.auth_token.as_ref().unwrap())
            .json(&json!({
                "title": title,
                "description": description,
                "provider_did": self.keypair.did,
                "consumer_did": consumer_did,
                "terms": terms,
                "valid_from": Utc::now(),
                "valid_until": Utc::now() + Duration::days(365)
            }))
            .send()
            .await?
            .json::<ContractResponse>()
            .await?;

        Ok(response)
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
        let contract = self.get_contract(contract_id).await?;

        // Generate standardized signing message
        let message = format!(
            "Contract Signature Request\n\
             Contract ID: {}\n\
             Title: {}\n\
             Provider DID: {}\n\
             Consumer DID: {}\n",
            contract.id, contract.title, contract.provider_did, contract.consumer_did
        );

        // Sign the message
        let signature = self.keypair.sign(message.as_bytes());

        // Submit signature to server
        let response = self.client
            .post(format!("{}/contracts/{}/sign", self.base_url, contract_id))
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

    /// Retrieves a specific contract by ID
    /// 
    /// # Arguments
    /// * `contract_id` - UUID of the contract to retrieve
    /// 
    /// # Returns
    /// ContractResponse containing the contract details
    async fn get_contract(
        &self,
        contract_id: Uuid,
    ) -> Result<ContractResponse, Box<dyn std::error::Error>> {
        let response = self.client
            .get(format!("{}/contracts/{}", self.base_url, contract_id))
            .bearer_auth(self.auth_token.as_ref().unwrap())
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
    let mut client = ContractClient::new("http://localhost:8080".to_string());

    // Perform DID-based authentication
    client.authenticate().await?;
    println!("Authenticated successfully!");

    // Create a new contract
    let contract = client.create_contract(
        "Test Contract",
        "A test contract between two parties",
        "did:example:consumer123",
        "1. First party agrees to...\n2. Second party agrees to...",
    ).await?;
    println!("Created contract: {:?}", contract);

    // Sign the contract
    let signed_contract = client.sign_contract(contract.id).await?;
    println!("Signed contract: {:?}", signed_contract);

    // List all contracts
    let contracts = client.list_contracts().await?;
    println!("All contracts: {:?}", contracts);

    // Verify signatures
    let valid = client.verify_signatures(contract.id).await?;
    println!("Signatures valid: {}", valid);

    Ok(())
}

/// Integration tests for the ContractClient
#[cfg(test)]
mod tests {
    use super::*;

    /// Tests a complete contract workflow including:
    /// - Provider and consumer authentication
    /// - Contract creation
    /// - Multiple party signing
    /// - Signature verification
    async fn test_contract_workflow() -> Result<(), Box<dyn std::error::Error>> {
        // Set up provider and consumer clients
        let mut provider = ContractClient::new("http://localhost:8080".to_string());
        provider.authenticate().await?;

        let mut consumer = ContractClient::new("http://localhost:8080".to_string());
        consumer.authenticate().await?;

        // Create contract as provider
        let contract = provider.create_contract(
            "Service Agreement",
            "Agreement for providing consulting services",
            &consumer.keypair.did,
            "Terms and conditions...",
        ).await?;

        // Provider signs first
        let contract = provider.sign_contract(contract.id).await?;
        assert_eq!(contract.signatures.len(), 1);
        assert_eq!(contract.status, "PendingSignatures");

        // Consumer signs second
        let contract = consumer.sign_contract(contract.id).await?;
        assert_eq!(contract.signatures.len(), 2);
        assert_eq!(contract.status, "Active");

        // Verify all signatures
        let valid = provider.verify_signatures(contract.id).await?;
        assert!(valid);

        Ok(())
    }

    #[tokio::test]
    async fn test_full_workflow() {
        test_contract_workflow().await.unwrap();
    }
} 