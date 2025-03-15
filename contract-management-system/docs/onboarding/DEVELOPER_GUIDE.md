# Developer Onboarding Guide

## Table of Contents
1. [System Overview](#system-overview)
2. [Core Concepts](#core-concepts)
3. [Technology Stack](#technology-stack)
4. [Project Structure](#project-structure)
5. [Development Workflow](#development-workflow)
6. [Key Components](#key-components)
7. [Quick Start Tutorials](#quick-start-tutorials)

## System Overview

The Digital Contract Management System (DCMS) is a secure platform that handles:
- Contract lifecycle management
- Secure model training with privacy preservation
- Identity and access management
- Enclave-based secure computation

### High-Level Architecture
```
DCMS
├── Frontend (Web UI)
├── Backend Services
│   ├── Contract Management
│   ├── Model Training
│   ├── Identity Management
│   └── Enclave Management
└── Infrastructure
    ├── AWS ECS/Fargate
    ├── RDS PostgreSQL
    └── Nitro Enclaves
```

## Core Concepts

### 1. Contract Management
- **Digital Contracts**: Legally binding agreements between parties
- **Contract Lifecycle**: Draft → Review → Sign → Execute → Archive
- **Smart Validation**: Automated terms verification and compliance checks

### 2. Privacy-Preserving Training
- **Differential Privacy**: Mathematical privacy guarantees for training data
- **Secure Enclaves**: Isolated computation environments
- **Privacy Budget**: Controlled information disclosure tracking

### 3. Identity and Access Management
- **Identity Types**: Users, Machines, Enclaves
- **Credential Management**: Secure rotation and attestation
- **Access Control**: Role-based permissions and audit logging

## Technology Stack

### Backend
- **Language**: Rust
- **Framework**: Actix-web
- **ORM**: SeaORM
- **Database**: PostgreSQL
- **Security**: AWS Nitro Enclaves

### Infrastructure
- **Cloud**: AWS (ECS, RDS, ECR)
- **IaC**: Terraform
- **Containers**: Docker
- **CI/CD**: GitHub Actions

## Project Structure

```
contract-management-system/
├── src/
│   ├── contracts/       # Contract management
│   ├── training/        # Model training
│   ├── iam/            # Identity management
│   ├── enclave/        # Enclave integration
│   └── common/         # Shared utilities
├── migrations/         # Database migrations
├── tests/             # Integration tests
└── deployment/        # Infrastructure code
```

## Development Workflow

### 1. Local Development Setup
```bash
# Clone repository
git clone <repository-url>
cd contract-management-system

# Set up environment
cp .env.example .env
cargo build

# Run migrations
cargo run --bin migrations

# Start development server
cargo run --bin server
```

### 2. Making Changes
1. **Create Feature Branch**:
   ```bash
   git checkout -b feature/your-feature-name
   ```

2. **Run Tests**:
   ```bash
   cargo test
   cargo clippy
   ```

3. **Submit PR**:
   - Create detailed PR description
   - Link related issues
   - Add tests and documentation

## Key Components

### 1. Contract Management
```rust
// Example contract creation
pub async fn create_contract(
    title: String,
    terms: ContractTerms,
    parties: Vec<PartyInfo>,
) -> Result<Contract> {
    let contract = Contract::new(title, terms, parties);
    contract.validate()?;
    contract.save().await
}
```

### 2. Privacy-Preserving Training
```rust
// Example training configuration
pub struct TrainingConfig {
    pub epsilon: f64,          // Privacy budget
    pub noise_mechanism: Noise,
    pub batch_size: usize,
    pub learning_rate: f64,
}

// Initialize private training
let trainer = PrivateTrainer::new(config);
trainer.train_with_privacy(model, data).await?;
```

### 3. Identity Management
```rust
// Example identity verification
pub async fn verify_identity(
    id: Uuid,
    credentials: Credentials,
) -> Result<IdentityInfo> {
    let identity = Identity::find(id).await?;
    identity.verify_credentials(credentials).await?;
    identity.generate_session_token()
}
```

## Quick Start Tutorials

### 1. Creating a New Contract

```rust
// Example: Creating and validating a contract
async fn tutorial_create_contract() -> Result<()> {
    // Initialize contract
    let contract = Contract::builder()
        .title("Service Agreement")
        .add_party(provider)
        .add_party(consumer)
        .set_terms(terms)
        .build()?;

    // Validate and save
    contract.validate()?;
    contract.save().await?;
    
    Ok(())
}
```

### 2. Implementing Privacy-Preserving Training

```rust
// Example: Setting up private training
async fn tutorial_private_training() -> Result<()> {
    // Configure privacy settings
    let config = PrivacyConfig {
        epsilon: 1.0,
        delta: 1e-5,
        noise_mechanism: NoiseType::Gaussian,
    };

    // Initialize trainer
    let trainer = PrivateTrainer::new(config);
    
    // Train model
    trainer.train_with_privacy(model, data).await?;
    
    Ok(())
}
```

### 3. Managing Identities

```rust
// Example: Identity management
async fn tutorial_identity_management() -> Result<()> {
    // Create new identity
    let identity = Identity::new(
        "service-account",
        IdentityType::Machine,
        metadata,
    )?;

    // Generate credentials
    let credentials = identity.generate_credentials().await?;
    
    // Set up rotation policy
    identity.set_rotation_policy(RotationPolicy {
        max_age_days: 90,
        require_mfa: true,
    });

    Ok(())
}
```

## Best Practices

### 1. Code Style
- Follow Rust idioms and patterns
- Use meaningful variable names
- Add documentation for public APIs
- Implement proper error handling

### 2. Testing
- Write unit tests for core functionality
- Add integration tests for workflows
- Test edge cases and error conditions
- Use test fixtures and helpers

### 3. Security
- Never commit secrets
- Use secure credential storage
- Implement proper access controls
- Follow least privilege principle

## Common Tasks

### 1. Database Operations
```rust
// Example: Database query
async fn find_contracts_by_status(
    status: ContractStatus
) -> Result<Vec<Contract>> {
    Contract::find()
        .filter(contract::Column::Status.eq(status))
        .all()
        .await
}
```

### 2. Enclave Operations
```rust
// Example: Enclave attestation
async fn verify_enclave(
    attestation_doc: AttestationDoc
) -> Result<()> {
    let verifier = EnclaveVerifier::new();
    verifier.verify_attestation(attestation_doc).await?;
    Ok(())
}
```

### 3. Audit Logging
```rust
// Example: Audit logging
async fn log_contract_event(
    contract_id: Uuid,
    event_type: EventType,
    details: EventDetails,
) -> Result<()> {
    AuditLogger::new()
        .contract_id(contract_id)
        .event_type(event_type)
        .details(details)
        .log()
        .await
}
```

## Troubleshooting Guide

### Common Issues

1. **Database Connection**
```bash
# Check connection
psql $DATABASE_URL -c "\dt"

# Reset database
cargo run --bin migrations -- fresh
```

2. **Build Errors**
```bash
# Clean build
cargo clean
cargo build

# Check dependencies
cargo tree
```

3. **Runtime Errors**
- Check logs: `RUST_LOG=debug cargo run`
- Verify configurations
- Check service dependencies

## Additional Resources

1. **Documentation**
   - [API Reference](../api/README.md)
   - [Database Schema](../db/SCHEMA.md)
   - [Security Model](../security/README.md)

2. **External Links**
   - [Rust Book](https://doc.rust-lang.org/book/)
   - [SeaORM Docs](https://www.sea-ql.org/SeaORM/)
   - [AWS Nitro Enclaves](https://aws.amazon.com/ec2/nitro/nitro-enclaves/)

3. **Tools**
   - [Rust Analyzer](https://rust-analyzer.github.io/)
   - [cargo-watch](https://github.com/watchexec/cargo-watch)
   - [sea-orm-cli](https://www.sea-ql.org/SeaORM/docs/generate-entity/sea-orm-cli/) 