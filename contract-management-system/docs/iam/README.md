# Identity and Access Management (IAM)

The Contract Management System includes a comprehensive Identity and Access Management (IAM) system that provides secure authentication, authorization, and audit capabilities. This document outlines the key components and usage patterns.

## Core Components

### 1. Identity Management

The system supports multiple identity types:
- User identities (human users)
- Machine identities (service accounts)
- Groups (collections of identities)
- Roles (reusable permission sets)

```rust
// Create a new user identity
let request = CreateIdentityRequest {
    name: "john.doe@example.com".to_string(),
    identity_type: IdentityType::User,
    metadata: Some(json!({
        "department": "Engineering",
        "location": "HQ"
    })),
    initial_credentials: Some(credentials),
};

let identity = identity_service.create_identity(request).await?;
```

### 2. Credential Management

Supports multiple credential types with secure storage and rotation:
- Passwords (with bcrypt hashing)
- Access Keys
- Certificates
- OAuth2 Tokens
- Enclave Attestations

```rust
// Assign new credentials
let credentials = IdentityCredentials {
    identity_id: user_id,
    credential_type: CredentialType::Password,
    credential_data: json!({"hash": "$2b$12$..."}),
    expires_at: Some(expiry_date),
    last_rotated: chrono::Utc::now(),
    rotation_policy: CredentialRotationPolicy {
        max_age_days: 90,
        require_rotation: true,
        notify_before_days: 14,
    },
};

identity_service.assign_credentials(user_id, credentials).await?;
```

### 3. Role-Based Access Control (RBAC)

Hierarchical role system with inheritance:
- Predefined roles (Admin, ModelTrainer, DataProvider, Auditor)
- Custom role creation
- Role assignment to identities

```rust
// Create a custom role
let request = CreateRoleRequest {
    name: "DataScientist".to_string(),
    description: Some("Data scientist role for model training".to_string()),
    permissions: vec![
        Permission::ReadTrainingData,
        Permission::CreateTrainingJob,
        Permission::MonitorTraining,
    ],
};

let role = role_service.create_role(request).await?;

// Assign role to identity
role_service.assign_role_to_identity(role.id, user_id).await?;
```

### 4. Policy Management

Fine-grained access control through policies:
- Resource-based policies
- Action-based restrictions
- Conditional access rules
- Priority-based evaluation

```rust
// Create an access policy
let request = CreatePolicyRequest {
    name: "TrainingDataAccess".to_string(),
    effect: PolicyEffect::Allow,
    resources: vec![ResourcePattern {
        resource_type: "training_data".to_string(),
        pattern: "datasets/*".to_string(),
    }],
    actions: vec![Permission::ReadTrainingData],
    conditions: vec![PolicyCondition {
        condition_type: ConditionType::StringEquals,
        key: "env".to_string(),
        values: vec!["production".to_string()],
    }],
    priority: 100,
};

let policy = policy_service.create_policy(request).await?;
```

### 5. Audit Logging

Comprehensive audit trail for all IAM operations:
- Authentication events
- Authorization decisions
- Identity management
- Role/Policy changes
- Resource access

```rust
// Query audit logs
let filter = AuditFilter {
    event_type: Some(AuditEventType::Authorization),
    identity_id: Some(user_id),
    start_time: Some(start_date),
    end_time: Some(end_date),
    ..Default::default()
};

let events = audit_service.get_audit_events(filter).await?;

// Export audit logs
let export = audit_service
    .export_audit_events(filter, AuditExportFormat::Json)
    .await?;
```

## Security Features

1. **Credential Security**:
   - Encrypted storage of sensitive credentials
   - Automatic credential rotation
   - Configurable expiration policies
   - Support for hardware security modules

2. **Access Control**:
   - Least privilege principle enforcement
   - Resource-level permissions
   - Conditional access policies
   - Role-based access control

3. **Audit Trail**:
   - Comprehensive event logging
   - Tamper-evident audit records
   - Exportable audit logs
   - Real-time monitoring capabilities

## Best Practices

1. **Identity Management**:
   - Use machine identities for automated processes
   - Implement regular credential rotation
   - Apply the principle of least privilege
   - Regularly review access patterns

2. **Role Management**:
   - Create roles based on job functions
   - Avoid direct permission assignments
   - Review role assignments periodically
   - Document role purposes and scope

3. **Policy Management**:
   - Use specific resource patterns
   - Set appropriate policy priorities
   - Implement conditional access where needed
   - Regular policy review and cleanup

4. **Audit and Compliance**:
   - Monitor authentication failures
   - Review high-privilege operations
   - Export audit logs regularly
   - Maintain compliance evidence

## Database Schema

The IAM system uses the following database tables:

1. `identities`: Stores identity information
2. `credentials`: Manages identity credentials
3. `roles`: Defines available roles
4. `role_assignments`: Maps roles to identities
5. `policies`: Stores access policies
6. `audit_events`: Records audit trail

Each table includes appropriate indexes for performance optimization.

## Integration Points

1. **Authentication Flow**:
   ```rust
   // Validate credentials
   let valid = identity_service
       .validate_credentials(user_id, &provided_credentials)
       .await?;
   ```

2. **Authorization Check**:
   ```rust
   // Check permission
   let allowed = role_service
       .check_identity_has_permission(user_id, Permission::CreateContract)
       .await?;
   ```

3. **Audit Integration**:
   ```rust
   // Log security event
   audit_service
       .log_authentication_event(
           user_id,
           success,
           error_msg,
           metadata,
       )
       .await?;
   ```

## Error Handling

The IAM system provides detailed error types for common scenarios:
- Invalid credentials
- Expired credentials
- Insufficient permissions
- Invalid role assignments
- Policy conflicts

Handle errors appropriately in your application code:
```rust
match result {
    Ok(_) => // Success case,
    Err(e) => match e.downcast_ref::<IamError>() {
        Some(IamError::InvalidCredentials) => // Handle auth failure,
        Some(IamError::InsufficientPermissions) => // Handle authorization failure,
        _ => // Handle other errors,
    }
}
```

## Monitoring and Metrics

The IAM system exports metrics for monitoring:
- Authentication success/failure rates
- Authorization decision counts
- Credential rotation status
- Policy evaluation performance
- Audit log volume

Monitor these metrics using the integrated Prometheus/Grafana stack. 