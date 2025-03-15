use sea_orm::{DatabaseConnection, EntityTrait, QueryFilter, Set, QueryOrder, QuerySelect, JoinType};
use uuid::Uuid;
use serde_json::json;
use crate::error::ApiError;
use crate::models::{contracts, ContractStatus, ContractType};
use crate::models::{users, organizations};
use crate::api::contracts::{CreateContractRequest, UpdateContractRequest, ContractResponse, ContractSummary};
use crate::models::contract_signatures;
use crate::services::audit::{AuditService, AuditEvent};

pub struct ContractService {
    db: DatabaseConnection,
    audit: AuditService,
}

impl ContractService {
    pub fn new(db: DatabaseConnection) -> Self {
        Self {
            audit: AuditService::new(db.clone()),
            db,
        }
    }

    pub async fn list_contracts(
        &self,
        page: usize,
        per_page: usize,
        status: Option<&ContractStatus>,
        search: Option<&str>,
    ) -> Result<(Vec<ContractSummary>, usize), ApiError> {
        let mut query = contracts::Entity::find()
            .join(
                JoinType::LeftJoin,
                contracts::Relation::Provider.def().on_condition(|_left, right| {
                    Expr::col((users::Entity, users::Column::Id))
                        .eq(right.column(users::Column::Id))
                }),
            )
            .join(
                JoinType::LeftJoin,
                contracts::Relation::Consumer.def().on_condition(|_left, right| {
                    Expr::col((users::Entity, users::Column::Id))
                        .eq(right.column(users::Column::Id))
                }),
            )
            .join(
                JoinType::LeftJoin,
                users::Relation::Organization.def().on_condition(|_left, right| {
                    Expr::col((organizations::Entity, organizations::Column::Id))
                        .eq(right.column(organizations::Column::Id))
                }),
            );

        if let Some(status) = status {
            query = query.filter(contracts::Column::Status.eq(status));
        }

        if let Some(search) = search {
            query = query.filter(
                contracts::Column::Title
                    .contains(search)
                    .or(contracts::Column::Description.contains(search)),
            );
        }

        let total = query.clone().count(&self.db).await?;

        let contracts = query
            .order_by_desc(contracts::Column::CreatedAt)
            .offset(((page - 1) * per_page) as u64)
            .limit(per_page as u64)
            .all(&self.db)
            .await?;

        let mut summaries = Vec::with_capacity(contracts.len());
        for contract in contracts {
            let provider = users::Entity::find_by_id(contract.provider_id)
                .one(&self.db)
                .await?;
            let consumer = users::Entity::find_by_id(contract.consumer_id)
                .one(&self.db)
                .await?;

            let provider_org = if let Some(ref provider) = provider {
                organizations::Entity::find_by_id(provider.organization_id)
                    .one(&self.db)
                    .await?
            } else {
                None
            };

            let consumer_org = if let Some(ref consumer) = consumer {
                organizations::Entity::find_by_id(consumer.organization_id)
                    .one(&self.db)
                    .await?
            } else {
                None
            };

            summaries.push(ContractSummary {
                id: contract.id,
                title: contract.title,
                description: contract.description,
                status: contract.status,
                created_at: contract.created_at.to_rfc3339(),
                provider_name: provider
                    .as_ref()
                    .map(|p| format!("{} ({}) - {}", p.full_name(), provider_org.as_ref().map(|o| o.name.as_str()).unwrap_or("Unknown Org"), p.email))
                    .unwrap_or_else(|| "Unknown Provider".to_string()),
                consumer_name: consumer
                    .as_ref()
                    .map(|c| format!("{} ({}) - {}", c.full_name(), consumer_org.as_ref().map(|o| o.name.as_str()).unwrap_or("Unknown Org"), c.email))
                    .unwrap_or_else(|| "Unknown Consumer".to_string()),
            });
        }

        Ok((summaries, total))
    }

    pub async fn get_contract(&self, id: Uuid) -> Result<ContractResponse, ApiError> {
        let contract = contracts::Entity::find_by_id(id)
            .one(&self.db)
            .await?
            .ok_or(ApiError::NotFound)?;

        Ok(ContractResponse {
            id: contract.id,
            title: contract.title,
            description: contract.description,
            status: contract.status,
            contract_type: contract.contract_type,
            provider_id: contract.provider_id,
            consumer_id: contract.consumer_id,
            terms: serde_json::from_value(contract.terms)?,
            created_at: contract.created_at.to_rfc3339(),
            updated_at: contract.updated_at.to_rfc3339(),
            valid_from: contract.valid_from.to_rfc3339(),
            valid_until: contract.valid_until.map(|d| d.to_rfc3339()),
        })
    }

    pub async fn create_contract(
        &self,
        user_id: Uuid,
        request: CreateContractRequest,
    ) -> Result<Uuid, ApiError> {
        let contract = contracts::ActiveModel {
            id: Set(Uuid::new_v4()),
            title: Set(request.title.clone()),
            description: Set(request.description.clone()),
            contract_type: Set(request.contract_type.clone()),
            provider_id: Set(request.provider_id.unwrap_or(user_id)),
            consumer_id: Set(request.consumer_id.ok_or(ApiError::ValidationError("consumer_id is required".into()))?),
            terms: Set(serde_json::to_value(&request.terms)?),
            status: Set(ContractStatus::Draft),
            valid_from: Set(chrono::DateTime::parse_from_rfc3339(&request.valid_from)?.into()),
            valid_until: Set(request.valid_until.map(|d| chrono::DateTime::parse_from_rfc3339(&d).unwrap().into())),
            created_at: Set(chrono::Utc::now()),
            updated_at: Set(chrono::Utc::now()),
        };

        let contract_id = contract.id.clone().unwrap();
        
        contracts::Entity::insert(contract)
            .exec(&self.db)
            .await?;

        // Log audit event
        self.audit.log_event(AuditEvent {
            user_id,
            resource_type: "contract".to_string(),
            resource_id: Some(contract_id),
            action: "create".to_string(),
            details: json!({
                "title": request.title,
                "contract_type": request.contract_type,
                "provider_id": request.provider_id,
                "consumer_id": request.consumer_id,
            }),
            ip_address: None, // TODO: Pass from API layer
            user_agent: None, // TODO: Pass from API layer
        })
        .await?;

        Ok(contract_id)
    }

    pub async fn update_contract(
        &self,
        id: Uuid,
        user_id: Uuid,
        request: UpdateContractRequest,
    ) -> Result<(), ApiError> {
        let contract = contracts::Entity::find_by_id(id)
            .one(&self.db)
            .await?
            .ok_or(ApiError::NotFound)?;

        if contract.provider_id != user_id && contract.consumer_id != user_id {
            return Err(ApiError::Forbidden);
        }

        let mut contract: contracts::ActiveModel = contract.into();
        let mut changes = serde_json::Map::new();

        if let Some(title) = request.title.clone() {
            contract.title = Set(title.clone());
            changes.insert("title".to_string(), json!(title));
        }

        if let Some(description) = request.description.clone() {
            contract.description = Set(description.clone());
            changes.insert("description".to_string(), json!(description));
        }

        if let Some(status) = request.status.clone() {
            contract.status = Set(status.clone());
            changes.insert("status".to_string(), json!(status));
        }

        contract.updated_at = Set(chrono::Utc::now());

        contracts::Entity::update(contract)
            .exec(&self.db)
            .await?;

        // Log audit event
        self.audit.log_event(AuditEvent {
            user_id,
            resource_type: "contract".to_string(),
            resource_id: Some(id),
            action: "update".to_string(),
            details: json!({ "changes": changes }),
            ip_address: None,
            user_agent: None,
        })
        .await?;

        Ok(())
    }

    pub async fn delete_contract(&self, id: Uuid, user_id: Uuid) -> Result<(), ApiError> {
        let contract = contracts::Entity::find_by_id(id)
            .one(&self.db)
            .await?
            .ok_or(ApiError::NotFound)?;

        if contract.provider_id != user_id {
            return Err(ApiError::Forbidden);
        }

        if contract.status != ContractStatus::Draft {
            return Err(ApiError::InvalidContractState);
        }

        contracts::Entity::delete_by_id(id)
            .exec(&self.db)
            .await?;

        // Log audit event
        self.audit.log_event(AuditEvent {
            user_id,
            resource_type: "contract".to_string(),
            resource_id: Some(id),
            action: "delete".to_string(),
            details: json!({
                "title": contract.title,
                "status": contract.status,
            }),
            ip_address: None,
            user_agent: None,
        })
        .await?;

        Ok(())
    }

    pub async fn sign_contract(&self, id: Uuid, user_id: Uuid) -> Result<(), ApiError> {
        let contract = contracts::Entity::find_by_id(id)
            .one(&self.db)
            .await?
            .ok_or(ApiError::NotFound)?;

        if contract.provider_id != user_id && contract.consumer_id != user_id {
            return Err(ApiError::Forbidden);
        }

        if contract.status != ContractStatus::Draft && contract.status != ContractStatus::PendingSignature {
            return Err(ApiError::InvalidContractState);
        }

        let existing_signature = contract_signatures::Entity::find()
            .filter(
                contract_signatures::Column::ContractId.eq(id)
                    .and(contract_signatures::Column::SignerId.eq(user_id))
            )
            .one(&self.db)
            .await?;

        if existing_signature.is_some() {
            return Err(ApiError::InvalidContractState);
        }

        let signature = contract_signatures::ActiveModel {
            id: Set(Uuid::new_v4()),
            contract_id: Set(id),
            signer_id: Set(user_id),
            signature_type: Set("digital".to_string()),
            signature: Set("signed".to_string()),
            signed_at: Set(chrono::Utc::now()),
        };

        contract_signatures::Entity::insert(signature)
            .exec(&self.db)
            .await?;

        let signatures_count = contract_signatures::Entity::find()
            .filter(contract_signatures::Column::ContractId.eq(id))
            .count(&self.db)
            .await?;

        let new_status = if signatures_count == 1 {
            ContractStatus::PendingSignature
        } else {
            ContractStatus::Active
        };

        let mut contract: contracts::ActiveModel = contract.into();
        contract.status = Set(new_status.clone());
        contract.updated_at = Set(chrono::Utc::now());

        contracts::Entity::update(contract)
            .exec(&self.db)
            .await?;

        // Log audit event
        self.audit.log_event(AuditEvent {
            user_id,
            resource_type: "contract".to_string(),
            resource_id: Some(id),
            action: "sign".to_string(),
            details: json!({
                "new_status": new_status,
                "signatures_count": signatures_count,
            }),
            ip_address: None,
            user_agent: None,
        })
        .await?;

        Ok(())
    }

    pub async fn terminate_contract(&self, id: Uuid, user_id: Uuid) -> Result<(), ApiError> {
        let contract = contracts::Entity::find_by_id(id)
            .one(&self.db)
            .await?
            .ok_or(ApiError::NotFound)?;

        if contract.provider_id != user_id && contract.consumer_id != user_id {
            return Err(ApiError::Forbidden);
        }

        if contract.status != ContractStatus::Active {
            return Err(ApiError::InvalidContractState);
        }

        let mut contract: contracts::ActiveModel = contract.into();
        contract.status = Set(ContractStatus::Terminated);
        contract.updated_at = Set(chrono::Utc::now());

        contracts::Entity::update(contract)
            .exec(&self.db)
            .await?;

        // Log audit event
        self.audit.log_event(AuditEvent {
            user_id,
            resource_type: "contract".to_string(),
            resource_id: Some(id),
            action: "terminate".to_string(),
            details: json!({
                "title": contract.title,
                "status": contract.status,
            }),
            ip_address: None,
            user_agent: None,
        })
        .await?;

        Ok(())
    }

    pub async fn suspend_contract(&self, id: Uuid, user_id: Uuid) -> Result<(), ApiError> {
        let contract = contracts::Entity::find_by_id(id)
            .one(&self.db)
            .await?
            .ok_or(ApiError::NotFound)?;

        if contract.provider_id != user_id && contract.consumer_id != user_id {
            return Err(ApiError::Forbidden);
        }

        if contract.status != ContractStatus::Active {
            return Err(ApiError::InvalidContractState);
        }

        let mut contract: contracts::ActiveModel = contract.into();
        contract.status = Set(ContractStatus::Suspended);
        contract.updated_at = Set(chrono::Utc::now());

        contracts::Entity::update(contract)
            .exec(&self.db)
            .await?;

        Ok(())
    }

    pub async fn activate_contract(&self, id: Uuid, user_id: Uuid) -> Result<(), ApiError> {
        let contract = contracts::Entity::find_by_id(id)
            .one(&self.db)
            .await?
            .ok_or(ApiError::NotFound)?;

        // Verify user has permission
        if contract.provider_id != user_id && contract.consumer_id != user_id {
            return Err(ApiError::Forbidden);
        }

        // Can only activate suspended contracts
        if contract.status != ContractStatus::Suspended {
            return Err(ApiError::InvalidContractState);
        }

        let mut contract: contracts::ActiveModel = contract.into();
        contract.status = Set(ContractStatus::Active);
        contract.updated_at = Set(chrono::Utc::now());

        contracts::Entity::update(contract)
            .exec(&self.db)
            .await?;

        Ok(())
    }
}
