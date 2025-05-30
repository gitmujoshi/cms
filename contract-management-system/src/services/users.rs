//! Contract Management System - Users Service
//! 
//! This module implements the user management service for the Contract Management System.
//! It provides comprehensive user management functionality including:
//! - User profile management
//! - Role management
//! - Permission control
//! - User authentication
//! - User session management
//!
//! Features:
//! - User CRUD operations
//! - Role-based access control
//! - Permission management
//! - Session handling
//! - Profile management
//!
//! Security Features:
//! - Password management
//! - Session security
//! - Access control
//! - Audit logging
//! - Data privacy
//!
//! Integration Points:
//! - Authentication system
//! - Role management
//! - Organization management
//! - Audit system
//! - Notification system
//!
//! Usage:
//! 1. Initialize the users service
//! 2. Manage user profiles
//! 3. Handle user roles and permissions
//! 4. Process user sessions
//! 5. Manage user authentication
//!
//! Author: Contract Management System Team
//! License: MIT

use sea_orm::{DatabaseConnection, EntityTrait, QueryFilter, Set, QueryOrder};
use uuid::Uuid;
use serde_json::json;
use argon2::{
    password_hash::{rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Argon2,
};
use crate::error::ApiError;
use crate::models::users::{self, UserRole, UserStatus};
use crate::services::audit::{AuditService, AuditEvent};

#[derive(Debug, Clone)]
pub struct CreateUserRequest {
    pub organization_id: Uuid,
    pub email: String,
    pub password: String,
    pub first_name: String,
    pub last_name: String,
    pub role: UserRole,
}

#[derive(Debug, Clone)]
pub struct UpdateUserRequest {
    pub email: Option<String>,
    pub password: Option<String>,
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub role: Option<UserRole>,
    pub status: Option<UserStatus>,
}

#[derive(Debug, Clone)]
pub struct AuthenticateRequest {
    pub email: String,
    pub password: String,
}

pub struct UserService {
    db: DatabaseConnection,
    audit: AuditService,
}

impl UserService {
    pub fn new(db: DatabaseConnection) -> Self {
        Self {
            audit: AuditService::new(db.clone()),
            db,
        }
    }

    pub async fn list_users(
        &self,
        page: usize,
        per_page: usize,
        organization_id: Option<Uuid>,
        status: Option<UserStatus>,
        search: Option<&str>,
    ) -> Result<(Vec<users::Model>, usize), ApiError> {
        let mut query = users::Entity::find();

        if let Some(org_id) = organization_id {
            query = query.filter(users::Column::OrganizationId.eq(org_id));
        }

        if let Some(status) = status {
            query = query.filter(users::Column::Status.eq(status.to_string()));
        }

        if let Some(search) = search {
            query = query.filter(
                users::Column::Email
                    .contains(search)
                    .or(users::Column::FirstName.contains(search))
                    .or(users::Column::LastName.contains(search)),
            );
        }

        let total = query.clone().count(&self.db).await?;

        let users = query
            .order_by_desc(users::Column::CreatedAt)
            .offset(((page - 1) * per_page) as u64)
            .limit(per_page as u64)
            .all(&self.db)
            .await?;

        Ok((users, total))
    }

    pub async fn get_user(&self, id: Uuid) -> Result<users::Model, ApiError> {
        let user = users::Entity::find_by_id(id)
            .one(&self.db)
            .await?
            .ok_or(ApiError::NotFound)?;

        Ok(user)
    }

    pub async fn create_user(
        &self,
        admin_user_id: Uuid,
        request: CreateUserRequest,
    ) -> Result<Uuid, ApiError> {
        // Check if user with same email exists
        let existing = users::Entity::find()
            .filter(users::Column::Email.eq(&request.email))
            .one(&self.db)
            .await?;

        if existing.is_some() {
            return Err(ApiError::ValidationError("User with this email already exists".into()));
        }

        // Hash password
        let salt = SaltString::generate(&mut OsRng);
        let argon2 = Argon2::default();
        let password_hash = argon2
            .hash_password(request.password.as_bytes(), &salt)?
            .to_string();

        let user = users::ActiveModel {
            id: Set(Uuid::new_v4()),
            organization_id: Set(request.organization_id),
            email: Set(request.email.clone()),
            password_hash: Set(password_hash),
            first_name: Set(request.first_name.clone()),
            last_name: Set(request.last_name.clone()),
            role: Set(request.role.to_string()),
            status: Set(UserStatus::Active.to_string()),
            last_login_at: Set(None),
            created_at: Set(chrono::Utc::now()),
            updated_at: Set(chrono::Utc::now()),
        };

        let user_id = user.id.clone().unwrap();

        users::Entity::insert(user)
            .exec(&self.db)
            .await?;

        self.audit.log_event(AuditEvent {
            user_id: admin_user_id,
            resource_type: "user".to_string(),
            resource_id: Some(user_id),
            action: "create".to_string(),
            details: json!({
                "email": request.email,
                "first_name": request.first_name,
                "last_name": request.last_name,
                "role": request.role.to_string(),
                "organization_id": request.organization_id,
            }),
            ip_address: None,
            user_agent: None,
        })
        .await?;

        Ok(user_id)
    }

    pub async fn update_user(
        &self,
        id: Uuid,
        admin_user_id: Uuid,
        request: UpdateUserRequest,
    ) -> Result<(), ApiError> {
        let user = users::Entity::find_by_id(id)
            .one(&self.db)
            .await?
            .ok_or(ApiError::NotFound)?;

        let mut user: users::ActiveModel = user.into();
        let mut changes = serde_json::Map::new();

        if let Some(email) = request.email.clone() {
            // Check if email is already taken by another user
            let existing = users::Entity::find()
                .filter(users::Column::Email.eq(&email))
                .filter(users::Column::Id.ne(id))
                .one(&self.db)
                .await?;

            if existing.is_some() {
                return Err(ApiError::ValidationError("Email already taken".into()));
            }

            user.email = Set(email.clone());
            changes.insert("email".to_string(), json!(email));
        }

        if let Some(password) = request.password {
            let salt = SaltString::generate(&mut OsRng);
            let argon2 = Argon2::default();
            let password_hash = argon2
                .hash_password(password.as_bytes(), &salt)?
                .to_string();

            user.password_hash = Set(password_hash);
            changes.insert("password".to_string(), json!("changed"));
        }

        if let Some(first_name) = request.first_name {
            user.first_name = Set(first_name.clone());
            changes.insert("first_name".to_string(), json!(first_name));
        }

        if let Some(last_name) = request.last_name {
            user.last_name = Set(last_name.clone());
            changes.insert("last_name".to_string(), json!(last_name));
        }

        if let Some(role) = request.role {
            user.role = Set(role.to_string());
            changes.insert("role".to_string(), json!(role.to_string()));
        }

        if let Some(status) = request.status {
            user.status = Set(status.to_string());
            changes.insert("status".to_string(), json!(status.to_string()));
        }

        user.updated_at = Set(chrono::Utc::now());

        users::Entity::update(user)
            .exec(&self.db)
            .await?;

        self.audit.log_event(AuditEvent {
            user_id: admin_user_id,
            resource_type: "user".to_string(),
            resource_id: Some(id),
            action: "update".to_string(),
            details: json!({ "changes": changes }),
            ip_address: None,
            user_agent: None,
        })
        .await?;

        Ok(())
    }

    pub async fn delete_user(&self, id: Uuid, admin_user_id: Uuid) -> Result<(), ApiError> {
        let user = users::Entity::find_by_id(id)
            .one(&self.db)
            .await?
            .ok_or(ApiError::NotFound)?;

        users::Entity::delete_by_id(id)
            .exec(&self.db)
            .await?;

        self.audit.log_event(AuditEvent {
            user_id: admin_user_id,
            resource_type: "user".to_string(),
            resource_id: Some(id),
            action: "delete".to_string(),
            details: json!({
                "email": user.email,
                "name": format!("{} {}", user.first_name, user.last_name),
            }),
            ip_address: None,
            user_agent: None,
        })
        .await?;

        Ok(())
    }

    pub async fn authenticate(&self, request: AuthenticateRequest) -> Result<users::Model, ApiError> {
        let user = users::Entity::find()
            .filter(users::Column::Email.eq(&request.email))
            .one(&self.db)
            .await?
            .ok_or(ApiError::InvalidCredentials)?;

        if !user.is_active() {
            return Err(ApiError::AccountInactive);
        }

        let parsed_hash = PasswordHash::new(&user.password_hash)
            .map_err(|_| ApiError::InvalidCredentials)?;

        if !Argon2::default()
            .verify_password(request.password.as_bytes(), &parsed_hash)
            .is_ok()
        {
            return Err(ApiError::InvalidCredentials);
        }

        // Update last login timestamp
        let mut user_model: users::ActiveModel = user.clone().into();
        user_model.last_login_at = Set(Some(chrono::Utc::now()));
        user_model.updated_at = Set(chrono::Utc::now());

        users::Entity::update(user_model)
            .exec(&self.db)
            .await?;

        self.audit.log_event(AuditEvent {
            user_id: user.id,
            resource_type: "user".to_string(),
            resource_id: Some(user.id),
            action: "login".to_string(),
            details: json!({}),
            ip_address: None,
            user_agent: None,
        })
        .await?;

        Ok(user)
    }
} 