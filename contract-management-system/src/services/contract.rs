use std::sync::Arc;
use chrono::{DateTime, Utc};
use sea_orm::*;
use uuid::Uuid;

use crate::auth::did::{DIDVerifier, SignatureProof};
use crate::models::contract::{self, ActiveModel, ContractSignature, Model as Contract, SignatureType};
use crate::error::Result;
use crate::services::ledger::{LedgerService, ContractEvent};
use crate::models::signature::SignatureModel;
use crate::error::AppError;

pub struct ContractService {
    db: DatabaseConnection,
    did_verifier: Arc<DIDVerifier>,
    ledger: LedgerService,
}

impl ContractService {
    pub fn new(db: DatabaseConnection, did_verifier: Arc<DIDVerifier>, ledger: LedgerService) -> Self {
        Self { db, did_verifier, ledger }
    }

    pub async fn create_contract(
        &self,
        title: String,
        description: String,
        provider_did: String,
        consumer_did: String,
        terms: String,
        valid_from: DateTime<Utc>,
        valid_until: Option<DateTime<Utc>>,
    ) -> Result<Contract> {
        let contract = Contract::new(
            title,
            description,
            provider_did,
            consumer_did,
            terms,
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
            title, description, terms, valid_from, valid_until.map(|d| d.to_string()).unwrap_or_default()
        );
        let content_hash = self.ledger.calculate_content_hash(&content);

        // Record on blockchain
        self.ledger.record_event(ContractEvent::Created {
            contract_id: contract.id,
            provider_did: provider_did.clone(),
            consumer_did: consumer_did.clone(),
            content_hash,
        }).await.map_err(|e| AppError::BlockchainError(e.to_string()))?;

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
        self.ledger.record_event(ContractEvent::Signed {
            contract_id,
            signer_did,
            signature,
            timestamp: Utc::now().timestamp() as u64,
        }).await.map_err(|e| AppError::BlockchainError(e.to_string()))?;

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

        for signature in &contract.signatures {
            let is_valid = self.did_verifier
                .verify_signature(
                    &signature.signer_did,
                    &message,
                    &signature.signature,
                    &signature.verification_method,
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
        self.ledger.record_event(ContractEvent::Updated {
            contract_id,
            content_hash,
            updater_did,
        }).await.map_err(|e| AppError::BlockchainError(e.to_string()))?;

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
        self.ledger.record_event(ContractEvent::Voided {
            contract_id,
            reason,
            voider_did,
        }).await.map_err(|e| AppError::BlockchainError(e.to_string()))?;

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
}

#[cfg(test)]
mod tests {
    use super::*;
    use sea_orm::DatabaseBackend;
    use sea_orm::MockDatabase;

    #[tokio::test]
    async fn test_create_contract() {
        let db = MockDatabase::new(DatabaseBackend::Postgres)
            .append_query_results(vec![vec![Contract::new(
                "Test Contract".to_string(),
                "Description".to_string(),
                "did:example:123".to_string(),
                "did:example:456".to_string(),
                "Terms".to_string(),
                Utc::now(),
                None,
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

        let service = ContractService::new(db, did_verifier, ledger);

        let result = service
            .create_contract(
                "Test Contract".to_string(),
                "Description".to_string(),
                "did:example:123".to_string(),
                "did:example:456".to_string(),
                "Terms".to_string(),
                Utc::now(),
                None,
            )
            .await;

        assert!(result.is_ok());
        let contract = result.unwrap();
        assert_eq!(contract.title, "Test Contract");
        assert_eq!(contract.provider_did, "did:example:123");
        assert_eq!(contract.consumer_did, "did:example:456");
    }
} 