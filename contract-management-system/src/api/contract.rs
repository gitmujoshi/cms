use actix_web::{web, HttpResponse, Scope};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::auth::did::SignatureProof;
use crate::services::contract::ContractService;
use crate::error::Result;

pub fn contract_routes() -> Scope {
    web::scope("/contracts")
        .service(create_contract)
        .service(sign_contract)
        .service(get_contract)
        .service(list_contracts)
        .service(verify_signatures)
}

#[derive(Debug, Deserialize)]
pub struct CreateContractRequest {
    pub title: String,
    pub description: String,
    pub provider_did: String,
    pub consumer_did: String,
    pub terms: String,
    pub valid_from: DateTime<Utc>,
    pub valid_until: Option<DateTime<Utc>>,
}

#[derive(Debug, Serialize)]
pub struct ContractResponse {
    pub id: Uuid,
    pub title: String,
    pub description: String,
    pub provider_did: String,
    pub consumer_did: String,
    pub terms: String,
    pub status: String,
    pub valid_from: DateTime<Utc>,
    pub valid_until: Option<DateTime<Utc>>,
    pub signatures: Vec<SignatureResponse>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize)]
pub struct SignatureResponse {
    pub id: Uuid,
    pub signer_did: String,
    pub signature_type: String,
    pub verification_method: String,
    pub signed_at: DateTime<Utc>,
    pub proof: SignatureProof,
}

#[derive(Debug, Deserialize)]
pub struct SignContractRequest {
    pub signer_did: String,
    pub signature: String,
    pub verification_method: String,
}

#[post("")]
async fn create_contract(
    service: web::Data<ContractService>,
    req: web::Json<CreateContractRequest>,
) -> Result<HttpResponse> {
    let contract = service
        .create_contract(
            req.title.clone(),
            req.description.clone(),
            req.provider_did.clone(),
            req.consumer_did.clone(),
            req.terms.clone(),
            req.valid_from,
            req.valid_until,
        )
        .await?;

    Ok(HttpResponse::Created().json(contract_to_response(&contract)))
}

#[post("/{id}/sign")]
async fn sign_contract(
    service: web::Data<ContractService>,
    path: web::Path<Uuid>,
    req: web::Json<SignContractRequest>,
) -> Result<HttpResponse> {
    let contract = service
        .sign_contract(
            path.into_inner(),
            req.signer_did.clone(),
            req.signature.clone(),
            req.verification_method.clone(),
        )
        .await?;

    Ok(HttpResponse::Ok().json(contract_to_response(&contract)))
}

#[get("/{id}")]
async fn get_contract(
    service: web::Data<ContractService>,
    path: web::Path<Uuid>,
) -> Result<HttpResponse> {
    let contract = service.get_contract(path.into_inner()).await?;
    Ok(HttpResponse::Ok().json(contract_to_response(&contract)))
}

#[get("")]
async fn list_contracts(
    service: web::Data<ContractService>,
    query: web::Query<ListContractsQuery>,
) -> Result<HttpResponse> {
    let contracts = service.list_contracts(query.signer_did.clone()).await?;
    let responses: Vec<_> = contracts.into_iter().map(contract_to_response).collect();
    Ok(HttpResponse::Ok().json(responses))
}

#[derive(Debug, Deserialize)]
pub struct ListContractsQuery {
    pub signer_did: Option<String>,
}

#[get("/{id}/verify")]
async fn verify_signatures(
    service: web::Data<ContractService>,
    path: web::Path<Uuid>,
) -> Result<HttpResponse> {
    let is_valid = service.verify_contract_signatures(path.into_inner()).await?;
    Ok(HttpResponse::Ok().json(json!({ "valid": is_valid })))
}

fn contract_to_response(contract: &crate::models::contract::Model) -> ContractResponse {
    ContractResponse {
        id: contract.id,
        title: contract.title.clone(),
        description: contract.description.clone(),
        provider_did: contract.provider_did.clone(),
        consumer_did: contract.consumer_did.clone(),
        terms: contract.terms.clone(),
        status: format!("{:?}", contract.status),
        valid_from: contract.valid_from,
        valid_until: contract.valid_until,
        signatures: contract.signatures.iter().map(|s| SignatureResponse {
            id: s.id,
            signer_did: s.signer_did.clone(),
            signature_type: format!("{:?}", s.signature_type),
            verification_method: s.verification_method.clone(),
            signed_at: s.signed_at,
            proof: s.proof.clone(),
        }).collect(),
        created_at: contract.created_at,
        updated_at: contract.updated_at,
    }
} 