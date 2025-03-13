use anyhow::{Context, Result};
use sea_orm::{DatabaseConnection, EntityTrait, Set};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use crate::iam::{Identity, Permission};
use crate::models::contracts::{self, Entity as Contract};
use crate::audit::AuditService;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContractData {
    pub id: Option<Uuid>,
    pub title: String,
    pub description: String,
    pub provider_id: Uuid,
    pub consumer_id: Uuid,
    pub status: ContractStatus,
    pub terms: ContractTerms,
    pub signatures: Vec<ContractSignature>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContractTerms {
    pub data_access_scope: DataAccessScope,
    pub usage_restrictions: Vec<String>,
    pub retention_period: chrono::Duration,
    pub security_requirements: SecurityRequirements,
    pub compliance_requirements: Vec<String>,
    pub pricing: Option<PricingTerms>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityRequirements {
    pub encryption_required: bool,
    pub min_encryption_level: EncryptionLevel,
    pub audit_logging_required: bool,
    pub network_isolation_required: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EncryptionLevel {
    Standard,
    High,
    Military,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PricingTerms {
    pub currency: String,
    pub amount: f64,
    pub billing_period: BillingPeriod,
    pub payment_terms: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BillingPeriod {
    Monthly,
    Quarterly,
    Annually,
    OneTime,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataAccessScope {
    pub allowed_data_types: Vec<String>,
    pub allowed_operations: Vec<String>,
    pub geographic_restrictions: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContractSignature {
    pub signer_id: Uuid,
    pub signature_type: SignatureType,
    pub signature_data: String,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SignatureType {
    Digital,
    Biometric,
    MultiFactor,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ContractStatus {
    Draft,
    PendingSignature,
    Active,
    Suspended,
    Terminated,
    Expired,
}

pub struct ContractService {
    db: DatabaseConnection,
    audit: AuditService,
}

impl ContractService {
    pub fn new(db: DatabaseConnection, audit: AuditService) -> Self {
        Self { db, audit }
    }

    pub async fn create_contract(&self, creator_id: Uuid, data: ContractData) -> Result<ContractData> {
        let contract = contracts::ActiveModel {
            id: Set(data.id.unwrap_or_else(Uuid::new_v4)),
            title: Set(data.title.clone()),
            description: Set(data.description.clone()),
            provider_id: Set(data.provider_id),
            consumer_id: Set(data.consumer_id),
            status: Set(ContractStatus::Draft),
            terms: Set(serde_json::to_value(&data.terms)?),
            signatures: Set(serde_json::to_value(&data.signatures)?),
            created_at: Set(chrono::Utc::now()),
            updated_at: Set(chrono::Utc::now()),
        };

        let result = Contract::insert(contract)
            .exec(&self.db)
            .await
            .context("Failed to create contract")?;

        self.audit.log_event(crate::audit::AuditEventData {
            id: None,
            event_type: crate::audit::AuditEventType::ContractCreation,
            identity_id: Some(creator_id),
            resource_type: Some("contract".to_string()),
            resource_id: Some(result.last_insert_id),
            action: "create".to_string(),
            status: crate::audit::AuditEventStatus::Success,
            error_message: None,
            request_metadata: serde_json::json!({
                "title": data.title,
                "provider_id": data.provider_id,
                "consumer_id": data.consumer_id,
            }),
            timestamp: chrono::Utc::now(),
        })
        .await?;

        Ok(data)
    }

    pub async fn update_contract(
        &self,
        updater_id: Uuid,
        contract_id: Uuid,
        data: ContractData,
    ) -> Result<ContractData> {
        let contract = contracts::ActiveModel {
            id: Set(contract_id),
            title: Set(data.title.clone()),
            description: Set(data.description.clone()),
            provider_id: Set(data.provider_id),
            consumer_id: Set(data.consumer_id),
            status: Set(data.status),
            terms: Set(serde_json::to_value(&data.terms)?),
            signatures: Set(serde_json::to_value(&data.signatures)?),
            updated_at: Set(chrono::Utc::now()),
            ..Default::default()
        };

        Contract::update(contract)
            .exec(&self.db)
            .await
            .context("Failed to update contract")?;

        self.audit.log_event(crate::audit::AuditEventData {
            id: None,
            event_type: crate::audit::AuditEventType::ContractUpdate,
            identity_id: Some(updater_id),
            resource_type: Some("contract".to_string()),
            resource_id: Some(contract_id),
            action: "update".to_string(),
            status: crate::audit::AuditEventStatus::Success,
            error_message: None,
            request_metadata: serde_json::json!({
                "title": data.title,
                "status": format!("{:?}", data.status),
            }),
            timestamp: chrono::Utc::now(),
        })
        .await?;

        Ok(data)
    }

    pub async fn sign_contract(
        &self,
        signer_id: Uuid,
        contract_id: Uuid,
        signature: ContractSignature,
    ) -> Result<()> {
        let mut contract = Contract::find_by_id(contract_id)
            .one(&self.db)
            .await
            .context("Failed to find contract")?
            .ok_or_else(|| anyhow::anyhow!("Contract not found"))?;

        let mut signatures: Vec<ContractSignature> =
            serde_json::from_value(contract.signatures.clone())?;
        signatures.push(signature.clone());

        let update = contracts::ActiveModel {
            id: Set(contract_id),
            signatures: Set(serde_json::to_value(&signatures)?),
            status: Set(ContractStatus::Active),
            updated_at: Set(chrono::Utc::now()),
            ..Default::default()
        };

        Contract::update(update)
            .exec(&self.db)
            .await
            .context("Failed to update contract signatures")?;

        self.audit.log_event(crate::audit::AuditEventData {
            id: None,
            event_type: crate::audit::AuditEventType::ContractSigning,
            identity_id: Some(signer_id),
            resource_type: Some("contract".to_string()),
            resource_id: Some(contract_id),
            action: "sign".to_string(),
            status: crate::audit::AuditEventStatus::Success,
            error_message: None,
            request_metadata: serde_json::json!({
                "signature_type": format!("{:?}", signature.signature_type),
                "timestamp": signature.timestamp,
            }),
            timestamp: chrono::Utc::now(),
        })
        .await?;

        Ok(())
    }

    pub async fn get_contract(&self, contract_id: Uuid) -> Result<Option<ContractData>> {
        let contract = Contract::find_by_id(contract_id)
            .one(&self.db)
            .await
            .context("Failed to find contract")?;

        Ok(contract.map(|c| ContractData {
            id: Some(c.id),
            title: c.title,
            description: c.description,
            provider_id: c.provider_id,
            consumer_id: c.consumer_id,
            status: c.status,
            terms: serde_json::from_value(c.terms).unwrap(),
            signatures: serde_json::from_value(c.signatures).unwrap(),
            created_at: c.created_at,
            updated_at: c.updated_at,
        }))
    }

    pub async fn list_contracts(&self, filters: ContractFilters) -> Result<Vec<ContractData>> {
        let mut query = Contract::find();

        if let Some(provider_id) = filters.provider_id {
            query = query.filter(contracts::Column::ProviderId.eq(provider_id));
        }

        if let Some(consumer_id) = filters.consumer_id {
            query = query.filter(contracts::Column::ConsumerId.eq(consumer_id));
        }

        if let Some(status) = filters.status {
            query = query.filter(contracts::Column::Status.eq(status));
        }

        let contracts = query
            .all(&self.db)
            .await
            .context("Failed to list contracts")?;

        Ok(contracts
            .into_iter()
            .map(|c| ContractData {
                id: Some(c.id),
                title: c.title,
                description: c.description,
                provider_id: c.provider_id,
                consumer_id: c.consumer_id,
                status: c.status,
                terms: serde_json::from_value(c.terms).unwrap(),
                signatures: serde_json::from_value(c.signatures).unwrap(),
                created_at: c.created_at,
                updated_at: c.updated_at,
            })
            .collect())
    }
}

#[derive(Debug, Default)]
pub struct ContractFilters {
    pub provider_id: Option<Uuid>,
    pub consumer_id: Option<Uuid>,
    pub status: Option<ContractStatus>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use sea_orm::{DatabaseBackend, MockDatabase, MockExecResult};
    use std::sync::Once;

    static INIT: Once = Once::new();

    async fn setup_test_db() -> DatabaseConnection {
        INIT.call_once(|| {
            let _ = env_logger::builder().is_test(true).try_init();
        });

        let db = MockDatabase::new(DatabaseBackend::Postgres)
            .append_query_results(vec![
                vec![contracts::Model {
                    id: Uuid::new_v4(),
                    title: "Test Contract".to_string(),
                    description: "Test Description".to_string(),
                    provider_id: Uuid::new_v4(),
                    consumer_id: Uuid::new_v4(),
                    status: ContractStatus::Draft,
                    terms: serde_json::json!({}),
                    signatures: serde_json::json!([]),
                    created_at: chrono::Utc::now(),
                    updated_at: chrono::Utc::now(),
                }],
            ])
            .append_exec_results(vec![MockExecResult {
                last_insert_id: 1,
                rows_affected: 1,
            }])
            .into_connection();

        db
    }

    #[tokio::test]
    async fn test_contract_lifecycle() {
        let db = setup_test_db().await;
        let audit = AuditService::new(db.clone());
        let service = ContractService::new(db, audit);

        let contract_data = ContractData {
            id: None,
            title: "Test Contract".to_string(),
            description: "Test Description".to_string(),
            provider_id: Uuid::new_v4(),
            consumer_id: Uuid::new_v4(),
            status: ContractStatus::Draft,
            terms: ContractTerms {
                data_access_scope: DataAccessScope {
                    allowed_data_types: vec!["test".to_string()],
                    allowed_operations: vec!["read".to_string()],
                    geographic_restrictions: vec!["US".to_string()],
                },
                usage_restrictions: vec!["test".to_string()],
                retention_period: chrono::Duration::days(30),
                security_requirements: SecurityRequirements {
                    encryption_required: true,
                    min_encryption_level: EncryptionLevel::Standard,
                    audit_logging_required: true,
                    network_isolation_required: false,
                },
                compliance_requirements: vec!["GDPR".to_string()],
                pricing: None,
            },
            signatures: vec![],
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        };

        let result = service
            .create_contract(Uuid::new_v4(), contract_data.clone())
            .await;
        assert!(result.is_ok());

        let signature = ContractSignature {
            signer_id: Uuid::new_v4(),
            signature_type: SignatureType::Digital,
            signature_data: "test".to_string(),
            timestamp: chrono::Utc::now(),
        };

        let sign_result = service
            .sign_contract(signature.signer_id, result.unwrap().id.unwrap(), signature)
            .await;
        assert!(sign_result.is_ok());
    }
} 