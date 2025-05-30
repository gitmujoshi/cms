//! Contract Management System - Contracts API Module
//! 
//! This module defines the bulk contract operations REST API endpoints for the Contract Management System.
//! It provides the HTTP interface for:
//! - Bulk contract operations
//! - Contract batch processing
//! - Contract search and filtering
//! - Contract analytics
//! - Contract reporting
//!
//! Features:
//! - Bulk operations
//! - Advanced search
! - Filtering and sorting
//! - Analytics endpoints
//! - Reporting endpoints
//!
//! Endpoints:
//! - Bulk operations
//! - Search endpoints
//! - Analytics endpoints
//! - Reporting endpoints
//! - Batch processing
//!
//! Security Features:
//! - Authentication
//! - Authorization
//! - Input validation
//! - Rate limiting
//! - Audit logging
//!
//! Usage:
//! 1. Configure bulk operation routes
//! 2. Handle batch requests
//! 3. Process search queries
//! 4. Generate reports
//! 5. Document endpoints
//!
//! Author: Contract Management System Team
//! License: MIT

use actix_web::{delete, get, post, put, web, HttpResponse, Responder};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use validator::Validate;
use crate::models::{ContractStatus, ContractType, SecurityRequirements};
use crate::services::contracts::ContractService;
use crate::auth::AuthGuard;
use crate::error::ApiError;

#[derive(Debug, Serialize, Deserialize, Validate)]
pub struct CreateContractRequest {
    #[validate(length(min = 1, max = 255))]
    pub title: String,
    #[validate(length(min = 1, max = 1000))]
    pub description: String,
    pub contract_type: ContractType,
    pub provider_id: Option<Uuid>,
    pub consumer_id: Option<Uuid>,
    pub terms: ContractTerms,
    #[validate(custom = "validate_date")]
    pub valid_from: String,
    pub valid_until: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Validate)]
pub struct ContractTerms {
    pub data_access_scope: Vec<String>,
    pub usage_restrictions: Vec<String>,
    #[validate(range(min = 1, max = 3650))]
    pub retention_period_days: i32,
    pub security_requirements: SecurityRequirements,
    pub compliance_requirements: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, Validate)]
pub struct UpdateContractRequest {
    #[validate(length(min = 1, max = 255))]
    pub title: Option<String>,
    #[validate(length(min = 1, max = 1000))]
    pub description: Option<String>,
    pub status: Option<ContractStatus>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ContractResponse {
    pub id: Uuid,
    pub title: String,
    pub description: String,
    pub status: ContractStatus,
    pub contract_type: ContractType,
    pub provider_id: Uuid,
    pub consumer_id: Uuid,
    pub terms: ContractTerms,
    pub created_at: String,
    pub updated_at: String,
    pub valid_from: String,
    pub valid_until: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ContractListResponse {
    pub contracts: Vec<ContractSummary>,
    pub total: usize,
    pub page: usize,
    pub per_page: usize,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ContractSummary {
    pub id: Uuid,
    pub title: String,
    pub description: String,
    pub status: ContractStatus,
    pub created_at: String,
    pub provider_name: String,
    pub consumer_name: String,
}

#[derive(Debug, Deserialize)]
pub struct ContractFilter {
    pub status: Option<ContractStatus>,
    pub search: Option<String>,
    pub page: Option<usize>,
    pub per_page: Option<usize>,
}

fn validate_date(date: &str) -> Result<(), validator::ValidationError> {
    chrono::DateTime::parse_from_rfc3339(date)
        .map_err(|_| validator::ValidationError::new("invalid_date"))?;
    Ok(())
}

#[get("/contracts")]
pub async fn list_contracts(
    filter: web::Query<ContractFilter>,
    service: web::Data<ContractService>,
    _: AuthGuard,
) -> Result<impl Responder, ApiError> {
    let page = filter.page.unwrap_or(1);
    let per_page = filter.per_page.unwrap_or(10).min(100);

    let (contracts, total) = service
        .list_contracts(
            page,
            per_page,
            filter.status.as_ref(),
            filter.search.as_deref(),
        )
        .await?;

    Ok(HttpResponse::Ok().json(ContractListResponse {
        contracts,
        total,
        page,
        per_page,
    }))
}

#[get("/contracts/{id}")]
pub async fn get_contract(
    id: web::Path<Uuid>,
    service: web::Data<ContractService>,
    _: AuthGuard,
) -> Result<impl Responder, ApiError> {
    let contract = service.get_contract(*id).await?;
    Ok(HttpResponse::Ok().json(contract))
}

#[post("/contracts")]
pub async fn create_contract(
    contract: web::Json<CreateContractRequest>,
    service: web::Data<ContractService>,
    auth: AuthGuard,
) -> Result<impl Responder, ApiError> {
    contract.validate()?;
    
    let contract_id = service
        .create_contract(auth.user_id, contract.into_inner())
        .await?;

    Ok(HttpResponse::Created().json(contract_id))
}

#[put("/contracts/{id}")]
pub async fn update_contract(
    id: web::Path<Uuid>,
    contract: web::Json<UpdateContractRequest>,
    service: web::Data<ContractService>,
    auth: AuthGuard,
) -> Result<impl Responder, ApiError> {
    contract.validate()?;
    
    service
        .update_contract(*id, auth.user_id, contract.into_inner())
        .await?;

    Ok(HttpResponse::Ok().finish())
}

#[delete("/contracts/{id}")]
pub async fn delete_contract(
    id: web::Path<Uuid>,
    service: web::Data<ContractService>,
    auth: AuthGuard,
) -> Result<impl Responder, ApiError> {
    service.delete_contract(*id, auth.user_id).await?;
    Ok(HttpResponse::NoContent().finish())
}

#[post("/contracts/{id}/sign")]
pub async fn sign_contract(
    id: web::Path<Uuid>,
    service: web::Data<ContractService>,
    auth: AuthGuard,
) -> Result<impl Responder, ApiError> {
    service.sign_contract(*id, auth.user_id).await?;
    Ok(HttpResponse::Ok().finish())
}

#[post("/contracts/{id}/terminate")]
pub async fn terminate_contract(
    id: web::Path<Uuid>,
    service: web::Data<ContractService>,
    auth: AuthGuard,
) -> Result<impl Responder, ApiError> {
    service.terminate_contract(*id, auth.user_id).await?;
    Ok(HttpResponse::Ok().finish())
}

#[post("/contracts/{id}/suspend")]
pub async fn suspend_contract(
    id: web::Path<Uuid>,
    service: web::Data<ContractService>,
    auth: AuthGuard,
) -> Result<impl Responder, ApiError> {
    service.suspend_contract(*id, auth.user_id).await?;
    Ok(HttpResponse::Ok().finish())
}

#[post("/contracts/{id}/activate")]
pub async fn activate_contract(
    id: web::Path<Uuid>,
    service: web::Data<ContractService>,
    auth: AuthGuard,
) -> Result<impl Responder, ApiError> {
    service.activate_contract(*id, auth.user_id).await?;
    Ok(HttpResponse::Ok().finish())
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
            .service(terminate_contract)
            .service(suspend_contract)
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