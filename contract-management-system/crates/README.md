# Contract Management System Crates

This directory contains the various crates that make up the Contract Management System. Each crate is designed to be modular and focused on specific functionality while maintaining clean integration points with the rest of the system.

## Crate Structure

### 1. `contract-templates`
Core contract template functionality.

```
contract-templates/
├── src/
│   ├── templates/       # Contract template definitions
│   │   ├── ai_training.rs
│   │   ├── data_sharing.rs
│   │   ├── model_ownership.rs
│   │   └── compliance.rs
│   ├── validation/      # Template validation logic
│   │   ├── schema.rs
│   │   ├── rules.rs
│   │   └── constraints.rs
│   ├── storage/         # Template storage backends
│   │   ├── s3.rs
│   │   └── versioning.rs
│   ├── rendering/       # Template rendering engines
│   │   ├── handlebars.rs
│   │   ├── tera.rs
│   │   └── variables.rs
│   ├── error.rs        # Error types
│   └── types.rs        # Common types
```

**Integration Points:**
- Used by the contract management API for template operations
- Interfaces with S3 for template storage
- Provides validation hooks for contract creation

### 2. `model-training`
Machine learning model training infrastructure.

```
model-training/
├── src/
│   ├── models/         # Model architectures
│   │   ├── mnist.rs
│   │   └── common.rs
│   ├── data/          # Data handling
│   │   ├── preprocessing.rs
│   │   ├── augmentation.rs
│   │   └── loader.rs
│   ├── training/      # Training logic
│   │   ├── trainer.rs
│   │   ├── metrics.rs
│   │   └── checkpoint.rs
│   ├── enclave/       # Enclave integration
│   │   ├── attestation.rs
│   │   └── secure_execution.rs
│   └── utils/         # Utility functions
       ├── secure_storage.rs
       └── metrics.rs
```

**Integration Points:**
- Interfaces with AWS Nitro Enclaves
- Uses IAM for access control
- Exports metrics to Prometheus
- Stores artifacts in S3

### 3. `contract-management`
Core contract management functionality.

```
contract-management/
├── src/
│   ├── contracts/     # Contract operations
│   │   ├── creation.rs
│   │   ├── validation.rs
│   │   └── lifecycle.rs
│   ├── workflow/      # Contract workflow
│   │   ├── states.rs
│   │   ├── transitions.rs
│   │   └── actions.rs
│   ├── storage/       # Contract storage
│   │   ├── database.rs
│   │   └── attachments.rs
│   └── api/          # API endpoints
       ├── handlers.rs
       └── routes.rs
```

**Integration Points:**
- Uses contract-templates for template management
- Interfaces with IAM for access control
- Stores data in PostgreSQL
- Exports metrics to Prometheus

### 4. `common`
Shared utilities and types.

```
common/
├── src/
│   ├── types/        # Common type definitions
│   ├── utils/        # Shared utilities
│   ├── errors/       # Error handling
│   └── metrics/      # Metrics collection
```

**Integration Points:**
- Used by all other crates
- Provides common functionality
- Standardizes error handling

## Integration Overview

### Authentication & Authorization
```rust
// Example of IAM integration in contract management
use iam::{Identity, Permission};

async fn create_contract(identity: Identity, contract: Contract) -> Result<()> {
    // Check permissions
    if !identity.has_permission(Permission::CreateContract) {
        return Err(Error::InsufficientPermissions);
    }
    
    // Create contract
    let contract_id = contracts::create(contract).await?;
    
    // Log audit event
    audit::log_contract_creation(identity.id, contract_id).await?;
    
    Ok(())
}
```

### Secure Storage
```rust
// Example of secure storage integration in model training
use model_training::utils::secure_storage::SecureStorage;

async fn store_training_data(data: &[u8]) -> Result<()> {
    let storage = SecureStorage::new(config)?;
    
    // Initialize secure storage
    storage.initialize().await?;
    
    // Store data
    storage.store("training_data", data).await?;
    
    Ok(())
}
```

### Metrics Collection
```rust
// Example of metrics integration
use common::metrics::{Counter, Histogram};

lazy_static! {
    static ref CONTRACT_OPERATIONS: Counter = Counter::new(
        "contract_operations_total",
        "Total number of contract operations"
    ).unwrap();
    
    static ref OPERATION_DURATION: Histogram = Histogram::new(
        "contract_operation_duration_seconds",
        "Duration of contract operations"
    ).unwrap();
}
```

## Development Guidelines

### 1. Crate Dependencies
- Keep dependencies between crates minimal and well-defined
- Use `common` crate for shared functionality
- Avoid circular dependencies

### 2. Error Handling
- Use the error types from `common::errors`
- Implement specific error types for each crate
- Provide context for errors

```rust
use common::errors::{Error, Result};

#[derive(Error, Debug)]
pub enum ContractError {
    #[error("Invalid template: {0}")]
    InvalidTemplate(String),
    #[error("Storage error: {0}")]
    StorageError(#[from] StorageError),
}
```

### 3. Testing
- Unit tests within each crate
- Integration tests between crates
- End-to-end tests for complete workflows

```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_contract_creation_workflow() {
        // Setup
        let identity = create_test_identity().await?;
        let template = load_test_template().await?;
        
        // Execute
        let contract = create_contract(identity, template).await?;
        
        // Verify
        assert_eq!(contract.status, ContractStatus::Draft);
        assert!(audit_log_exists(contract.id).await?);
    }
}
```

### 4. Documentation
- Document public APIs
- Include examples in documentation
- Keep README files updated

### 5. Metrics and Monitoring
- Define metrics in each crate
- Use common metric types
- Include dashboards and alerts

## Building and Testing

```bash
# Build all crates
cargo build --workspace

# Test specific crate
cargo test -p contract-templates

# Run all tests
cargo test --workspace

# Check formatting
cargo fmt --all -- --check

# Run clippy
cargo clippy --workspace
```

## Deployment

The crates are compiled into the main application binary and deployed as a single unit. However, they can also be used independently in other projects:

```toml
[dependencies]
contract-templates = { path = "../crates/contract-templates" }
model-training = { path = "../crates/model-training" }
contract-management = { path = "../crates/contract-management" }
common = { path = "../crates/common" }
```

## Future Considerations

1. **Modularity**
   - Consider splitting large crates
   - Extract reusable components
   - Maintain clean interfaces

2. **Performance**
   - Profile each crate independently
   - Optimize critical paths
   - Monitor resource usage

3. **Security**
   - Regular security audits
   - Dependency updates
   - Vulnerability scanning 