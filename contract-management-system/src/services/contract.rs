use std::sync::Arc;
use chrono::{DateTime, Utc};
use sea_orm::*;
use uuid::Uuid;

use crate::auth::did::{DIDVerifier, SignatureProof};
use crate::models::contract::{self, ActiveModel, ContractSignature, Model as Contract, SignatureType, ContractParty, PartyRole, ContractStatus, DataProcessingTerms, InfrastructureRequirements};
use crate::error::Result;
use crate::services::ledger::{LedgerService, ContractEvent};
use crate::models::signature::SignatureModel;
use crate::error::AppError;
use crate::services::blockchain::BlockchainService;
use crate::services::enclave::EnclaveService;
use crate::error::ServiceError;

pub struct ContractService {
    db: DatabaseConnection,
    did_verifier: Arc<DIDVerifier>,
    ledger: LedgerService,
    blockchain_service: BlockchainService,
    enclave_service: EnclaveService,
}

impl ContractService {
    pub fn new(db: DatabaseConnection, did_verifier: Arc<DIDVerifier>, ledger: LedgerService, blockchain_service: BlockchainService, enclave_service: EnclaveService) -> Self {
        Self { db, did_verifier, ledger, blockchain_service, enclave_service }
    }

    pub async fn create_contract(
        &self,
        title: String,
        description: String,
        processing_terms: DataProcessingTerms,
        infrastructure_requirements: InfrastructureRequirements,
        valid_from: DateTime<Utc>,
        valid_until: DateTime<Utc>,
    ) -> Result<Contract> {
        let contract = Contract::new(
            title,
            description,
            processing_terms,
            infrastructure_requirements,
            valid_from,
            valid_until,
        );

        let model: ActiveModel = contract.clone().into();
        let result = contract::Entity::insert(model)
            .exec(&self.db)
            .await?;

        // Calculate content hash
        let content = format!(
            "Title: {}\nDescription: {}\nTerms: {}\nValid From: {}\nValid Until: {}",
            title, description, processing_terms, valid_from, valid_until.to_string()
        );
        let content_hash = self.ledger.calculate_content_hash(&content);

        // Record on blockchain
        self.blockchain_service.record_contract_creation(&contract).await?;

        Ok(contract)
    }

    pub async fn sign_contract(
        &self,
        contract_id: Uuid,
        signer_did: String,
        signature: String,
        verification_method: String,
    ) -> Result<Contract> {
        // Get contract
        let mut contract = self.get_contract(contract_id).await?;

        // Verify signer is authorized
        if signer_did != contract.provider_did && signer_did != contract.consumer_did {
            return Err(anyhow::anyhow!("Unauthorized signer").into());
        }

        // Generate signing message
        let message = contract.generate_signing_message();

        // Verify signature
        let is_valid = self.did_verifier
            .verify_signature(
                &signer_did,
                &message,
                &signature,
                &verification_method,
            )
            .await
            .map_err(|e| anyhow::anyhow!("Signature verification failed: {}", e))?;

        if !is_valid {
            return Err(anyhow::anyhow!("Invalid signature").into());
        }

        // Create signature record
        let signature = ContractSignature {
            id: Uuid::new_v4(),
            contract_id,
            signer_did: signer_did.clone(),
            signature,
            signature_type: SignatureType::Ed25519, // TODO: Determine from verification method
            verification_method,
            signed_at: Utc::now(),
            proof: SignatureProof {
                type_: "Ed25519Signature2020".to_string(),
                created: Utc::now(),
                verification_method: format!("{}#keys-1", signer_did),
                proof_purpose: "contractSigning".to_string(),
                proof_value: signature,
            },
        };

        // Add signature to contract
        contract.add_signature(signature)
            .map_err(|e| anyhow::anyhow!(e))?;

        // Update contract in database
        let model: ActiveModel = contract.clone().into();
        contract::Entity::update(model)
            .exec(&self.db)
            .await?;

        // Record signature on blockchain
        self.blockchain_service.record_signature(contract_id, &signer_did, &signature.signature).await?;

        Ok(contract)
    }

    pub async fn get_contract(&self, id: Uuid) -> Result<Contract> {
        let contract = contract::Entity::find_by_id(id)
            .one(&self.db)
            .await?
            .ok_or_else(|| anyhow::anyhow!("Contract not found"))?;

        Ok(contract)
    }

    pub async fn list_contracts(&self, signer_did: Option<String>) -> Result<Vec<Contract>> {
        let mut query = contract::Entity::find();

        if let Some(did) = signer_did {
            query = query.filter(
                Condition::any()
                    .add(contract::Column::ProviderDid.eq(did.clone()))
                    .add(contract::Column::ConsumerDid.eq(did))
            );
        }

        let contracts = query
            .order_by_desc(contract::Column::CreatedAt)
            .all(&self.db)
            .await?;

        Ok(contracts)
    }

    pub async fn verify_contract_signatures(&self, contract_id: Uuid) -> Result<bool> {
        let contract = self.get_contract(contract_id).await?;
        let message = contract.generate_signing_message();

        // Verify all required parties have signed
        let data_providers = contract.parties.iter().filter(|p| p.role == PartyRole::DataProvider);
        let data_consumer = contract.parties.iter().find(|p| p.role == PartyRole::DataConsumer);
        let infra_provider = contract.parties.iter().find(|p| p.role == PartyRole::InfrastructureProvider);

        // Verify data consumer signature
        if let Some(consumer) = data_consumer {
            if consumer.signature.is_none() {
                return Ok(false);
            }
        } else {
            return Ok(false);
        }

        // Verify infrastructure provider signature
        if let Some(provider) = infra_provider {
            if provider.signature.is_none() {
                return Ok(false);
            }
        } else {
            return Ok(false);
        }

        // Verify all data provider signatures
        for provider in data_providers {
            if provider.signature.is_none() {
                return Ok(false);
            }

            // Verify signature cryptographically
            let is_valid = self.did_verifier
                .verify_signature(
                    &provider.did,
                    &message,
                    provider.signature.as_ref().unwrap(),
                    &format!("{}#keys-1", provider.did),
                )
                .await
                .map_err(|e| anyhow::anyhow!("Signature verification failed: {}", e))?;

            if !is_valid {
                return Ok(false);
            }
        }

        Ok(true)
    }

    pub async fn update_contract(
        &self,
        contract_id: Uuid,
        title: Option<String>,
        description: Option<String>,
        terms: Option<String>,
        updater_did: String,
    ) -> Result<Contract> {
        // Update contract in database
        let contract = contract::Entity::find_by_id(contract_id)
            .one(&self.db)
            .await?
            .ok_or_else(|| anyhow::anyhow!("Contract not found"))?;

        let mut updated_contract = contract.clone();
        if let Some(title) = title {
            updated_contract.title = title;
        }
        if let Some(description) = description {
            updated_contract.description = description;
        }
        if let Some(terms) = terms {
            updated_contract.terms = terms;
        }

        let model: ActiveModel = updated_contract.into();
        contract::Entity::update(model)
            .exec(&self.db)
            .await?;

        // Calculate new content hash
        let content = format!(
            "Title: {}\nDescription: {}\nTerms: {}",
            updated_contract.title, updated_contract.description, updated_contract.terms
        );
        let content_hash = self.ledger.calculate_content_hash(&content);

        // Record update on blockchain
        self.blockchain_service.record_contract_update(contract_id, content_hash).await?;

        Ok(updated_contract)
    }

    pub async fn void_contract(
        &self,
        contract_id: Uuid,
        reason: String,
        voider_did: String,
    ) -> Result<Contract> {
        // Void contract in database
        let contract = contract::Entity::find_by_id(contract_id)
            .one(&self.db)
            .await?
            .ok_or_else(|| anyhow::anyhow!("Contract not found"))?;

        let mut updated_contract = contract.clone();
        updated_contract.status = "voided".to_string();

        let model: ActiveModel = updated_contract.into();
        contract::Entity::update(model)
            .exec(&self.db)
            .await?;

        // Record void on blockchain
        self.blockchain_service.record_termination(contract_id, &voider_did, &reason).await?;

        Ok(updated_contract)
    }

    pub async fn verify_signatures(
        &self,
        contract_id: Uuid,
    ) -> Result<bool> {
        // Get contract from database
        let contract = self.get_contract(contract_id).await?;
        
        // Get blockchain events
        let events = self.ledger
            .get_contract_events(contract_id)
            .await
            .map_err(|e| AppError::BlockchainError(e.to_string()))?;

        // Verify number of signatures matches
        let db_signatures = SignatureModel::find_by_contract(&self.db, contract_id).await?;
        let blockchain_signatures = events.iter()
            .filter(|(event_type, _, _)| event_type == "SIGNED")
            .count();

        if db_signatures.len() != blockchain_signatures {
            return Ok(false);
        }

        // Verify each signature exists on blockchain
        for db_sig in db_signatures {
            let matching_event = events.iter().any(|(event_type, data, _)| {
                if event_type != "SIGNED" {
                    return false;
                }
                
                if let Ok(event_data) = serde_json::from_str::<serde_json::Value>(data) {
                    event_data["signer_did"] == db_sig.signer_did &&
                    event_data["signature"] == db_sig.signature
                } else {
                    false
                }
            });

            if !matching_event {
                return Ok(false);
            }
        }

        Ok(true)
    }

    pub async fn get_contract_history(
        &self,
        contract_id: Uuid,
    ) -> Result<Vec<(String, String, u64)>> {
        self.ledger
            .get_contract_events(contract_id)
            .await
            .map_err(|e| AppError::BlockchainError(e.to_string()))
    }

    pub async fn add_party(
        &self,
        contract: &mut Contract,
        did: String,
        role: PartyRole,
    ) -> Result<(), ServiceError> {
        // Validate party role
        match role {
            PartyRole::DataProvider => {
                // Verify data provider's credentials and data ownership
                self.verify_data_provider_credentials(&did).await?;
            },
            PartyRole::DataConsumer => {
                // Verify consumer's credentials and permissions
                self.verify_data_consumer_credentials(&did).await?;
            },
            PartyRole::InfrastructureProvider => {
                // Verify infrastructure provider's capabilities
                self.verify_infrastructure_provider(&did, &contract.infrastructure_requirements).await?;
            },
        }

        contract.add_party(did, role)
            .map_err(|e| ServiceError::ValidationError(e))
    }

    async fn verify_data_provider_credentials(&self, did: &str) -> Result<(), ServiceError> {
        // Verify data provider's credentials and data ownership
        let provider_info = self.did_verifier
            .resolve_did(did)
            .await
            .map_err(|e| ServiceError::ValidationError(format!("Failed to resolve DID: {}", e)))?;

        // Verify data ownership proofs
        let ownership_proof = provider_info
            .service
            .iter()
            .find(|s| s.type_ == "DataOwnershipProof")
            .ok_or_else(|| ServiceError::ValidationError("Missing data ownership proof".to_string()))?;

        // Verify compliance certifications
        let certifications = provider_info
            .service
            .iter()
            .filter(|s| s.type_ == "Certification")
            .collect::<Vec<_>>();

        if certifications.is_empty() {
            return Err(ServiceError::ValidationError("Missing required certifications".to_string()));
        }

        Ok(())
    }

    async fn verify_data_consumer_credentials(&self, did: &str) -> Result<(), ServiceError> {
        // Implement verification logic for data consumer
        // - Check access permissions
        // - Verify consumer's identity
        // - Check compliance requirements
        Ok(())
    }

    async fn verify_infrastructure_provider(
        &self,
        did: &str,
        requirements: &InfrastructureRequirements,
    ) -> Result<(), ServiceError> {
        // Verify the infrastructure provider can meet requirements
        self.enclave_service
            .verify_provider_capabilities(did, requirements)
            .await
            .map_err(|e| ServiceError::ValidationError(e.to_string()))
    }

    async fn initialize_enclave(&self, contract: &Contract) -> Result<(), ServiceError> {
        // Get infrastructure provider
        let infra_provider = contract.parties
            .iter()
            .find(|p| p.role == PartyRole::InfrastructureProvider)
            .ok_or_else(|| ServiceError::ValidationError("Missing infrastructure provider".to_string()))?;

        // Get data providers
        let data_providers = contract.parties
            .iter()
            .filter(|p| p.role == PartyRole::DataProvider)
            .collect::<Vec<_>>();

        if data_providers.is_empty() {
            return Err(ServiceError::ValidationError("No data providers found".to_string()));
        }

        // Initialize enclave with all data providers
        self.enclave_service
            .initialize_enclave_with_providers(
                contract.id,
                &infra_provider.did,
                &data_providers.iter().map(|p| p.did.as_str()).collect::<Vec<_>>(),
                &contract.infrastructure_requirements,
            )
            .await
            .map_err(|e| ServiceError::EnclaveError(e.to_string()))?;

        // Set up data access controls for each provider
        for provider in data_providers {
            self.enclave_service
                .configure_data_access(
                    contract.id,
                    &provider.did,
                    &contract.processing_terms.access_controls,
                )
                .await
                .map_err(|e| ServiceError::EnclaveError(format!("Failed to configure access for {}: {}", provider.did, e)))?;
        }

        Ok(())
    }

    async fn verify_signature(&self, did: &str, signature: &str) -> Result<(), ServiceError> {
        // Implement signature verification logic
        // - Verify DID ownership
        // - Check signature validity
        // - Verify timestamp
        Ok(())
    }

    pub async fn terminate_contract(
        &self,
        contract: &mut Contract,
        reason: String,
        terminator_did: &str,
    ) -> Result<(), ServiceError> {
        // Verify terminator is a party to the contract
        if !contract.parties.iter().any(|p| p.did == terminator_did) {
            return Err(ServiceError::AuthorizationError("Not authorized to terminate contract".to_string()));
        }

        // Record termination on blockchain
        self.blockchain_service.record_termination(contract.id, terminator_did, &reason).await?;

        // Terminate enclave
        self.enclave_service.terminate_enclave(contract.id).await?;

        contract.terminate(reason);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sea_orm::DatabaseBackend;
    use sea_orm::MockDatabase;
    use mockall::predicate::*;
    use mockall::mock;

    mock! {
        BlockchainService {
            fn record_contract_creation(&self, contract: &Contract) -> Result<String, ServiceError>;
            fn record_signature(&self, id: Uuid, did: &str, signature: &str) -> Result<(), ServiceError>;
            fn record_termination(&self, id: Uuid, terminator: &str, reason: &str) -> Result<(), ServiceError>;
        }
    }

    mock! {
        EnclaveService {
            fn verify_provider_capabilities(&self, did: &str, requirements: &InfrastructureRequirements) -> Result<(), String>;
            fn initialize_enclave(&self, id: Uuid, provider: &str, requirements: &InfrastructureRequirements) -> Result<(), String>;
            fn terminate_enclave(&self, id: Uuid) -> Result<(), String>;
            fn initialize_enclave_with_providers(&self, id: Uuid, provider: &str, providers: &[&str], requirements: &InfrastructureRequirements) -> Result<(), String>;
            fn configure_data_access(&self, id: Uuid, provider: &str, access_controls: &[&str]) -> Result<(), String>;
        }
    }

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

    #[tokio::test]
    async fn test_contract_creation() {
        let db = MockDatabase::new(DatabaseBackend::Postgres)
            .append_query_results(vec![vec![Contract::new(
                "Test Contract".to_string(),
                "Test Description".to_string(),
                DataProcessingTerms {
                    data_description: "Test data".to_string(),
                    allowed_operations: vec!["analyze".to_string()],
                    retention_period_days: 30,
                    access_controls: vec!["encryption".to_string()],
                    security_requirements: vec!["attestation".to_string()],
                },
                InfrastructureRequirements {
                    enclave_type: "AWS Nitro".to_string(),
                    attestation_type: "DCE".to_string(),
                    security_level: "High".to_string(),
                    certifications: vec!["ISO27001".to_string()],
                    performance_requirements: vec!["4 vCPUs".to_string()],
                },
                Utc::now(),
                Utc::now() + chrono::Duration::days(30),
            )]])
            .into_connection();

        let did_verifier = Arc::new(DIDVerifier::new(Arc::new(crate::auth::did::MultiResolver::new(
            chrono::Duration::minutes(5),
        ))));

        let ledger = LedgerService::new(
            "http://localhost:8545",
            "0x1234567890123456789012345678901234567890",
            "0x1234567890123456789012345678901234567890123456789012345678901234",
        ).await.unwrap();

        let mock_blockchain = MockBlockchainService::new();
        let mock_enclave = MockEnclaveService::new();

        let service = ContractService::new(db, did_verifier, ledger, mock_blockchain, mock_enclave);

        let result = service.create_contract(
            "Test Contract".to_string(),
            "Test Description".to_string(),
            DataProcessingTerms {
                data_description: "Test data".to_string(),
                allowed_operations: vec!["analyze".to_string()],
                retention_period_days: 30,
                access_controls: vec!["encryption".to_string()],
                security_requirements: vec!["attestation".to_string()],
            },
            InfrastructureRequirements {
                enclave_type: "AWS Nitro".to_string(),
                attestation_type: "DCE".to_string(),
                security_level: "High".to_string(),
                certifications: vec!["ISO27001".to_string()],
                performance_requirements: vec!["4 vCPUs".to_string()],
            },
            Utc::now(),
            Utc::now() + chrono::Duration::days(30),
        ).await;

        assert!(result.is_ok());
        let contract = result.unwrap();
        assert_eq!(contract.title, "Test Contract");
        assert_eq!(contract.provider_did, "did:example:123");
        assert_eq!(contract.consumer_did, "did:example:456");
    }

    #[tokio::test]
    async fn test_contract_signing_flow() {
        let mut mock_blockchain = MockBlockchainService::new();
        mock_blockchain
            .expect_record_signature()
            .times(3)
            .return_const(Ok(()));

        let mut mock_enclave = MockEnclaveService::new();
        mock_enclave
            .expect_initialize_enclave()
            .return_once(|_, _, _| Ok(()));

        let service = ContractService::new(
            MockDatabase::new(DatabaseBackend::Postgres).into_connection(),
            Arc::new(DIDVerifier::new(Arc::new(crate::auth::did::MultiResolver::new(
                chrono::Duration::minutes(5),
            )))),
            LedgerService::new(
                "http://localhost:8545",
                "0x1234567890123456789012345678901234567890",
                "0x1234567890123456789012345678901234567890123456789012345678901234",
            ).await.unwrap(),
            mock_blockchain,
            mock_enclave,
        );
        let mut contract = create_test_contract();

        // Add parties
        service.add_party(&mut contract, "did:example:provider".to_string(), PartyRole::DataProvider).await.unwrap();
        service.add_party(&mut contract, "did:example:consumer".to_string(), PartyRole::DataConsumer).await.unwrap();
        service.add_party(&mut contract, "did:example:infra".to_string(), PartyRole::InfrastructureProvider).await.unwrap();

        // Sign contract
        service.sign_contract(contract.id, "did:example:provider".to_string(), "sig1".to_string(), "Ed25519Signature2020".to_string()).await.unwrap();
        service.sign_contract(contract.id, "did:example:consumer".to_string(), "sig2".to_string(), "Ed25519Signature2020".to_string()).await.unwrap();
        service.sign_contract(contract.id, "did:example:infra".to_string(), "sig3".to_string(), "Ed25519Signature2020".to_string()).await.unwrap();

        assert_eq!(contract.status, ContractStatus::Active);
    }

    #[tokio::test]
    async fn test_multiple_providers_initialization() {
        let mut mock_blockchain = MockBlockchainService::new();
        let mut mock_enclave = MockEnclaveService::new();

        // Expect enclave initialization with multiple providers
        mock_enclave
            .expect_initialize_enclave_with_providers()
            .with(
                predicate::always(),
                predicate::eq("did:example:infra"),
                predicate::function(|providers: &[&str]| providers.len() == 2),
                predicate::always(),
            )
            .times(1)
            .return_once(|_, _, _, _| Ok(()));

        // Expect data access configuration for each provider
        mock_enclave
            .expect_configure_data_access()
            .times(2)
            .return_const(Ok(()));

        let service = ContractService::new(
            MockDatabase::new(DatabaseBackend::Postgres).into_connection(),
            Arc::new(DIDVerifier::new(Arc::new(crate::auth::did::MultiResolver::new(
                chrono::Duration::minutes(5),
            )))),
            LedgerService::new(
                "http://localhost:8545",
                "0x1234567890123456789012345678901234567890",
                "0x1234567890123456789012345678901234567890123456789012345678901234",
            ).await.unwrap(),
            mock_blockchain,
            mock_enclave,
        );

        let mut contract = create_test_contract();

        // Add multiple data providers
        service.add_party(&mut contract, "did:example:provider1".to_string(), PartyRole::DataProvider).await.unwrap();
        service.add_party(&mut contract, "did:example:provider2".to_string(), PartyRole::DataProvider).await.unwrap();
        service.add_party(&mut contract, "did:example:consumer".to_string(), PartyRole::DataConsumer).await.unwrap();
        service.add_party(&mut contract, "did:example:infra".to_string(), PartyRole::InfrastructureProvider).await.unwrap();

        // Initialize enclave
        assert!(service.initialize_enclave(&contract).await.is_ok());
    }
} 