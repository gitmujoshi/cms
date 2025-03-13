pub mod identity;
pub mod roles;
pub mod permissions;
pub mod policies;
pub mod audit;

use anyhow::Result;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Identity {
    pub id: Uuid,
    pub name: String,
    pub identity_type: IdentityType,
    pub status: IdentityStatus,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub last_modified: chrono::DateTime<chrono::Utc>,
    pub metadata: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum IdentityType {
    User,
    Machine,
    Group,
    Role,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum IdentityStatus {
    Active,
    Inactive,
    Suspended,
    Deleted,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IdentityCredentials {
    pub identity_id: Uuid,
    pub credential_type: CredentialType,
    pub credential_data: serde_json::Value,
    pub expires_at: Option<chrono::DateTime<chrono::Utc>>,
    pub last_rotated: chrono::DateTime<chrono::Utc>,
    pub rotation_policy: CredentialRotationPolicy,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CredentialType {
    Password,
    AccessKey,
    Certificate,
    OAuth2Token,
    EnclaveAttestation,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CredentialRotationPolicy {
    pub max_age_days: i32,
    pub require_rotation: bool,
    pub notify_before_days: i32,
}

pub trait IdentityManager {
    fn create_identity(&self, request: CreateIdentityRequest) -> Result<Identity>;
    fn get_identity(&self, id: Uuid) -> Result<Option<Identity>>;
    fn update_identity(&self, id: Uuid, request: UpdateIdentityRequest) -> Result<Identity>;
    fn delete_identity(&self, id: Uuid) -> Result<()>;
    fn list_identities(&self, filter: IdentityFilter) -> Result<Vec<Identity>>;
    fn assign_credentials(&self, id: Uuid, credentials: IdentityCredentials) -> Result<()>;
    fn rotate_credentials(&self, id: Uuid) -> Result<IdentityCredentials>;
    fn validate_credentials(&self, id: Uuid, credentials: &IdentityCredentials) -> Result<bool>;
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateIdentityRequest {
    pub name: String,
    pub identity_type: IdentityType,
    pub metadata: Option<serde_json::Value>,
    pub initial_credentials: Option<IdentityCredentials>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateIdentityRequest {
    pub name: Option<String>,
    pub status: Option<IdentityStatus>,
    pub metadata: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct IdentityFilter {
    pub identity_type: Option<IdentityType>,
    pub status: Option<IdentityStatus>,
    pub name_contains: Option<String>,
    pub created_after: Option<chrono::DateTime<chrono::Utc>>,
    pub created_before: Option<chrono::DateTime<chrono::Utc>>,
}

// Re-export commonly used types
pub use identity::IdentityService;
pub use permissions::{Permission, PermissionService};
pub use policies::{Policy, PolicyService};
pub use roles::{Role, RoleService}; 