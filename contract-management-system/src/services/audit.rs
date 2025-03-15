use sea_orm::{DatabaseConnection, EntityTrait, Set};
use serde_json::Value;
use uuid::Uuid;
use crate::error::ApiError;
use crate::models::audit_logs;

#[derive(Debug, Clone)]
pub struct AuditEvent {
    pub user_id: Uuid,
    pub resource_type: String,
    pub resource_id: Option<Uuid>,
    pub action: String,
    pub details: Value,
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
}

pub struct AuditService {
    db: DatabaseConnection,
}

impl AuditService {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }

    pub async fn log_event(&self, event: AuditEvent) -> Result<(), ApiError> {
        let audit_log = audit_logs::ActiveModel {
            id: Set(Uuid::new_v4()),
            user_id: Set(event.user_id),
            resource_type: Set(event.resource_type),
            resource_id: Set(event.resource_id),
            action: Set(event.action),
            details: Set(event.details),
            ip_address: Set(event.ip_address),
            user_agent: Set(event.user_agent),
            created_at: Set(chrono::Utc::now()),
        };

        audit_logs::Entity::insert(audit_log)
            .exec(&self.db)
            .await?;

        Ok(())
    }

    pub async fn get_resource_history(
        &self,
        resource_type: &str,
        resource_id: Uuid,
    ) -> Result<Vec<audit_logs::Model>, ApiError> {
        let logs = audit_logs::Entity::find()
            .filter(audit_logs::Column::ResourceType.eq(resource_type))
            .filter(audit_logs::Column::ResourceId.eq(resource_id))
            .order_by_desc(audit_logs::Column::CreatedAt)
            .all(&self.db)
            .await?;

        Ok(logs)
    }

    pub async fn get_user_activity(
        &self,
        user_id: Uuid,
        limit: Option<u64>,
    ) -> Result<Vec<audit_logs::Model>, ApiError> {
        let mut query = audit_logs::Entity::find()
            .filter(audit_logs::Column::UserId.eq(user_id))
            .order_by_desc(audit_logs::Column::CreatedAt);

        if let Some(limit) = limit {
            query = query.limit(limit);
        }

        let logs = query.all(&self.db).await?;

        Ok(logs)
    }
} 