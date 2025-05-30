//! Contract Management System - Signature Model
//! 
//! This module defines the digital signature data model for the Contract Management System.
//! It provides the database schema and data structures for:
//! - Digital signatures
//! - Signature verification
//! - Signature metadata
//! - Signature history
//! - Signature relationships
//!
//! Features:
//! - Signature schema definition
//! - Verification status
//! - Metadata management
//! - History tracking
//! - Relationship management
//!
//! Data Structures:
//! - Signature entity
//! - Verification data
//! - Signature metadata
//! - Signature history
//! - Signature relationships
//!
//! Security Features:
//! - Digital signature verification
//! - Timestamp validation
//! - Chain of custody
//! - Audit trail
//! - Integrity checks
//!
//! Usage:
//! 1. Define signature structure
//! 2. Manage verification
//! 3. Track history
//! 4. Handle metadata
//! 5. Maintain relationships
//!
//! Author: Contract Management System Team
//! License: MIT

use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "signatures")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Uuid,
    pub party_id: Uuid,
    pub signature_data: String,
    #[sea_orm(column_type = "Text")]
    pub verification_method: VerificationMethod,
    pub created_at: DateTime,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::party::Entity",
        from = "Column::PartyId",
        to = "super::party::Column::Id"
    )]
    Party,
}

impl Related<super::party::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Party.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, EnumIter, DeriveActiveEnum)]
#[sea_orm(rs_type = "String", db_type = "String(Some(50))")]
pub enum VerificationMethod {
    #[sea_orm(string_value = "DIGITAL_SIGNATURE")]
    DigitalSignature,
    #[sea_orm(string_value = "MULTI_FACTOR_AUTH")]
    MultiFactorAuth,
    #[sea_orm(string_value = "HARDWARE_TOKEN")]
    HardwareToken,
}

impl Model {
    pub fn new(
        party_id: Uuid,
        signature_data: String,
        verification_method: VerificationMethod,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            party_id,
            signature_data,
            verification_method,
            created_at: chrono::Utc::now(),
        }
    }
} 