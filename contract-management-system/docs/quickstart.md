# Quick Start Guide

## Getting Started in 5 Minutes

### Prerequisites
- Docker and Docker Compose
- Rust 1.75+
- Node.js 20+ (for frontend)

### 1. Clone and Setup
```bash
# Clone repository
git clone https://github.com/your-org/contract-management-system
cd contract-management-system

# Copy environment file
cp .env.example .env

# Install dependencies
cargo build
```

### 2. Start Development Environment
```bash
# Start required services
docker-compose up -d

# Run database migrations
diesel setup
diesel migration run

# Start the application
cargo run
```

### 3. Create Your First Contract

```bash
# Register a DID
curl -X POST http://localhost:8080/api/v1/auth/register \
  -H "Content-Type: application/json" \
  -d '{
    "did": "did:example:123",
    "public_key": "your-public-key"
  }'

# Get authentication challenge
curl -X POST http://localhost:8080/api/v1/auth/challenge \
  -H "Content-Type: application/json" \
  -d '{
    "did": "did:example:123"
  }'

# Submit signed challenge
curl -X POST http://localhost:8080/api/v1/auth/verify \
  -H "Content-Type: application/json" \
  -d '{
    "did": "did:example:123",
    "challenge": "received-challenge",
    "signature": "your-signature"
  }'

# Create a contract
curl -X POST http://localhost:8080/api/v1/contracts \
  -H "Authorization: Bearer your-jwt-token" \
  -H "Content-Type: application/json" \
  -d '{
    "title": "Test Contract",
    "description": "A test contract",
    "consumer_did": "did:example:456",
    "terms": "Contract terms...",
    "valid_from": "2024-03-21T00:00:00Z",
    "valid_until": "2025-03-21T00:00:00Z"
  }'
```

### 4. View Contract Events

```bash
# Get contract events
curl -X GET http://localhost:8080/api/v1/contracts/{contract-id}/events \
  -H "Authorization: Bearer your-jwt-token"
```

### 5. Monitor System

```bash
# View metrics
curl http://localhost:8080/metrics

# Check health
curl http://localhost:8080/health
```

## Next Steps

1. **Explore Documentation**
   - Read the full [Tutorial](tutorial.md)
   - Check [API Documentation](api/openapi.yaml)
   - Review [Architecture Guide](architecture.md)

2. **Development**
   - Write your first integration test
   - Create a custom contract template
   - Implement a client application

3. **Deployment**
   - Set up CI/CD pipeline
   - Configure monitoring
   - Plan production deployment

## Common Commands

### Development
```bash
# Run tests
cargo test

# Check code style
cargo fmt

# Run linter
cargo clippy

# Build release
cargo build --release
```

### Docker
```bash
# Build image
docker build -t cms .

# Run container
docker run -p 8080:8080 cms

# View logs
docker logs -f cms
```

### Database
```bash
# Create migration
diesel migration generate add_contracts

# Run pending migrations
diesel migration run

# Revert last migration
diesel migration revert
```

## Troubleshooting

### Common Issues

1. **Database Connection Failed**
```bash
# Check PostgreSQL is running
docker ps | grep postgres

# Verify connection
psql $DATABASE_URL -c "\dt"
```

2. **Blockchain Node Sync**
```bash
# Check node status
curl http://localhost:8080/blockchain/health
```

3. **Authentication Failed**
```bash
# Verify DID resolution
curl http://localhost:8080/api/v1/did/resolve/did:example:123

# Check JWT token
curl http://localhost:8080/api/v1/auth/verify-token
```

## Support

- GitHub Issues: [Report a bug](https://github.com/your-org/contract-management-system/issues)
- Documentation: [Full documentation](docs/README.md)
- Community: [Join Discord](https://discord.gg/your-server) 