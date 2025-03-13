# Digital Contract Management System

A secure and scalable system for managing digital contracts and secure computation environments for machine learning model training.

## Features

### Contract Management
- Create and manage digital contracts between data providers and consumers
- Complex contract terms and conditions support
- Digital signature workflow
- Contract lifecycle management
- Audit logging and compliance tracking

### Secure Computation
- AWS Nitro Enclaves integration
- Remote attestation verification
- Secure model training environment
- Resource isolation and monitoring
- Network security policies

### Identity and Access Management
- Role-based access control
- Identity verification
- Credential management
- Audit logging
- Policy enforcement

## Architecture

The system consists of several key components:

1. [Contract Management](docs/architecture/contracts_and_enclaves.md#contract-management-system)
   - Contract creation and management
   - Digital signatures
   - Terms enforcement
   - Audit logging

2. [Enclave Management](docs/architecture/contracts_and_enclaves.md#enclave-management-system)
   - AWS Nitro Enclaves integration
   - Attestation verification
   - Resource management
   - Security monitoring

3. [Identity Management](docs/architecture/identity.md)
   - User and machine identities
   - Role management
   - Access control
   - Credential rotation

## Getting Started

### Prerequisites
- Rust 1.75 or later
- PostgreSQL 13 or later
- AWS Account with Nitro Enclaves support
- Docker

### Installation
1. Clone the repository
```bash
git clone https://github.com/yourusername/contract-management-system.git
cd contract-management-system
```

2. Set up the database
```bash
cargo install sea-orm-cli
sea-orm-cli migrate up
```

3. Configure AWS credentials
```bash
aws configure
```

4. Build and run
```bash
cargo build --release
cargo run --release
```

## Documentation
- [Architecture Overview](docs/architecture/README.md)
- [Contract and Enclave Management](docs/architecture/contracts_and_enclaves.md)
- [Identity Management](docs/architecture/identity.md)
- [API Documentation](docs/api/README.md)
- [Deployment Guide](docs/deployment/README.md)

## Development

### Running Tests
```bash
cargo test
```

### Database Migrations
```bash
sea-orm-cli migrate up    # Apply migrations
sea-orm-cli migrate down  # Revert migrations
```

### Code Style
```bash
cargo fmt
cargo clippy
```

## License
This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.