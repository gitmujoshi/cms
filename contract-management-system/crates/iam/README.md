# Identity and Access Management (IAM) Crate

The IAM crate provides comprehensive identity management, authentication, authorization, and audit capabilities for the Contract Management System.

## Structure

```
iam/
├── src/
│   ├── identity.rs      # Identity management
│   ├── roles.rs         # Role-based access control
│   ├── permissions.rs   # Permission definitions
│   ├── policies.rs      # Policy management
│   ├── audit.rs         # Audit logging
│   ├── models.rs        # Database models
│   └── mod.rs          # Module definitions
```

## Core Features

### 1. Identity Management
```rust
use iam::identity::{IdentityService, CreateIdentityRequest, IdentityType};

// Create identity service
let identity_service = IdentityService::new(db, encryption_key);

// Create new identity
let request = CreateIdentityRequest {
    name: "service-account-1".to_string(),
    identity_type: IdentityType::Machine,
    metadata: Some(json!({
        "purpose": "model-training",
        "environment": "production"
    })),
    initial_credentials: None,
};

let identity = identity_service.create_identity(request).await?;
```

### 2. Role Management
```rust
use iam::roles::{RoleService, CreateRoleRequest};
use iam::permissions::Permission;

// Create role service
let role_service = RoleService::new(db);

// Create new role
let request = CreateRoleRequest {
    name: "model-trainer".to_string(),
    description: Some("Role for ML model training".to_string()),
    permissions: vec![
        Permission::CreateTrainingJob,
        Permission::ReadTrainingData,
        Permission::MonitorTraining,
    ],
};

let role = role_service.create_role(request).await?;
```

### 3. Policy Management
```rust
use iam::policies::{PolicyService, CreatePolicyRequest, PolicyEffect};

// Create policy service
let policy_service = PolicyService::new(db);

// Create new policy
let request = CreatePolicyRequest {
    name: "production-access".to_string(),
    effect: PolicyEffect::Allow,
    resources: vec![resource_pattern],
    actions: vec![Permission::ReadTrainingData],
    conditions: vec![condition],
    priority: 100,
};

let policy = policy_service.create_policy(request).await?;
```

### 4. Audit Logging
```rust
use iam::audit::{AuditService, AuditEventType};

// Create audit service
let audit_service = AuditService::new(db);

// Log security event
audit_service
    .log_authentication_event(
        user_id,
        true,
        None,
        json!({
            "ip": client_ip,
            "method": "password"
        })
    )
    .await?;
```

## Integration with Other Crates

### 1. Contract Management Integration
```rust
use iam::permissions::Permission;
use contract_management::contracts::Contract;

impl Contract {
    pub async fn sign(&mut self, identity: &Identity) -> Result<()> {
        // Check signing permission
        if !identity.has_permission(Permission::SignContract) {
            return Err(Error::InsufficientPermissions);
        }

        // Perform signing
        self.status = ContractStatus::Signed;
        self.signed_by = Some(identity.id);
        self.signed_at = Some(chrono::Utc::now());

        // Log audit event
        audit_service
            .log_contract_event(
                identity.id,
                self.id,
                "sign",
                true,
                None,
                json!({ "contract_type": self.contract_type })
            )
            .await?;

        Ok(())
    }
}
```

### 2. Model Training Integration
```rust
use iam::permissions::Permission;
use model_training::training::TrainingJob;

impl TrainingJob {
    pub async fn create(config: TrainingConfig, identity: &Identity) -> Result<Self> {
        // Check training permissions
        if !identity.has_permission(Permission::CreateTrainingJob) {
            return Err(Error::InsufficientPermissions);
        }

        // Create job
        let job = Self::new(config);

        // Log audit event
        audit_service
            .log_training_event(
                identity.id,
                job.id,
                "create",
                true,
                None,
                json!({
                    "model_type": job.model_type,
                    "dataset_id": job.dataset_id
                })
            )
            .await?;

        Ok(job)
    }
}
```

### 3. API Integration
```rust
use actix_web::{web, HttpResponse};
use iam::identity::Identity;

async fn create_contract(
    identity: Identity,
    contract: web::Json<CreateContractRequest>,
) -> Result<HttpResponse> {
    // Identity is extracted from request by middleware
    if !identity.has_permission(Permission::CreateContract) {
        return Err(Error::InsufficientPermissions);
    }

    // Create contract
    let contract = Contract::create(contract.0, &identity).await?;

    Ok(HttpResponse::Created().json(contract))
}
```

## Database Integration

### 1. Schema Management
- Uses SeaORM for database operations
- Migrations handled by main application
- Indexes optimized for common queries

### 2. Model Definitions
```rust
#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "identities")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: Uuid,
    pub name: String,
    pub identity_type: String,
    pub status: String,
    pub created_at: DateTime,
    pub last_modified: DateTime,
    pub metadata: Value,
}
```

## Metrics and Monitoring

### 1. Prometheus Metrics
```rust
lazy_static! {
    static ref AUTH_ATTEMPTS: Counter = Counter::new(
        "iam_authentication_attempts_total",
        "Total number of authentication attempts"
    ).unwrap();

    static ref AUTH_FAILURES: Counter = Counter::new(
        "iam_authentication_failures_total",
        "Total number of failed authentication attempts"
    ).unwrap();
}
```

### 2. Grafana Dashboards
- Authentication metrics
- Authorization decisions
- Audit event volume
- Policy evaluation performance

## Testing

### 1. Unit Tests
```rust
#[tokio::test]
async fn test_permission_evaluation() {
    let identity = create_test_identity().await?;
    let permission = Permission::ReadContract;

    assert!(identity.has_permission(permission));
}
```

### 2. Integration Tests
```rust
#[tokio::test]
async fn test_contract_signing_workflow() {
    let identity = create_test_identity().await?;
    let contract = create_test_contract().await?;

    // Attempt to sign
    let result = contract.sign(&identity).await;

    // Verify audit log
    let events = audit_service
        .get_audit_events(AuditFilter {
            event_type: Some(AuditEventType::Authorization),
            identity_id: Some(identity.id),
            resource_id: Some(contract.id.to_string()),
            ..Default::default()
        })
        .await?;

    assert_eq!(events.len(), 1);
    assert_eq!(events[0].action, "sign");
}
```

## Security Considerations

1. **Credential Security**
   - Encrypted storage
   - Regular rotation
   - Secure key management

2. **Access Control**
   - Least privilege principle
   - Regular permission reviews
   - Policy validation

3. **Audit Trail**
   - Tamper-evident logging
   - Secure storage
   - Retention policies

## Performance Optimization

1. **Database Queries**
   - Optimized indexes
   - Query caching
   - Connection pooling

2. **Permission Checking**
   - Permission caching
   - Batch evaluations
   - Optimized policy evaluation

3. **Audit Logging**
   - Asynchronous logging
   - Log batching
   - Storage partitioning 