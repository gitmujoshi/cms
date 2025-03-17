use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use sea_orm::entity::prelude::*;

use crate::auth::did::SignatureProof;

/// Represents the role of a party in the confidential computing contract
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum PartyRole {
    /// Provider of the data to be processed
    DataProvider,
    /// Consumer who will process the data
    DataConsumer,
    /// Provider of the confidential computing infrastructure
    InfrastructureProvider,
}

/// Represents a party in the contract
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContractParty {
    /// Decentralized Identifier of the party
    pub did: String,
    /// Role of the party in the contract
    pub role: PartyRole,
    /// Timestamp when the party signed the contract
    pub signed_at: Option<DateTime<Utc>>,
    /// Signature of the party
    pub signature: Option<String>,
}

/// Represents the data processing terms
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataProcessingTerms {
    /// Description of the data to be processed
    pub data_description: String,
    /// Allowed processing operations
    pub allowed_operations: Vec<String>,
    /// Data retention period in days
    pub retention_period_days: u32,
    /// Data access controls
    pub access_controls: Vec<String>,
    /// Required security measures
    pub security_requirements: Vec<String>,
}

/// Represents the infrastructure requirements
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InfrastructureRequirements {
    /// Type of confidential computing environment (e.g., "AWS Nitro Enclave")
    pub enclave_type: String,
    /// Required attestation type
    pub attestation_type: String,
    /// Minimum security level required
    pub security_level: String,
    /// Required certifications
    pub certifications: Vec<String>,
    /// Performance requirements
    pub performance_requirements: Vec<String>,
}

/// Represents the contract status
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ContractStatus {
    /// Contract is created but not signed
    Draft,
    /// Awaiting signatures from parties
    PendingSignatures,
    /// Contract is active and in force
    Active,
    /// Contract has been completed
    Completed,
    /// Contract has been terminated
    Terminated,
    /// Contract has expired
    Expired,
}

/// Main contract structure for confidential computing agreements
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Contract {
    /// Unique identifier for the contract
    pub id: Uuid,
    /// Contract title
    pub title: String,
    /// Contract description
    pub description: String,
    /// Contract version
    pub version: String,
    /// Contract creation timestamp
    pub created_at: DateTime<Utc>,
    /// Contract parties (Data Provider, Data Consumer, Infrastructure Provider)
    pub parties: Vec<ContractParty>,
    /// Data processing terms
    pub processing_terms: DataProcessingTerms,
    /// Infrastructure requirements
    pub infrastructure_requirements: InfrastructureRequirements,
    /// Contract validity period start
    pub valid_from: DateTime<Utc>,
    /// Contract validity period end
    pub valid_until: DateTime<Utc>,
    /// Current contract status
    pub status: ContractStatus,
    /// Blockchain transaction hash where the contract is recorded
    pub blockchain_tx_hash: Option<String>,
    /// Contract termination reason (if terminated)
    pub termination_reason: Option<String>,
}

impl Contract {
    /// Creates a new contract instance
    pub fn new(
        title: String,
        description: String,
        processing_terms: DataProcessingTerms,
        infrastructure_requirements: InfrastructureRequirements,
        valid_from: DateTime<Utc>,
        valid_until: DateTime<Utc>,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            title,
            description,
            version: "1.0".to_string(),
            created_at: Utc::now(),
            parties: Vec::new(),
            processing_terms,
            infrastructure_requirements,
            valid_from,
            valid_until,
            status: ContractStatus::Draft,
            blockchain_tx_hash: None,
            termination_reason: None,
        }
    }

    /// Adds a party to the contract
    pub fn add_party(&mut self, did: String, role: PartyRole) -> Result<(), String> {
        // For Data Consumer and Infrastructure Provider, check if role already exists
        match role {
            PartyRole::DataConsumer | PartyRole::InfrastructureProvider => {
                if self.parties.iter().any(|p| p.role == role) {
                    return Err(format!("Party with role {:?} already exists", role));
                }
            },
            PartyRole::DataProvider => {
                // Allow multiple Data Providers
                if self.parties.iter().any(|p| p.did == did) {
                    return Err("Party with this DID already exists".to_string());
                }
            }
        }

        self.parties.push(ContractParty {
            did,
            role,
            signed_at: None,
            signature: None,
        });

        Ok(())
    }

    /// Records a signature from a party
    pub fn sign(&mut self, did: &str, signature: String) -> Result<(), String> {
        let party = self.parties
            .iter_mut()
            .find(|p| p.did == did)
            .ok_or_else(|| "Party not found".to_string())?;

        party.signature = Some(signature);
        party.signed_at = Some(Utc::now());

        // Check if all parties have signed
        if self.parties.iter().all(|p| p.signature.is_some()) {
            self.status = ContractStatus::Active;
        } else {
            self.status = ContractStatus::PendingSignatures;
        }

        Ok(())
    }

    /// Validates the contract state
    pub fn validate(&self) -> Result<(), Vec<String>> {
        let mut errors = Vec::new();

        // Check required parties
        let data_providers = self.parties.iter().filter(|p| p.role == PartyRole::DataProvider).count();
        let data_consumers = self.parties.iter().filter(|p| p.role == PartyRole::DataConsumer).count();
        let infra_providers = self.parties.iter().filter(|p| p.role == PartyRole::InfrastructureProvider).count();

        if data_providers == 0 {
            errors.push("At least one Data Provider required".to_string());
        }
        if data_consumers != 1 {
            errors.push("Exactly one Data Consumer required".to_string());
        }
        if infra_providers != 1 {
            errors.push("Exactly one Infrastructure Provider required".to_string());
        }

        // Check validity period
        if self.valid_until <= self.valid_from {
            errors.push("Invalid validity period".to_string());
        }

        // Check processing terms
        if self.processing_terms.allowed_operations.is_empty() {
            errors.push("No allowed operations specified".to_string());
        }
        if self.processing_terms.retention_period_days == 0 {
            errors.push("Invalid retention period".to_string());
        }

        // Check infrastructure requirements
        if self.infrastructure_requirements.enclave_type.is_empty() {
            errors.push("No enclave type specified".to_string());
        }
        if self.infrastructure_requirements.attestation_type.is_empty() {
            errors.push("No attestation type specified".to_string());
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }

    /// Terminates the contract
    pub fn terminate(&mut self, reason: String) {
        self.status = ContractStatus::Terminated;
        self.termination_reason = Some(reason);
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContractSignature {
    pub id: Uuid,
    pub contract_id: Uuid,
    pub signer_did: String,
    pub signature: String,
    pub signature_type: SignatureType,
    pub verification_method: String,
    pub signed_at: DateTime<Utc>,
    pub proof: SignatureProof,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum SignatureType {
    Ed25519,
    EcdsaSecp256k1,
}

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "contracts")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: Uuid,
    pub title: String,
    pub description: String,
    pub provider_did: String,
    pub consumer_did: String,
    pub terms: String,
    pub status: ContractStatus,
    pub valid_from: DateTime<Utc>,
    pub valid_until: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    #[sea_orm(column_type = "JsonBinary")]
    pub signatures: Vec<ContractSignature>,
    #[sea_orm(column_type = "JsonBinary")]
    pub metadata: serde_json::Value,
}

impl Model {
    pub fn new(
        title: String,
        description: String,
        provider_did: String,
        consumer_did: String,
        terms: String,
        valid_from: DateTime<Utc>,
        valid_until: Option<DateTime<Utc>>,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            title,
            description,
            provider_did,
            consumer_did,
            terms,
            status: ContractStatus::Draft,
            valid_from,
            valid_until,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            signatures: Vec::new(),
            metadata: serde_json::Value::Object(serde_json::Map::new()),
        }
    }

    pub fn generate_signing_message(&self) -> String {
        format!(
            "Contract Signature Request\n\
             Contract ID: {}\n\
             Title: {}\n\
             Provider DID: {}\n\
             Consumer DID: {}\n\
             Valid From: {}\n\
             Terms Hash: {}\n\
             Timestamp: {}",
            self.id,
            self.title,
            self.provider_did,
            self.consumer_did,
            self.valid_from,
            sha256::digest(&self.terms),
            Utc::now()
        )
    }

    pub fn add_signature(&mut self, signature: ContractSignature) -> Result<(), String> {
        // Verify signer is authorized
        if signature.signer_did != self.provider_did && signature.signer_did != self.consumer_did {
            return Err("Unauthorized signer".to_string());
        }

        // Check for duplicate signatures
        if self.signatures.iter().any(|s| s.signer_did == signature.signer_did) {
            return Err("Contract already signed by this DID".to_string());
        }

        // Add signature
        self.signatures.push(signature);

        // Update contract status
        self.status = match self.signatures.len() {
            1 => ContractStatus::PendingSignatures,
            2 => ContractStatus::Active,
            _ => self.status.clone(),
        };

        self.updated_at = Utc::now();
        Ok(())
    }

    pub fn is_signed_by(&self, did: &str) -> bool {
        self.signatures.iter().any(|s| s.signer_did == did)
    }

    pub fn is_fully_signed(&self) -> bool {
        self.signatures.len() == 2
    }
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_contract() -> Contract {
        let processing_terms = DataProcessingTerms {
            data_description: "Test data".to_string(),
            allowed_operations: vec!["analyze".to_string()],
            retention_period_days: 30,
            access_controls: vec!["encryption".to_string()],
            security_requirements: vec!["attestation".to_string()],
        };

        let infrastructure_requirements = InfrastructureRequirements {
            enclave_type: "AWS Nitro".to_string(),
            attestation_type: "DCE".to_string(),
            security_level: "High".to_string(),
            certifications: vec!["ISO27001".to_string()],
            performance_requirements: vec!["4 vCPUs".to_string()],
        };

        Contract::new(
            "Test Contract".to_string(),
            "Test Description".to_string(),
            processing_terms,
            infrastructure_requirements,
            Utc::now(),
            Utc::now() + chrono::Duration::days(30),
        )
    }

    #[test]
    fn test_contract_creation() {
        let contract = create_test_contract();
        assert_eq!(contract.status, ContractStatus::Draft);
        assert_eq!(contract.parties.len(), 0);
    }

    #[test]
    fn test_add_parties() {
        let mut contract = create_test_contract();
        
        assert!(contract.add_party("did:example:provider".to_string(), PartyRole::DataProvider).is_ok());
        assert!(contract.add_party("did:example:consumer".to_string(), PartyRole::DataConsumer).is_ok());
        assert!(contract.add_party("did:example:infra".to_string(), PartyRole::InfrastructureProvider).is_ok());
        
        assert_eq!(contract.parties.len(), 3);
    }

    #[test]
    fn test_signing_flow() {
        let mut contract = create_test_contract();
        
        contract.add_party("did:example:provider".to_string(), PartyRole::DataProvider).unwrap();
        contract.add_party("did:example:consumer".to_string(), PartyRole::DataConsumer).unwrap();
        contract.add_party("did:example:infra".to_string(), PartyRole::InfrastructureProvider).unwrap();

        assert!(contract.sign("did:example:provider", "sig1".to_string()).is_ok());
        assert_eq!(contract.status, ContractStatus::PendingSignatures);

        assert!(contract.sign("did:example:consumer", "sig2".to_string()).is_ok());
        assert_eq!(contract.status, ContractStatus::PendingSignatures);

        assert!(contract.sign("did:example:infra", "sig3".to_string()).is_ok());
        assert_eq!(contract.status, ContractStatus::Active);
    }

    #[test]
    fn test_contract_validation() {
        let mut contract = create_test_contract();
        let validation_result = contract.validate();
        assert!(validation_result.is_err());
        
        contract.add_party("did:example:provider".to_string(), PartyRole::DataProvider).unwrap();
        contract.add_party("did:example:consumer".to_string(), PartyRole::DataConsumer).unwrap();
        contract.add_party("did:example:infra".to_string(), PartyRole::InfrastructureProvider).unwrap();
        
        assert!(contract.validate().is_ok());
    }

    #[test]
    fn test_multiple_data_providers() {
        let mut contract = create_test_contract();
        
        // Add multiple data providers
        assert!(contract.add_party("did:example:provider1".to_string(), PartyRole::DataProvider).is_ok());
        assert!(contract.add_party("did:example:provider2".to_string(), PartyRole::DataProvider).is_ok());
        assert!(contract.add_party("did:example:provider3".to_string(), PartyRole::DataProvider).is_ok());
        
        // Add other required parties
        assert!(contract.add_party("did:example:consumer".to_string(), PartyRole::DataConsumer).is_ok());
        assert!(contract.add_party("did:example:infra".to_string(), PartyRole::InfrastructureProvider).is_ok());
        
        // Validate contract
        assert!(contract.validate().is_ok());
        
        // Verify number of parties by role
        let data_providers = contract.parties.iter().filter(|p| p.role == PartyRole::DataProvider).count();
        assert_eq!(data_providers, 3);
        
        // Try to add duplicate data provider
        assert!(contract.add_party("did:example:provider1".to_string(), PartyRole::DataProvider).is_err());
    }

    #[test]
    fn test_signing_flow_multiple_providers() {
        let mut contract = create_test_contract();
        
        // Add multiple parties
        contract.add_party("did:example:provider1".to_string(), PartyRole::DataProvider).unwrap();
        contract.add_party("did:example:provider2".to_string(), PartyRole::DataProvider).unwrap();
        contract.add_party("did:example:consumer".to_string(), PartyRole::DataConsumer).unwrap();
        contract.add_party("did:example:infra".to_string(), PartyRole::InfrastructureProvider).unwrap();

        // Sign contract with all parties
        assert!(contract.sign("did:example:provider1", "sig1".to_string()).is_ok());
        assert_eq!(contract.status, ContractStatus::PendingSignatures);

        assert!(contract.sign("did:example:provider2", "sig2".to_string()).is_ok());
        assert_eq!(contract.status, ContractStatus::PendingSignatures);

        assert!(contract.sign("did:example:consumer", "sig3".to_string()).is_ok());
        assert_eq!(contract.status, ContractStatus::PendingSignatures);

        assert!(contract.sign("did:example:infra", "sig4".to_string()).is_ok());
        assert_eq!(contract.status, ContractStatus::Active);
    }
} 