use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use sea_orm::entity::prelude::*;

use crate::auth::did::SignatureProof;

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

#[derive(Debug, Clone, PartialEq, EnumIter, DeriveActiveEnum, Serialize, Deserialize)]
#[sea_orm(rs_type = "String", db_type = "String(Some(1))")]
pub enum ContractStatus {
    #[sea_orm(string_value = "D")]
    Draft,
    #[sea_orm(string_value = "P")]
    PendingSignatures,
    #[sea_orm(string_value = "A")]
    Active,
    #[sea_orm(string_value = "S")]
    Suspended,
    #[sea_orm(string_value = "T")]
    Terminated,
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