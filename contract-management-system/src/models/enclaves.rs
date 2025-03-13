use sea_orm::entity::prelude::*;
use serde_json::Value;
use uuid::Uuid;
use crate::enclave::{EnclaveStatus, EnclaveConfiguration, AttestationData, EnclaveMetrics};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "enclaves")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: Uuid,
    pub name: String,
    pub provider_id: Uuid,
    #[sea_orm(column_type = "String(Some(50))")]
    pub status: EnclaveStatus,
    pub attestation: Option<Value>,
    pub configuration: Value,
    pub metrics: Option<Value>,
    pub created_at: DateTimeWithTimeZone,
    pub updated_at: DateTimeWithTimeZone,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::identities::Entity",
        from = "Column::ProviderId",
        to = "super::identities::Column::Id"
    )]
    Provider,
}

impl Related<super::identities::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Provider.def()
    }
}

impl ActiveModelBehavior for ActiveModel {} 