use crate::iam::{Identity, Permission, Policy};
use anyhow::{Context, Result};
use sea_orm::{DatabaseConnection, EntityTrait, QueryFilter, Set};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tracing::{info, instrument};
use uuid::Uuid;
use crate::models::audit_events::{self, Entity as AuditEvent};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditEventData {
    pub id: Option<Uuid>,
    pub event_type: AuditEventType,
    pub identity_id: Option<Uuid>,
    pub resource_type: Option<String>,
    pub resource_id: Option<Uuid>,
    pub action: String,
    pub status: AuditEventStatus,
    pub error_message: Option<String>,
    pub request_metadata: serde_json::Value,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AuditEventType {
    Authentication,
    Authorization,
    IdentityManagement,
    RoleManagement,
    PolicyManagement,
    ResourceAccess,
    ContractCreation,
    ContractUpdate,
    ContractDeletion,
    ContractSigning,
    ContractActivation,
    CredentialRotation,
    EnclaveAttestation,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AuditEventStatus {
    Success,
    Failure,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AuditFilter {
    pub event_type: Option<AuditEventType>,
    pub identity_id: Option<Uuid>,
    pub resource_type: Option<String>,
    pub resource_id: Option<String>,
    pub action: Option<String>,
    pub status: Option<AuditEventStatus>,
    pub start_time: Option<chrono::DateTime<chrono::Utc>>,
    pub end_time: Option<chrono::DateTime<chrono::Utc>>,
}

pub struct AuditService {
    db: DatabaseConnection,
}

impl AuditService {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }

    pub async fn log_event(&self, event: AuditEventData) -> Result<()> {
        let audit_event = audit_events::ActiveModel {
            id: Set(event.id.unwrap_or_else(Uuid::new_v4)),
            event_type: Set(event.event_type),
            identity_id: Set(event.identity_id),
            resource_type: Set(event.resource_type),
            resource_id: Set(event.resource_id),
            action: Set(event.action),
            status: Set(event.status),
            error_message: Set(event.error_message),
            request_metadata: Set(event.request_metadata),
            timestamp: Set(event.timestamp),
        };

        AuditEvent::insert(audit_event)
            .exec(&self.db)
            .await
            .context("Failed to log audit event")?;

        Ok(())
    }

    pub async fn get_events(&self, filters: AuditEventFilters) -> Result<Vec<AuditEventData>> {
        let mut query = AuditEvent::find();

        if let Some(event_type) = filters.event_type {
            query = query.filter(audit_events::Column::EventType.eq(event_type));
        }

        if let Some(identity_id) = filters.identity_id {
            query = query.filter(audit_events::Column::IdentityId.eq(identity_id));
        }

        if let Some(resource_id) = filters.resource_id {
            query = query.filter(audit_events::Column::ResourceId.eq(resource_id));
        }

        if let Some(status) = filters.status {
            query = query.filter(audit_events::Column::Status.eq(status));
        }

        let events = query
            .all(&self.db)
            .await
            .context("Failed to fetch audit events")?;

        Ok(events
            .into_iter()
            .map(|e| AuditEventData {
                id: Some(e.id),
                event_type: e.event_type,
                identity_id: e.identity_id,
                resource_type: e.resource_type,
                resource_id: e.resource_id,
                action: e.action,
                status: e.status,
                error_message: e.error_message,
                request_metadata: e.request_metadata,
                timestamp: e.timestamp,
            })
            .collect())
    }

    pub async fn export_events(&self, format: ExportFormat) -> Result<Vec<u8>> {
        let events = self.get_events(AuditEventFilters::default()).await?;

        match format {
            ExportFormat::Json => serde_json::to_vec(&events)
                .context("Failed to serialize events to JSON"),
            ExportFormat::Csv => {
                let mut wtr = csv::Writer::from_writer(vec![]);
                for event in events {
                    wtr.serialize(event)
                        .context("Failed to serialize event to CSV")?;
                }
                wtr.into_inner().context("Failed to generate CSV")
            }
        }
    }
}

#[derive(Debug, Default)]
pub struct AuditEventFilters {
    pub event_type: Option<AuditEventType>,
    pub identity_id: Option<Uuid>,
    pub resource_id: Option<Uuid>,
    pub status: Option<AuditEventStatus>,
}

pub enum ExportFormat {
    Json,
    Csv,
}

#[cfg(test)]
mod tests {
    use super::*;
    use sea_orm::{DatabaseBackend, MockDatabase, MockExecResult};
    use std::sync::Once;

    static INIT: Once = Once::new();

    async fn setup_test_db() -> DatabaseConnection {
        INIT.call_once(|| {
            // Initialize logging for tests
            let _ = env_logger::builder().is_test(true).try_init();
        });

        // Create mock database with test data
        let db = MockDatabase::new(DatabaseBackend::Postgres)
            .append_query_results(vec![
                vec![audit_events::Model {
                    id: Uuid::new_v4(),
                    event_type: AuditEventType::Authentication,
                    identity_id: Some(Uuid::new_v4()),
                    resource_type: Some("user".to_string()),
                    resource_id: Some(Uuid::new_v4()),
                    action: "login".to_string(),
                    status: AuditEventStatus::Success,
                    error_message: None,
                    request_metadata: serde_json::json!({"ip": "127.0.0.1"}),
                    timestamp: chrono::Utc::now(),
                }],
            ])
            .append_exec_results(vec![MockExecResult {
                last_insert_id: 1,
                rows_affected: 1,
            }])
            .into_connection();

        db
    }

    #[tokio::test]
    async fn test_log_event() {
        let db = setup_test_db().await;
        let service = AuditService::new(db);

        let event = AuditEventData {
            id: None,
            event_type: AuditEventType::Authentication,
            identity_id: Some(Uuid::new_v4()),
            resource_type: Some("user".to_string()),
            resource_id: Some(Uuid::new_v4()),
            action: "login".to_string(),
            status: AuditEventStatus::Success,
            error_message: None,
            request_metadata: serde_json::json!({"ip": "127.0.0.1"}),
            timestamp: chrono::Utc::now(),
        };

        let result = service.log_event(event).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_get_events() {
        let db = setup_test_db().await;
        let service = AuditService::new(db);

        let filters = AuditEventFilters {
            event_type: Some(AuditEventType::Authentication),
            ..Default::default()
        };

        let result = service.get_events(filters).await;
        assert!(result.is_ok());
        assert!(!result.unwrap().is_empty());
    }

    #[tokio::test]
    async fn test_export_events() {
        let db = setup_test_db().await;
        let service = AuditService::new(db);

        let json_result = service.export_events(ExportFormat::Json).await;
        assert!(json_result.is_ok());

        let csv_result = service.export_events(ExportFormat::Csv).await;
        assert!(csv_result.is_ok());
    }
} 