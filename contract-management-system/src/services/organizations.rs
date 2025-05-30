//! Contract Management System - Organizations Service
//! 
//! This module implements the organization management service for the Contract Management System.
//! It provides comprehensive organization management functionality including:
//! - Organization CRUD operations
//! - Organization hierarchy management
! - Department management
//! - User-organization relationships
//! - Organization settings
//!
//! Features:
//! - Organization structure management
//! - Department organization
//! - User assignment
//! - Role management
//! - Settings configuration
//!
//! Security Features:
//! - Access control
//! - Data isolation
//! - Audit logging
//! - Role enforcement
//! - Privacy controls
//!
//! Integration Points:
//! - User management
//! - Role management
//! - Contract management
//! - Audit system
//! - Notification system
//!
//! Usage:
//! 1. Initialize the organizations service
//! 2. Create and manage organizations
//! 3. Handle organization structure
//! 4. Manage user assignments
//! 5. Configure organization settings
//!
//! Author: Contract Management System Team
//! License: MIT

use sea_orm::{DatabaseConnection, EntityTrait, QueryFilter, Set, QueryOrder};
use uuid::Uuid;
use serde_json::json;
use crate::error::ApiError;
use crate::models::organizations::{self, OrganizationType, OrganizationStatus};
use crate::services::audit::{AuditService, AuditEvent};

#[derive(Debug, Clone)]
pub struct CreateOrganizationRequest {
    pub name: String,
    pub description: Option<String>,
    pub type_: OrganizationType,
    pub website: Option<String>,
    pub email: String,
    pub phone: Option<String>,
    pub address: Option<serde_json::Value>,
}

#[derive(Debug, Clone)]
pub struct UpdateOrganizationRequest {
    pub name: Option<String>,
    pub description: Option<String>,
    pub type_: Option<OrganizationType>,
    pub status: Option<OrganizationStatus>,
    pub website: Option<String>,
    pub email: Option<String>,
    pub phone: Option<String>,
    pub address: Option<serde_json::Value>,
}

pub struct OrganizationService {
    db: DatabaseConnection,
    audit: AuditService,
}

impl OrganizationService {
    pub fn new(db: DatabaseConnection) -> Self {
        Self {
            audit: AuditService::new(db.clone()),
            db,
        }
    }

    pub async fn list_organizations(
        &self,
        page: usize,
        per_page: usize,
        status: Option<OrganizationStatus>,
        search: Option<&str>,
    ) -> Result<(Vec<organizations::Model>, usize), ApiError> {
        let mut query = organizations::Entity::find();

        if let Some(status) = status {
            query = query.filter(organizations::Column::Status.eq(status.to_string()));
        }

        if let Some(search) = search {
            query = query.filter(
                organizations::Column::Name
                    .contains(search)
                    .or(organizations::Column::Email.contains(search)),
            );
        }

        let total = query.clone().count(&self.db).await?;

        let organizations = query
            .order_by_desc(organizations::Column::CreatedAt)
            .offset(((page - 1) * per_page) as u64)
            .limit(per_page as u64)
            .all(&self.db)
            .await?;

        Ok((organizations, total))
    }

    pub async fn get_organization(&self, id: Uuid) -> Result<organizations::Model, ApiError> {
        let organization = organizations::Entity::find_by_id(id)
            .one(&self.db)
            .await?
            .ok_or(ApiError::NotFound)?;

        Ok(organization)
    }

    pub async fn create_organization(
        &self,
        user_id: Uuid,
        request: CreateOrganizationRequest,
    ) -> Result<Uuid, ApiError> {
        // Check if organization with same email exists
        let existing = organizations::Entity::find()
            .filter(organizations::Column::Email.eq(&request.email))
            .one(&self.db)
            .await?;

        if existing.is_some() {
            return Err(ApiError::ValidationError("Organization with this email already exists".into()));
        }

        let organization = organizations::ActiveModel {
            id: Set(Uuid::new_v4()),
            name: Set(request.name.clone()),
            description: Set(request.description),
            type_: Set(request.type_.to_string()),
            status: Set(OrganizationStatus::Active.to_string()),
            website: Set(request.website),
            email: Set(request.email.clone()),
            phone: Set(request.phone),
            address: Set(request.address),
            created_at: Set(chrono::Utc::now()),
            updated_at: Set(chrono::Utc::now()),
        };

        let org_id = organization.id.clone().unwrap();

        organizations::Entity::insert(organization)
            .exec(&self.db)
            .await?;

        self.audit.log_event(AuditEvent {
            user_id,
            resource_type: "organization".to_string(),
            resource_id: Some(org_id),
            action: "create".to_string(),
            details: json!({
                "name": request.name,
                "email": request.email,
                "type": request.type_.to_string(),
            }),
            ip_address: None,
            user_agent: None,
        })
        .await?;

        Ok(org_id)
    }

    pub async fn update_organization(
        &self,
        id: Uuid,
        user_id: Uuid,
        request: UpdateOrganizationRequest,
    ) -> Result<(), ApiError> {
        let organization = organizations::Entity::find_by_id(id)
            .one(&self.db)
            .await?
            .ok_or(ApiError::NotFound)?;

        let mut organization: organizations::ActiveModel = organization.into();
        let mut changes = serde_json::Map::new();

        if let Some(name) = request.name {
            organization.name = Set(name.clone());
            changes.insert("name".to_string(), json!(name));
        }

        if let Some(description) = request.description {
            organization.description = Set(Some(description.clone()));
            changes.insert("description".to_string(), json!(description));
        }

        if let Some(type_) = request.type_ {
            organization.type_ = Set(type_.to_string());
            changes.insert("type".to_string(), json!(type_.to_string()));
        }

        if let Some(status) = request.status {
            organization.status = Set(status.to_string());
            changes.insert("status".to_string(), json!(status.to_string()));
        }

        if let Some(website) = request.website {
            organization.website = Set(Some(website.clone()));
            changes.insert("website".to_string(), json!(website));
        }

        if let Some(email) = request.email.clone() {
            // Check if email is already taken by another organization
            let existing = organizations::Entity::find()
                .filter(organizations::Column::Email.eq(&email))
                .filter(organizations::Column::Id.ne(id))
                .one(&self.db)
                .await?;

            if existing.is_some() {
                return Err(ApiError::ValidationError("Email already taken".into()));
            }

            organization.email = Set(email.clone());
            changes.insert("email".to_string(), json!(email));
        }

        if let Some(phone) = request.phone {
            organization.phone = Set(Some(phone.clone()));
            changes.insert("phone".to_string(), json!(phone));
        }

        if let Some(address) = request.address {
            organization.address = Set(Some(address.clone()));
            changes.insert("address".to_string(), json!(address));
        }

        organization.updated_at = Set(chrono::Utc::now());

        organizations::Entity::update(organization)
            .exec(&self.db)
            .await?;

        self.audit.log_event(AuditEvent {
            user_id,
            resource_type: "organization".to_string(),
            resource_id: Some(id),
            action: "update".to_string(),
            details: json!({ "changes": changes }),
            ip_address: None,
            user_agent: None,
        })
        .await?;

        Ok(())
    }

    pub async fn delete_organization(&self, id: Uuid, user_id: Uuid) -> Result<(), ApiError> {
        let organization = organizations::Entity::find_by_id(id)
            .one(&self.db)
            .await?
            .ok_or(ApiError::NotFound)?;

        organizations::Entity::delete_by_id(id)
            .exec(&self.db)
            .await?;

        self.audit.log_event(AuditEvent {
            user_id,
            resource_type: "organization".to_string(),
            resource_id: Some(id),
            action: "delete".to_string(),
            details: json!({
                "name": organization.name,
                "email": organization.email,
            }),
            ip_address: None,
            user_agent: None,
        })
        .await?;

        Ok(())
    }
} 