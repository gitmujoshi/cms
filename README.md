# Contract Management System with AWS Nitro Enclaves

A secure and robust contract management system built with Rust, featuring user management, organization management, and comprehensive contract lifecycle management. The system includes AWS Nitro Enclaves integration for enhanced security during model training and sensitive operations.

## Key Features

### Core Features
- User and Organization Management
- Contract Lifecycle Management
- Role-based Access Control
- Digital Signature Support
- Comprehensive Audit Logging

### Security Features
- AWS Nitro Enclaves Integration
- Secure Model Training
- Encrypted Data Processing
- Hardware-backed Security

## Project Structure

```
.
├── contract-management-system/    # Main application code
├── deployment/                    # Deployment configurations
├── docker/                       # Docker-related files
├── docs/                         # Documentation
├── modules/                      # Reusable modules
├── monitoring/                   # Monitoring setup
└── iam-system/                  # IAM configurations
```

## Getting Started

### Prerequisites

- Rust (latest stable version)
- PostgreSQL
- AWS CLI with Nitro Enclaves support
- Docker
- Node.js (for web UI development)

### Setup

1. Clone the repository:
   ```bash
   git clone https://github.com/yourusername/contract-management-system.git
   cd contract-management-system
   ```

2. Install AWS Nitro Enclaves CLI:
   ```bash
   # For Amazon Linux 2
   sudo yum install -y aws-nitro-enclaves-cli-dev
   
   # For Ubuntu
   sudo apt-get install -y aws-nitro-enclaves-cli-dev
   ```

3. Set up the development environment:
   ```bash
   # Create and activate virtual environment
   python -m venv venv
   source venv/bin/activate
   
   # Install dependencies
   pip install -r requirements.txt
   ```

4. Configure AWS Nitro Enclaves:
   ```bash
   # Enable Nitro Enclaves
   sudo systemctl enable nitro-enclaves-allocator.service
   sudo systemctl start nitro-enclaves-allocator.service
   ```

5. Run the application:
   ```bash
   # Start the main application
   cd contract-management-system
   cargo run
   ```

## AWS Nitro Enclaves Integration

The system uses AWS Nitro Enclaves for secure model training and sensitive operations. This provides hardware-backed security and isolation for critical processes.

### Key Security Features

- **Isolated Execution**: Secure processing in isolated memory regions
- **Encrypted Communication**: Secure channel between parent instance and enclave
- **Attestation**: Hardware-backed attestation of enclave integrity
- **Secure Storage**: Encrypted storage for sensitive data

### Usage

1. Build the enclave image:
   ```bash
   cd deployment
   ./build_enclave.sh
   ```

2. Deploy the enclave:
   ```bash
   ./deploy_enclave.sh
   ```

3. Monitor enclave status:
   ```bash
   ./monitor_enclave.sh
   ```

## Documentation

- [Main Application Documentation](contract-management-system/README.md)
- [API Documentation](docs/api/README.md)
- [Database Documentation](docs/db/README.md)
- [Deployment Guide](deployment/README.md)
- [Docker Setup](docker/README.md)

## Contributing

Please read [CONTRIBUTING.md](contract-management-system/CONTRIBUTING.md) for details on our code of conduct and the process for submitting pull requests.

## License

This project is licensed under the MIT License - see the [LICENSE](contract-management-system/LICENSE) file for details.

## Security

For security concerns, please email security@yourdomain.com or create a security advisory in the GitHub repository.

## Support

For support, please:
1. Check the [documentation](docs/)
2. Open an issue in the GitHub repository
3. Contact support@yourdomain.com 