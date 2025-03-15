use std::sync::Arc;
use chrono::{DateTime, Utc};
use sea_orm::*;
use uuid::Uuid;

use crate::auth::did::{DIDVerifier, SignatureProof};
use crate::models::contract::{self, ActiveModel, ContractSignature, Model as Contract, SignatureType};
use crate::error::Result;

pub struct ContractService {
    db: DatabaseConnection,
    did_verifier: Arc<DIDVerifier>,
}

impl ContractService {
    pub fn new(db: DatabaseConnection, did_verifier: Arc<DIDVerifier>) -> Self {
        Self { db, did_verifier }
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

        let service = ContractService::new(db, did_verifier);

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