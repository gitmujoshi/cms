use actix_web::{delete, get, post, put, web, HttpResponse, Responder};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use anyhow::{Context, Result};
use sea_orm::{DatabaseConnection, EntityTrait, QueryFilter, Set};
use crate::models::contracts::{self, Entity as Contract};
use crate::models::contract_signatures::{self, Entity as ContractSignature};
use crate::iam::audit::{AuditEvent, AuditEventType};

use crate::models::{Contract, ContractStatus, ContractType};

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateContractRequest {
    pub title: String,
    pub description: String,
    pub contract_type: ContractType,
    pub parties: Vec<PartyInput>,
    pub terms: ContractTermsInput,
    pub valid_from: chrono::DateTime<chrono::Utc>,
    pub valid_until: Option<chrono::DateTime<chrono::Utc>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PartyInput {
    pub name: String,
    pub role: String,
    pub contact_info: ContactInfoInput,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ContactInfoInput {
    pub email: String,
    pub phone: Option<String>,
    pub organization: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ContractTermsInput {
    pub data_usage_terms: DataUsageTermsInput,
    pub security_requirements: SecurityRequirementsInput,
    pub compliance_requirements: Vec<ComplianceRequirementInput>,
    pub model_training_terms: ModelTrainingTermsInput,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DataUsageTermsInput {
    pub allowed_purposes: Vec<String>,
    pub usage_restrictions: Vec<String>,
    pub retention_period: i32,
    pub data_handling_requirements: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SecurityRequirementsInput {
    pub encryption_level: String,
    pub access_controls: Vec<String>,
    pub audit_requirements: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ComplianceRequirementInput {
    pub regulation: String,
    pub requirements: Vec<String>,
    pub verification_method: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ModelTrainingTermsInput {
    pub allowed_algorithms: Vec<String>,
    pub performance_requirements: Vec<String>,
    pub resource_limits: ResourceLimitsInput,
    pub output_restrictions: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ResourceLimitsInput {
    pub max_cpu_cores: i32,
    pub max_memory_gb: i32,
    pub max_training_time_hours: i32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UpdateContractRequest {
    pub title: Option<String>,
    pub description: Option<String>,
    pub status: Option<ContractStatus>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ContractData {
    pub id: Option<Uuid>,
    pub title: String,
    pub description: String,
    pub provider_id: Uuid,
    pub consumer_id: Uuid,
    pub terms: String,
    pub status: ContractStatus,
    pub valid_from: chrono::DateTime<chrono::Utc>,
    pub valid_until: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub enum ContractStatus {
    Draft,
    PendingSignature,
    Active,
    Completed,
    Terminated,
}

pub struct ContractService {
    db: DatabaseConnection,
}

impl ContractService {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }

    pub async fn get_contracts(&self, filters: ContractFilters) -> Result<Vec<ContractData>> {
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
            .context("Failed to fetch contracts")?;

        Ok(contracts
            .into_iter()
            .map(|c| ContractData {
                id: Some(c.id),
                title: c.title,
                description: c.description,
                provider_id: c.provider_id,
                consumer_id: c.consumer_id,
                terms: c.terms,
                status: c.status,
                valid_from: c.valid_from,
                valid_until: c.valid_until,
            })
            .collect())
    }

    pub async fn get_contract(&self, id: Uuid) -> Result<Option<ContractData>> {
        let contract = Contract::find_by_id(id)
            .one(&self.db)
            .await
            .context("Failed to fetch contract")?;

        Ok(contract.map(|c| ContractData {
            id: Some(c.id),
            title: c.title,
            description: c.description,
            provider_id: c.provider_id,
            consumer_id: c.consumer_id,
            terms: c.terms,
            status: c.status,
            valid_from: c.valid_from,
            valid_until: c.valid_until,
        }))
    }

    pub async fn create_contract(&self, data: ContractData) -> Result<Uuid> {
        let contract = contracts::ActiveModel {
            id: Set(Uuid::new_v4()),
            title: Set(data.title),
            description: Set(data.description),
            provider_id: Set(data.provider_id),
            consumer_id: Set(data.consumer_id),
            terms: Set(data.terms),
            status: Set(ContractStatus::Draft),
            valid_from: Set(data.valid_from),
            valid_until: Set(data.valid_until),
            created_at: Set(chrono::Utc::now()),
            updated_at: Set(chrono::Utc::now()),
        };

        let result = Contract::insert(contract)
            .exec(&self.db)
            .await
            .context("Failed to create contract")?;

        // Log audit event
        AuditEvent::new(
            AuditEventType::ContractCreation,
            Some(result.last_insert_id),
            "Contract created successfully",
        )
        .log()
        .await?;

        Ok(result.last_insert_id)
    }

    pub async fn update_contract(&self, id: Uuid, data: ContractData) -> Result<()> {
        let contract = Contract::find_by_id(id)
            .one(&self.db)
            .await
            .context("Failed to fetch contract")?
            .ok_or_else(|| anyhow::anyhow!("Contract not found"))?;

        let mut contract: contracts::ActiveModel = contract.into();

        contract.title = Set(data.title);
        contract.description = Set(data.description);
        contract.terms = Set(data.terms);
        contract.valid_from = Set(data.valid_from);
        contract.valid_until = Set(data.valid_until);
        contract.updated_at = Set(chrono::Utc::now());

        Contract::update(contract)
            .exec(&self.db)
            .await
            .context("Failed to update contract")?;

        // Log audit event
        AuditEvent::new(
            AuditEventType::ContractUpdate,
            Some(id),
            "Contract updated successfully",
        )
        .log()
        .await?;

        Ok(())
    }

    pub async fn delete_contract(&self, id: Uuid) -> Result<()> {
        Contract::delete_by_id(id)
            .exec(&self.db)
            .await
            .context("Failed to delete contract")?;

        // Log audit event
        AuditEvent::new(
            AuditEventType::ContractDeletion,
            Some(id),
            "Contract deleted successfully",
        )
        .log()
        .await?;

        Ok(())
    }

    pub async fn sign_contract(&self, id: Uuid, signer_id: Uuid, signature: String) -> Result<()> {
        let contract = Contract::find_by_id(id)
            .one(&self.db)
            .await
            .context("Failed to fetch contract")?
            .ok_or_else(|| anyhow::anyhow!("Contract not found"))?;

        // Verify contract is in signable state
        if contract.status != ContractStatus::PendingSignature {
            return Err(anyhow::anyhow!("Contract is not in signable state"));
        }

        // Create signature record
        let signature = contract_signatures::ActiveModel {
            id: Set(Uuid::new_v4()),
            contract_id: Set(id),
            signer_id: Set(signer_id),
            signature: Set(signature),
            signed_at: Set(chrono::Utc::now()),
        };

        ContractSignature::insert(signature)
            .exec(&self.db)
            .await
            .context("Failed to save contract signature")?;

        // Check if all required signatures are collected
        let signatures = ContractSignature::find()
            .filter(contract_signatures::Column::ContractId.eq(id))
            .all(&self.db)
            .await
            .context("Failed to fetch signatures")?;

        // If both provider and consumer have signed, activate the contract
        if signatures.len() == 2 {
            self.activate_contract(id).await?;
        }

        // Log audit event
        AuditEvent::new(
            AuditEventType::ContractSigning,
            Some(id),
            "Contract signed successfully",
        )
        .log()
        .await?;

        Ok(())
    }

    async fn activate_contract(&self, id: Uuid) -> Result<()> {
        let mut contract = Contract::find_by_id(id)
            .one(&self.db)
            .await
            .context("Failed to fetch contract")?
            .ok_or_else(|| anyhow::anyhow!("Contract not found"))?;

        let mut active_model: contracts::ActiveModel = contract.clone().into();
        active_model.status = Set(ContractStatus::Active);
        active_model.updated_at = Set(chrono::Utc::now());

        Contract::update(active_model)
            .exec(&self.db)
            .await
            .context("Failed to activate contract")?;

        // Log audit event
        AuditEvent::new(
            AuditEventType::ContractActivation,
            Some(id),
            "Contract activated successfully",
        )
        .log()
        .await?;

        Ok(())
    }
}

#[derive(Debug, Default)]
pub struct ContractFilters {
    pub provider_id: Option<Uuid>,
    pub consumer_id: Option<Uuid>,
    pub status: Option<ContractStatus>,
}

#[get("/contracts")]
pub async fn list_contracts() -> impl Responder {
    // TODO: Implement database query
    HttpResponse::Ok().json(vec![])
}

#[get("/contracts/{id}")]
pub async fn get_contract(id: web::Path<Uuid>) -> impl Responder {
    // TODO: Implement database query
    HttpResponse::NotFound().finish()
}

#[post("/contracts")]
pub async fn create_contract(contract: web::Json<CreateContractRequest>) -> impl Responder {
    // TODO: Implement contract creation
    HttpResponse::Created().json(contract.0)
}

#[put("/contracts/{id}")]
pub async fn update_contract(
    id: web::Path<Uuid>,
    contract: web::Json<UpdateContractRequest>,
) -> impl Responder {
    // TODO: Implement contract update
    HttpResponse::Ok().json(contract.0)
}

#[delete("/contracts/{id}")]
pub async fn delete_contract(id: web::Path<Uuid>) -> impl Responder {
    // TODO: Implement contract deletion
    HttpResponse::NoContent().finish()
}

#[post("/contracts/{id}/sign")]
pub async fn sign_contract(id: web::Path<Uuid>) -> impl Responder {
    // TODO: Implement contract signing
    HttpResponse::Ok().finish()
}

#[post("/contracts/{id}/activate")]
pub async fn activate_contract(id: web::Path<Uuid>) -> impl Responder {
    // TODO: Implement contract activation
    HttpResponse::Ok().finish()
}

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/api/v1")
            .service(list_contracts)
            .service(get_contract)
            .service(create_contract)
            .service(update_contract)
            .service(delete_contract)
            .service(sign_contract)
            .service(activate_contract),
    );
}

#[cfg(test)]
mod tests {
    use super::*;
    use sea_orm::{DatabaseBackend, MockDatabase, MockExecResult};

    #[tokio::test]
    async fn test_create_contract() {
        let db = MockDatabase::new(DatabaseBackend::Postgres)
            .append_query_results(vec![vec![contracts::Model {
                id: Uuid::new_v4(),
                title: "Test Contract".to_string(),
                description: "Test Description".to_string(),
                provider_id: Uuid::new_v4(),
                consumer_id: Uuid::new_v4(),
                terms: "Test Terms".to_string(),
                status: ContractStatus::Draft,
                valid_from: chrono::Utc::now(),
                valid_until: chrono::Utc::now(),
                created_at: chrono::Utc::now(),
                updated_at: chrono::Utc::now(),
            }]])
            .append_exec_results(vec![MockExecResult {
                last_insert_id: 1,
                rows_affected: 1,
            }])
            .into_connection();

        let service = ContractService::new(db);
        let result = service
            .create_contract(ContractData {
                id: None,
                title: "Test Contract".to_string(),
                description: "Test Description".to_string(),
                provider_id: Uuid::new_v4(),
                consumer_id: Uuid::new_v4(),
                terms: "Test Terms".to_string(),
                status: ContractStatus::Draft,
                valid_from: chrono::Utc::now(),
                valid_until: chrono::Utc::now(),
            })
            .await;

        assert!(result.is_ok());
    }

    // Add more tests for other contract operations
} 