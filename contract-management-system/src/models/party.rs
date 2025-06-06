//! Contract Management System - Party Model
//! 
//! This module defines the contract party data model for the Contract Management System.
//! It provides the database schema and data structures for:
//! - Contract parties
//! - Party roles
//! - Party relationships
//! - Party metadata
//! - Party history
//!
//! Features:
//! - Party schema definition
//! - Role management
//! - Relationship tracking
//! - Metadata handling
//! - History management
//!
//! Data Structures:
//! - Party entity
//! - Role definitions
//! - Party metadata
//! - Party relationships
//! - Party history
//!
//! Relationships:
//! - Contracts
//! - Organizations
//! - Users
//! - Signatures
//! - Audit logs
//!
//! Usage:
//! 1. Define party structure
//! 2. Manage roles
//! 3. Track relationships
//! 4. Handle metadata
//! 5. Maintain history
//!
//! Author: Contract Management System Team
//! License: MIT

use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "parties")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Uuid,
    pub contract_id: Uuid,
    pub name: String,
    #[sea_orm(column_type = "Text")]
    pub role: PartyRole,
    pub email: String,
    pub phone: Option<String>,
    pub organization: String,
    pub created_at: DateTime,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::contract::Entity",
        from = "Column::ContractId",
        to = "super::contract::Column::Id"
    )]
    Contract,
    #[sea_orm(has_many = "super::signature::Entity")]
    Signatures,
}

impl Related<super::contract::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Contract.def()
    }
}

impl Related<super::signature::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Signatures.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, EnumIter, DeriveActiveEnum)]
#[sea_orm(rs_type = "String", db_type = "String(Some(50))")]
pub enum PartyRole {
    #[sea_orm(string_value = "DATA_PROVIDER")]
    DataProvider,
    #[sea_orm(string_value = "MODEL_TRAINER")]
    ModelTrainer,
    #[sea_orm(string_value = "AUDITOR")]
    Auditor,
    #[sea_orm(string_value = "ADMINISTRATOR")]
    Administrator,
}

impl Model {
    pub fn new(
        contract_id: Uuid,
        name: String,
        role: PartyRole,
        email: String,
        phone: Option<String>,
        organization: String,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            contract_id,
            name,
            role,
            email,
            phone,
            organization,
            created_at: chrono::Utc::now(),
        }
    }
} 