use sea_orm::entity::prelude::*;
use serde_json::Value;
use uuid::Uuid;
use crate::contracts::{ContractStatus, ContractTerms, ContractSignature};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "contracts")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: Uuid,
    pub title: String,
    pub description: String,
    pub provider_id: Uuid,
    pub consumer_id: Uuid,
    #[sea_orm(column_type = "String(Some(50))")]
    pub status: ContractStatus,
    pub terms: Value,
    pub signatures: Value,
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
    #[sea_orm(
        belongs_to = "super::identities::Entity",
        from = "Column::ConsumerId",
        to = "super::identities::Column::Id"
    )]
    Consumer,
}

impl Related<super::identities::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Provider.def()
    }
}

impl ActiveModelBehavior for ActiveModel {} 