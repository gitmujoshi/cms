use sea_orm::entity::prelude::*;
use serde_json::Value;

// Identity Model
#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "identities")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Uuid,
    pub name: String,
    #[sea_orm(column_type = "String")]
    pub identity_type: String,
    #[sea_orm(column_type = "String")]
    pub status: String,
    pub created_at: DateTime,
    pub last_modified: DateTime,
    pub metadata: Value,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(has_many = "super::credentials::Entity")]
    Credentials,
    #[sea_orm(has_many = "super::role_assignment::Entity")]
    RoleAssignments,
}

impl Related<super::credentials::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Credentials.def()
    }
}

impl Related<super::role_assignment::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::RoleAssignments.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}

// Credentials Model
#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "credentials")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub identity_id: Uuid,
    #[sea_orm(column_type = "String")]
    pub credential_type: String,
    pub encrypted_data: Vec<u8>,
    pub expires_at: Option<DateTime>,
    pub last_rotated: DateTime,
    pub max_age_days: i32,
    pub require_rotation: bool,
    pub notify_before_days: i32,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::identity::Entity",
        from = "Column::IdentityId",
        to = "super::identity::Column::Id"
    )]
    Identity,
}

impl Related<super::identity::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Identity.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}

// Role Model
#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "roles")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub permissions: Value,
    pub created_at: DateTime,
    pub last_modified: DateTime,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(has_many = "super::role_assignment::Entity")]
    RoleAssignments,
}

impl Related<super::role_assignment::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::RoleAssignments.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}

// Role Assignment Model
#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "role_assignments")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub role_id: Uuid,
    #[sea_orm(primary_key)]
    pub identity_id: Uuid,
    pub assigned_at: DateTime,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::role::Entity",
        from = "Column::RoleId",
        to = "super::role::Column::Id"
    )]
    Role,
    #[sea_orm(
        belongs_to = "super::identity::Entity",
        from = "Column::IdentityId",
        to = "super::identity::Column::Id"
    )]
    Identity,
}

impl Related<super::role::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Role.def()
    }
}

impl Related<super::identity::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Identity.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}

// Policy Model
#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "policies")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    #[sea_orm(column_type = "String")]
    pub effect: String,
    pub resources: Value,
    pub actions: Value,
    pub conditions: Value,
    pub priority: i32,
    pub created_at: DateTime,
    pub last_modified: DateTime,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}

// Audit Model
#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "audit_events")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Uuid,
    #[sea_orm(column_type = "String")]
    pub event_type: String,
    pub identity_id: Uuid,
    pub resource_type: String,
    pub resource_id: Option<String>,
    pub action: String,
    #[sea_orm(column_type = "String")]
    pub status: String,
    pub error_message: Option<String>,
    pub request_metadata: Value,
    pub timestamp: DateTime,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::identity::Entity",
        from = "Column::IdentityId",
        to = "super::identity::Column::Id"
    )]
    Identity,
}

impl Related<super::identity::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Identity.def()
    }
}

impl ActiveModelBehavior for ActiveModel {} 