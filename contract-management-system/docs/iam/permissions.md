# IAM Permissions and Roles Reference

This document provides a comprehensive reference for the permissions and roles available in the IAM system.

## Available Permissions

### Contract Management
- `CreateContract`: Create new contracts
- `ReadContract`: View existing contracts
- `UpdateContract`: Modify contract details
- `DeleteContract`: Remove contracts
- `ListContracts`: List available contracts
- `SignContract`: Sign/approve contracts
- `ValidateContract`: Validate contract terms

### Identity Management
- `CreateIdentity`: Create new identities
- `ReadIdentity`: View identity details
- `UpdateIdentity`: Modify identity information
- `DeleteIdentity`: Remove identities
- `ListIdentities`: List available identities
- `ManageCredentials`: Manage identity credentials

### Role Management
- `CreateRole`: Create new roles
- `ReadRole`: View role details
- `UpdateRole`: Modify role permissions
- `DeleteRole`: Remove roles
- `ListRoles`: List available roles
- `AssignRole`: Assign roles to identities

### Model Training
- `CreateTrainingJob`: Create new training jobs
- `ReadTrainingJob`: View training job details
- `UpdateTrainingJob`: Modify training parameters
- `DeleteTrainingJob`: Cancel/remove training jobs
- `ListTrainingJobs`: List training jobs
- `MonitorTraining`: Monitor training progress

### Data Access
- `ReadTrainingData`: Access training datasets
- `WriteTrainingData`: Modify training data
- `DeleteTrainingData`: Remove training data
- `ShareTrainingData`: Share data with other parties

### Enclave Operations
- `CreateEnclave`: Create new enclaves
- `ReadEnclave`: View enclave details
- `UpdateEnclave`: Modify enclave configuration
- `DeleteEnclave`: Remove enclaves
- `ListEnclaves`: List available enclaves
- `AttestEnclave`: Perform enclave attestation

### Audit Operations
- `ReadAuditLogs`: View audit logs
- `ExportAuditLogs`: Export audit data
- `ConfigureAuditing`: Configure audit settings

### System Administration
- `ConfigureSystem`: Modify system settings
- `ManageEncryption`: Manage encryption keys
- `ViewMetrics`: Access system metrics
- `ManageBackups`: Manage system backups

## Predefined Permission Sets

### Administrator Permissions
```rust
pub fn get_admin_permissions() -> PermissionSet {
    // Full system access
    // Includes all available permissions
}
```

### Model Trainer Permissions
```rust
pub fn get_model_trainer_permissions() -> PermissionSet {
    // Permissions for model training operations
    // - Read contracts
    // - Manage training jobs
    // - Access training data
    // - Monitor metrics
}
```

### Data Provider Permissions
```rust
pub fn get_data_provider_permissions() -> PermissionSet {
    // Permissions for data management
    // - Read/sign contracts
    // - Monitor training
    // - Manage training data
    // - View metrics
}
```

### Auditor Permissions
```rust
pub fn get_auditor_permissions() -> PermissionSet {
    // Permissions for audit operations
    // - Read-only access to contracts
    // - View training jobs
    // - Access audit logs
    // - View metrics
}
```

## Permission Set Operations

### Creating Custom Permission Sets
```rust
let mut permissions = PermissionSet::new();
permissions.add(Permission::ReadContract);
permissions.add(Permission::ListContracts);
permissions.add(Permission::MonitorTraining);
```

### Combining Permission Sets
```rust
let mut combined = PermissionSet::new();
combined.merge(&set1);  // Add all permissions from set1
combined.merge(&set2);  // Add all permissions from set2
```

### Checking Permissions
```rust
// Check single permission
if permission_set.has(&Permission::ReadContract) {
    // Handle permission granted
}

// Check multiple permissions
let required = vec![
    Permission::ReadContract,
    Permission::SignContract,
];

if permission_set.has_all(&required) {
    // Handle all permissions granted
}

if permission_set.has_any(&required) {
    // Handle at least one permission granted
}
```

## Best Practices for Permission Management

1. **Principle of Least Privilege**
   - Grant minimum required permissions
   - Regularly review and revoke unnecessary permissions
   - Use time-bound permission grants where appropriate

2. **Role-Based Access Control**
   - Create roles based on job functions
   - Use predefined permission sets where possible
   - Document custom role requirements

3. **Permission Auditing**
   - Monitor permission usage patterns
   - Review high-privilege permission grants
   - Track permission changes in audit logs

4. **Permission Inheritance**
   - Use role hierarchy appropriately
   - Avoid duplicate permission assignments
   - Consider impact of inherited permissions

## Common Permission Patterns

### Read-Only Access
```rust
let mut readonly = PermissionSet::new();
readonly.add(Permission::ReadContract);
readonly.add(Permission::ListContracts);
readonly.add(Permission::ReadTrainingJob);
readonly.add(Permission::ListTrainingJobs);
readonly.add(Permission::ViewMetrics);
```

### Data Management
```rust
let mut data_manager = PermissionSet::new();
data_manager.add(Permission::ReadTrainingData);
data_manager.add(Permission::WriteTrainingData);
data_manager.add(Permission::ShareTrainingData);
data_manager.add(Permission::MonitorTraining);
```

### Security Administration
```rust
let mut security_admin = PermissionSet::new();
security_admin.add(Permission::ManageCredentials);
security_admin.add(Permission::ManageEncryption);
security_admin.add(Permission::ReadAuditLogs);
security_admin.add(Permission::ConfigureAuditing);
```

## Permission Evaluation Rules

1. **Explicit Deny**
   - Deny policies take precedence over allow policies
   - Used for explicit access restrictions

2. **Required Permissions**
   - Some operations may require multiple permissions
   - All required permissions must be granted

3. **Conditional Permissions**
   - Permissions may be subject to conditions
   - Conditions evaluated at runtime

4. **Permission Conflicts**
   - Higher priority policies take precedence
   - Conflicts resolved through policy evaluation order 