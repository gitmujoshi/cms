use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "contracts")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Uuid,
    pub title: String,
    pub description: String,
    #[sea_orm(column_type = "Text")]
    pub status: ContractStatus,
    #[sea_orm(column_type = "Text")]
    pub contract_type: ContractType,
    #[sea_orm(column_type = "JsonBinary")]
    pub terms: ContractTerms,
    pub created_at: DateTime,
    pub updated_at: DateTime,
    pub valid_from: DateTime,
    pub valid_until: Option<DateTime>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(has_many = "super::party::Entity")]
    Parties,
}

impl Related<super::party::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Parties.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, EnumIter, DeriveActiveEnum)]
#[sea_orm(rs_type = "String", db_type = "String(Some(50))")]
pub enum ContractStatus {
    #[sea_orm(string_value = "DRAFT")]
    Draft,
    #[sea_orm(string_value = "PENDING_APPROVAL")]
    PendingApproval,
    #[sea_orm(string_value = "ACTIVE")]
    Active,
    #[sea_orm(string_value = "EXPIRED")]
    Expired,
    #[sea_orm(string_value = "TERMINATED")]
    Terminated,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, EnumIter, DeriveActiveEnum)]
#[sea_orm(rs_type = "String", db_type = "String(Some(50))")]
pub enum ContractType {
    #[sea_orm(string_value = "DATA_SHARING")]
    DataSharing,
    #[sea_orm(string_value = "MODEL_TRAINING")]
    ModelTraining,
    #[sea_orm(string_value = "RESULT_SHARING")]
    ResultSharing,
    #[sea_orm(string_value = "HYBRID")]
    Hybrid,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ContractTerms {
    pub data_usage_terms: DataUsageTerms,
    pub security_requirements: SecurityRequirements,
    pub compliance_requirements: Vec<ComplianceRequirement>,
    pub model_training_terms: ModelTrainingTerms,
}

impl Default for ContractTerms {
    fn default() -> Self {
        Self {
            data_usage_terms: DataUsageTerms::default(),
            security_requirements: SecurityRequirements::default(),
            compliance_requirements: Vec::new(),
            model_training_terms: ModelTrainingTerms::default(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DataUsageTerms {
    pub allowed_purposes: Vec<String>,
    pub usage_restrictions: Vec<String>,
    pub retention_period: i32,
    pub data_handling_requirements: Vec<String>,
}

impl Default for DataUsageTerms {
    fn default() -> Self {
        Self {
            allowed_purposes: Vec::new(),
            usage_restrictions: Vec::new(),
            retention_period: 365,
            data_handling_requirements: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SecurityRequirements {
    pub encryption_level: EncryptionLevel,
    pub access_controls: Vec<String>,
    pub audit_requirements: Vec<String>,
}

impl Default for SecurityRequirements {
    fn default() -> Self {
        Self {
            encryption_level: EncryptionLevel::Standard,
            access_controls: Vec::new(),
            audit_requirements: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum EncryptionLevel {
    Standard,
    High,
    Military,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ComplianceRequirement {
    pub regulation: String,
    pub requirements: Vec<String>,
    pub verification_method: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ModelTrainingTerms {
    pub allowed_algorithms: Vec<String>,
    pub performance_requirements: Vec<String>,
    pub resource_limits: ResourceLimits,
    pub output_restrictions: Vec<String>,
}

impl Default for ModelTrainingTerms {
    fn default() -> Self {
        Self {
            allowed_algorithms: Vec::new(),
            performance_requirements: Vec::new(),
            resource_limits: ResourceLimits::default(),
            output_restrictions: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ResourceLimits {
    pub max_cpu_cores: i32,
    pub max_memory_gb: i32,
    pub max_training_time_hours: i32,
}

impl Default for ResourceLimits {
    fn default() -> Self {
        Self {
            max_cpu_cores: 4,
            max_memory_gb: 16,
            max_training_time_hours: 24,
        }
    }
}

impl Model {
    pub fn new(
        title: String,
        description: String,
        contract_type: ContractType,
        terms: ContractTerms,
        valid_from: DateTime,
        valid_until: Option<DateTime>,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            title,
            description,
            status: ContractStatus::Draft,
            contract_type,
            terms,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
            valid_from,
            valid_until,
        }
    }

    pub fn is_valid(&self) -> bool {
        let now = chrono::Utc::now();
        now >= self.valid_from
            && self.valid_until.map_or(true, |end| now <= end)
            && self.status == ContractStatus::Active
    }
} 