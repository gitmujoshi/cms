# IAM Audit Logging

The IAM system includes comprehensive audit logging capabilities to track all security-relevant events and maintain compliance with security requirements.

## Audit Event Types

### 1. Authentication Events
- Login attempts (successful/failed)
- Password changes
- Multi-factor authentication events
- Session management

### 2. Authorization Events
- Permission checks
- Access denials
- Policy evaluations
- Role assignments

### 3. Identity Management Events
- Identity creation/modification/deletion
- Credential management
- Status changes
- Profile updates

### 4. Role Management Events
- Role creation/modification/deletion
- Permission changes
- Role assignments/removals
- Role hierarchy changes

### 5. Policy Management Events
- Policy creation/modification/deletion
- Policy priority changes
- Condition updates
- Resource pattern changes

### 6. Resource Access Events
- Contract operations
- Training data access
- Model training operations
- System configuration changes

## Audit Event Structure

Each audit event contains:
```rust
pub struct AuditEvent {
    pub id: Uuid,                    // Unique event identifier
    pub event_type: AuditEventType,  // Type of event
    pub identity_id: Uuid,           // Identity performing the action
    pub resource_type: String,       // Type of resource affected
    pub resource_id: Option<String>, // Specific resource identifier
    pub action: String,              // Action performed
    pub status: AuditEventStatus,    // Outcome (Success/Failure)
    pub error_message: Option<String>, // Error details if failed
    pub request_metadata: Value,     // Additional context
    pub timestamp: DateTime<Utc>,    // When the event occurred
}
```

## Querying Audit Logs

### Basic Query
```rust
let filter = AuditFilter {
    event_type: Some(AuditEventType::Authentication),
    identity_id: Some(user_id),
    start_time: Some(one_hour_ago),
    end_time: Some(now),
    ..Default::default()
};

let events = audit_service.get_audit_events(filter).await?;
```

### Advanced Filtering
```rust
let filter = AuditFilter {
    event_type: Some(AuditEventType::Authorization),
    resource_type: Some("contract".to_string()),
    status: Some(AuditEventStatus::Failure),
    start_time: Some(start_date),
    end_time: Some(end_date),
};

let events = audit_service.get_audit_events(filter).await?;
```

## Exporting Audit Logs

### JSON Export
```rust
let export = audit_service
    .export_audit_events(filter, AuditExportFormat::Json)
    .await?;
```

### CSV Export
```rust
let export = audit_service
    .export_audit_events(filter, AuditExportFormat::Csv)
    .await?;
```

## Audit Event Examples

### Authentication Event
```rust
audit_service
    .log_authentication_event(
        user_id,
        success: true,
        None,
        json!({
            "ip_address": "192.168.1.1",
            "user_agent": "Mozilla/5.0...",
            "mfa_used": true
        })
    )
    .await?;
```

### Authorization Event
```rust
audit_service
    .log_authorization_event(
        user_id,
        "contract".to_string(),
        Some(contract_id.to_string()),
        Permission::SignContract,
        false,
        Some("Insufficient permissions".to_string()),
        json!({
            "contract_type": "training",
            "requested_action": "sign"
        })
    )
    .await?;
```

### Identity Management Event
```rust
audit_service
    .log_identity_management_event(
        admin_id,
        target_user_id,
        "update_status".to_string(),
        true,
        None,
        json!({
            "old_status": "active",
            "new_status": "suspended",
            "reason": "security policy violation"
        })
    )
    .await?;
```

## Best Practices

### 1. Event Logging
- Log all security-relevant events
- Include sufficient context in metadata
- Use consistent event types and actions
- Maintain accurate timestamps

### 2. Data Protection
- Protect audit logs from tampering
- Implement appropriate retention policies
- Encrypt sensitive information
- Regular backups of audit data

### 3. Monitoring
- Set up alerts for security events
- Monitor authentication failures
- Track unusual access patterns
- Review high-privilege operations

### 4. Compliance
- Maintain required audit trails
- Export logs for compliance reviews
- Document retention policies
- Regular audit log reviews

## Performance Considerations

### 1. Database Optimization
- Indexed fields for efficient queries
- Partitioned tables by date
- Regular maintenance of indexes
- Archival strategy for old events

### 2. Query Performance
- Use appropriate filters
- Limit result sets
- Consider date range restrictions
- Use efficient export formats

### 3. Storage Management
- Implement log rotation
- Archive old audit data
- Compress archived logs
- Monitor storage usage

## Integration with Monitoring

### 1. Metrics Collection
```rust
// Example metrics
- audit_events_total{type="authentication"} 
- audit_events_failed{type="authorization"}
- audit_log_size_bytes
- audit_query_duration_seconds
```

### 2. Alerting Rules
```yaml
# Example alert rules
- alert: HighAuthenticationFailures
  expr: rate(audit_events_failed{type="authentication"}[5m]) > 10
  for: 5m
  labels:
    severity: warning
  annotations:
    description: "High rate of authentication failures detected"

- alert: AuditLogStorageNearFull
  expr: audit_log_size_bytes / audit_log_max_bytes > 0.9
  for: 1h
  labels:
    severity: warning
  annotations:
    description: "Audit log storage is nearly full"
```

## Error Handling

### Common Error Scenarios
1. Database connectivity issues
2. Storage capacity limits
3. Invalid filter parameters
4. Export format errors

### Error Recovery
```rust
match audit_service.log_event(event).await {
    Ok(_) => // Success case,
    Err(e) => match e.downcast_ref::<AuditError>() {
        Some(AuditError::DatabaseError(_)) => {
            // Implement retry logic
            // Log error to separate facility
            // Alert operations team
        },
        Some(AuditError::StorageError(_)) => {
            // Handle storage issues
            // Implement fallback storage
        },
        _ => // Handle other errors,
    }
}
``` 