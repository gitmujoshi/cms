use chrono::{DateTime, Duration, Utc};
use ed25519_dalek::{Keypair, SecretKey, Signer};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::json;
use uuid::Uuid;
use rand::rngs::OsRng;

// API Response Types
#[derive(Debug, Deserialize)]
struct ContractResponse {
    id: Uuid,
    title: String,
    provider_did: String,
    consumer_did: String,
    status: String,
    signatures: Vec<SignatureResponse>,
}

#[derive(Debug, Deserialize)]
struct SignatureResponse {
    signer_did: String,
    signed_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
struct AuthResponse {
    token: String,
}

// DID Key Management
struct DIDKeyPair {
    did: String,
    keypair: Keypair,
}

impl DIDKeyPair {
    fn new() -> Self {
        let mut csprng = OsRng;
        let keypair = Keypair::generate(&mut csprng);
        let did = format!("did:key:ed25519:{}", bs58::encode(keypair.public.as_bytes()).into_string());
        
        Self { did, keypair }
    }

    fn sign(&self, message: &[u8]) -> String {
        let signature = self.keypair.sign(message);
        bs58::encode(signature.to_bytes()).into_string()
    }
}

// API Client
struct ContractClient {
    client: Client,
    base_url: String,
    auth_token: Option<String>,
    keypair: DIDKeyPair,
}

impl ContractClient {
    fn new(base_url: String) -> Self {
        Self {
            client: Client::new(),
            base_url,
            auth_token: None,
            keypair: DIDKeyPair::new(),
        }
    }

    async fn authenticate(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        // 1. Request authentication challenge
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

        // 2. Sign the challenge
        let signature = self.keypair.sign(challenge.as_bytes());

        // 3. Verify signature and get token
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

    async fn sign_contract(
        &self,
        contract_id: Uuid,
    ) -> Result<ContractResponse, Box<dyn std::error::Error>> {
        // 1. Get contract to generate signing message
        let contract = self.get_contract(contract_id).await?;

        // 2. Generate signing message
        let message = format!(
            "Contract Signature Request\n\
             Contract ID: {}\n\
             Title: {}\n\
             Provider DID: {}\n\
             Consumer DID: {}\n",
            contract.id, contract.title, contract.provider_did, contract.consumer_did
        );

        // 3. Sign message
        let signature = self.keypair.sign(message.as_bytes());

        // 4. Submit signature
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

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize client
    let mut client = ContractClient::new("http://localhost:8080".to_string());

    // Authenticate
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

// Example of using the client in a more complex scenario
#[cfg(test)]
mod tests {
    use super::*;

    async fn test_contract_workflow() -> Result<(), Box<dyn std::error::Error>> {
        // Initialize provider client
        let mut provider = ContractClient::new("http://localhost:8080".to_string());
        provider.authenticate().await?;

        // Initialize consumer client
        let mut consumer = ContractClient::new("http://localhost:8080".to_string());
        consumer.authenticate().await?;

        // Provider creates contract
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