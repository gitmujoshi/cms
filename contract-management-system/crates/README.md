# Contract Management System Crates

This directory contains the various crates that make up the Contract Management System. Each crate is designed to be modular and focused on specific functionality while maintaining clean integration points with the rest of the system.

## Latest Updates (March 2024)

- Added support for AI-powered contract analysis in `model-training`
- Enhanced security with improved enclave attestation
- Implemented real-time contract monitoring
- Added support for multi-tenant deployments
- Improved performance with caching and optimization
- Enhanced error handling and logging
- Added comprehensive API documentation
- Implemented automated testing pipeline

## Crate Structure

### 1. `contract-templates`
Core contract template functionality with AI-powered analysis.

```
contract-templates/
├── src/
│   ├── templates/       # Contract template definitions
│   │   ├── ai_training.rs
│   │   ├── data_sharing.rs
│   │   ├── model_ownership.rs
│   │   ├── compliance.rs
│   │   └── ai_analysis.rs      # New: AI-powered template analysis
│   ├── validation/      # Template validation logic
│   │   ├── schema.rs
│   │   ├── rules.rs
│   │   ├── constraints.rs
│   │   └── ai_validator.rs     # New: AI-based validation
│   ├── storage/         # Template storage backends
│   │   ├── s3.rs
│   │   ├── versioning.rs
│   │   └── cache.rs           # New: Caching layer
│   ├── rendering/       # Template rendering engines
│   │   ├── handlebars.rs
│   │   ├── tera.rs
│   │   ├── variables.rs
│   │   └── ai_renderer.rs     # New: AI-enhanced rendering
│   ├── error.rs        # Error types
│   └── types.rs        # Common types
```

**Integration Points:**
- Used by the contract management API for template operations
- Interfaces with S3 for template storage
- Provides validation hooks for contract creation
- Integrates with AI services for enhanced analysis

### 2. `model-training`
Machine learning model training infrastructure with enhanced security.

```
model-training/
├── src/
│   ├── models/         # Model architectures
│   │   ├── mnist.rs
│   │   ├── common.rs
│   │   └── contract_analysis.rs  # New: Contract-specific models
│   ├── data/          # Data handling
│   │   ├── preprocessing.rs
│   │   ├── augmentation.rs
│   │   ├── loader.rs
│   │   └── secure_loader.rs     # New: Secure data loading
│   ├── training/      # Training logic
│   │   ├── trainer.rs
│   │   ├── metrics.rs
│   │   ├── checkpoint.rs
│   │   └── distributed.rs      # New: Distributed training
│   ├── enclave/       # Enclave integration
│   │   ├── attestation.rs
│   │   ├── secure_execution.rs
│   │   └── key_management.rs   # New: Enhanced key management
│   └── utils/         # Utility functions
       ├── secure_storage.rs
       ├── metrics.rs
       └── monitoring.rs        # New: Enhanced monitoring
```

**Integration Points:**
- Interfaces with AWS Nitro Enclaves
- Uses IAM for access control
- Exports metrics to Prometheus
- Stores artifacts in S3
- Integrates with monitoring systems

### 3. `audit-engine`
Enhanced audit and compliance tracking.

```
audit-engine/
├── src/
│   ├── logging/       # Audit logging
│   │   ├── events.rs
│   │   ├── storage.rs
│   │   └── realtime.rs        # New: Real-time monitoring
│   ├── compliance/    # Compliance tracking
│   │   ├── rules.rs
│   │   ├── checks.rs
│   │   └── reporting.rs       # New: Compliance reporting
│   ├── security/      # Security monitoring
│   │   ├── alerts.rs
│   │   ├── incidents.rs
│   │   └── response.rs        # New: Incident response
│   └── api/          # API endpoints
       ├── handlers.rs
       └── routes.rs
```

**Integration Points:**
- Real-time monitoring and alerts
- Compliance reporting
- Security incident tracking
- Integration with external audit systems

### 4. `enclave-runtime`
Secure execution environment with enhanced features.

```
enclave-runtime/
├── src/
│   ├── security/     # Security features
│   │   ├── attestation.rs
│   │   ├── encryption.rs
│   │   └── key_rotation.rs    # New: Key rotation
│   ├── execution/    # Execution environment
│   │   ├── runtime.rs
│   │   ├── isolation.rs
│   │   └── monitoring.rs      # New: Runtime monitoring
│   ├── storage/      # Secure storage
│   │   ├── secure_fs.rs
│   │   └── key_store.rs
│   └── api/         # API endpoints
       ├── handlers.rs
       └── routes.rs
```

**Integration Points:**
- Secure execution environment
- Key management and rotation
- Runtime monitoring
- Integration with cloud providers

### 5. `iam`
Enhanced identity and access management.

```
iam/
├── src/
│   ├── auth/        # Authentication
│   │   ├── providers.rs
│   │   ├── tokens.rs
│   │   └── mfa.rs           # New: Multi-factor auth
│   ├── access/      # Access control
│   │   ├── roles.rs
│   │   ├── permissions.rs
│   │   └── policies.rs
│   ├── users/       # User management
│   │   ├── management.rs
│   │   └── groups.rs
│   └── api/        # API endpoints
       ├── handlers.rs
       └── routes.rs
```

**Integration Points:**
- Multi-factor authentication
- Role-based access control
- User and group management
- Integration with external auth providers

### 6. `web-ui`
Modern web interface with enhanced features.

```
web-ui/
├── src/
│   ├── components/  # UI components
│   │   ├── contracts/
│   │   ├── templates/
│   │   └── admin/
│   ├── pages/      # Page components
│   │   ├── dashboard.rs
│   │   ├── contracts.rs
│   │   └── settings.rs
│   ├── state/      # State management
│   │   ├── store.rs
│   │   └── actions.rs
│   └── api/       # API integration
       ├── client.rs
       └── types.rs
```

**Integration Points:**
- Modern React-based UI
- Real-time updates
- Secure client-side operations
- API integration

## Development Guidelines

### 1. Crate Dependencies
- Keep dependencies between crates minimal and well-defined
- Use `common` crate for shared functionality
- Avoid circular dependencies
- Use semantic versioning for crate versions

### 2. Error Handling
- Use the error types from `common::errors`
- Implement specific error types for each crate
- Provide context for errors
- Include error codes for API responses

```rust
use common::errors::{Error, Result};

#[derive(Error, Debug)]
pub enum ContractError {
    #[error("Invalid template: {0}")]
    InvalidTemplate(String),
    #[error("Storage error: {0}")]
    StorageError(#[from] StorageError),
    #[error("AI analysis failed: {0}")]
    AIAnalysisError(String),  // New error type
}
```

### 3. Testing
- Unit tests within each crate
- Integration tests between crates
- End-to-end tests for complete workflows
- Performance tests for critical paths
- Security tests for sensitive operations

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
        
        // New: Verify AI analysis
        assert!(contract.ai_analysis.is_some());
    }
}
```

### 4. Documentation
- Document public APIs
- Include examples in documentation
- Keep README files updated
- Add architecture diagrams
- Document security considerations

### 5. Metrics and Monitoring
- Define metrics in each crate
- Use common metric types
- Include dashboards and alerts
- Monitor performance and security
- Track AI model performance

## Building and Testing

```bash
# Build all crates
cargo build --workspace

# Test specific crate
cargo test -p contract-templates

# Run all tests
cargo test --workspace

# Run performance tests
cargo bench --workspace

# Check formatting
cargo fmt --all -- --check

# Run clippy
cargo clippy --workspace

# Generate documentation
cargo doc --workspace --no-deps

# Run security audit
cargo audit
```

## Security Considerations

- All sensitive operations must use the enclave runtime
- Implement proper key rotation and management
- Use secure storage for sensitive data
- Follow least privilege principle
- Regular security audits and updates

## Performance Optimization

- Use caching where appropriate
- Implement connection pooling
- Optimize database queries
- Use async/await for I/O operations
- Monitor and optimize AI model performance

## Getting Started

1. Install Rust and required tools:
```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
rustup component add rustfmt clippy
```

2. Clone the repository:
```bash
git clone https://github.com/your-org/contract-management-system.git
cd contract-management-system
```

3. Build and test:
```bash
cargo build --workspace
cargo test --workspace
```

4. Start development:
```bash
cargo run -p web-ui  # Start the web UI
cargo run -p api     # Start the API server
```

## Contributing

1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Run tests and checks
5. Submit a pull request

## License

This project is licensed under the MIT License - see the LICENSE file for details. 