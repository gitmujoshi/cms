# API Reference

## Overview

The Contract Management System exposes RESTful APIs for managing contracts, identities, and secure training operations.

## Authentication

All API endpoints require authentication using JWT tokens:

```bash
Authorization: Bearer <jwt_token>
```

## API Endpoints

### Contract Management

#### Create Contract
```http
POST /api/v1/contracts
Content-Type: application/json

{
    "title": "Service Agreement",
    "description": "...",
    "provider_id": "uuid",
    "consumer_id": "uuid",
    "terms": {
        "start_date": "2024-03-20",
        "end_date": "2025-03-20",
        "conditions": [...]
    }
}
```

#### List Contracts
```http
GET /api/v1/contracts?status=active&page=1&limit=10
```

#### Get Contract
```http
GET /api/v1/contracts/{contract_id}
```

#### Update Contract
```http
PUT /api/v1/contracts/{contract_id}
Content-Type: application/json

{
    "status": "signed",
    "terms": {...}
}
```

### Identity Management

#### Create Identity
```http
POST /api/v1/identities
Content-Type: application/json

{
    "name": "service-account-1",
    "type": "machine",
    "metadata": {
        "department": "engineering",
        "purpose": "model-training"
    }
}
```

#### Rotate Credentials
```http
POST /api/v1/identities/{identity_id}/rotate
```

#### Verify Attestation
```http
POST /api/v1/identities/{identity_id}/verify
Content-Type: application/json

{
    "attestation_doc": "...",
    "signature": "..."
}
```

### Secure Training

#### Initialize Training
```http
POST /api/v1/training/jobs
Content-Type: application/json

{
    "model_name": "privacy-preserving-model",
    "privacy_config": {
        "epsilon": 1.0,
        "delta": 1e-5,
        "noise_mechanism": "gaussian"
    },
    "training_config": {
        "batch_size": 32,
        "epochs": 10,
        "learning_rate": 0.001
    }
}
```

#### Get Training Status
```http
GET /api/v1/training/jobs/{job_id}
```

#### Stop Training
```http
POST /api/v1/training/jobs/{job_id}/stop
```

## Response Formats

### Success Response
```json
{
    "status": "success",
    "data": {
        ...
    }
}
```

### Error Response
```json
{
    "status": "error",
    "error": {
        "code": "CONTRACT_NOT_FOUND",
        "message": "Contract with ID xyz not found",
        "details": {...}
    }
}
```

## Error Codes

| Code | Description |
|------|-------------|
| `INVALID_REQUEST` | Request validation failed |
| `UNAUTHORIZED` | Authentication required |
| `FORBIDDEN` | Insufficient permissions |
| `NOT_FOUND` | Resource not found |
| `CONFLICT` | Resource conflict |
| `INTERNAL_ERROR` | Server error |

## Rate Limiting

API endpoints are rate-limited:
- 100 requests per minute for authenticated users
- 10 requests per minute for unauthenticated users

Rate limit headers:
```http
X-RateLimit-Limit: 100
X-RateLimit-Remaining: 95
X-RateLimit-Reset: 1584722400
```

## Pagination

List endpoints support pagination:

```http
GET /api/v1/contracts?page=2&limit=20
```

Response includes pagination metadata:
```json
{
    "status": "success",
    "data": [...],
    "pagination": {
        "total": 100,
        "page": 2,
        "limit": 20,
        "pages": 5
    }
}
```

## WebSocket APIs

### Real-time Training Metrics
```javascript
// Connect to WebSocket
ws://api.example.com/ws/v1/training/{job_id}/metrics

// Message format
{
    "type": "metric",
    "data": {
        "loss": 0.234,
        "accuracy": 0.956,
        "epoch": 5,
        "privacy_budget_remaining": 0.8
    }
}
```

## SDK Examples

### Rust
```rust
use contract_management_sdk::{Client, ContractBuilder};

async fn create_contract() -> Result<Contract> {
    let client = Client::new(API_KEY);
    
    let contract = ContractBuilder::new()
        .title("Service Agreement")
        .provider(provider_id)
        .consumer(consumer_id)
        .terms(terms)
        .build()?;
        
    client.contracts().create(contract).await
}
```

### Python
```python
from contract_management import Client

client = Client(api_key="...")

# Create contract
contract = client.contracts.create(
    title="Service Agreement",
    provider_id="...",
    consumer_id="...",
    terms={...}
)
```

## Versioning

The API follows semantic versioning:
- Major version changes (/v2/) indicate breaking changes
- Minor version changes maintain backward compatibility
- API version is specified in the URL path

## Security

1. **Authentication**
   - JWT-based authentication
   - Token expiration and rotation
   - Multi-factor authentication support

2. **Authorization**
   - Role-based access control
   - Fine-grained permissions
   - Resource-level access control

3. **Data Protection**
   - TLS 1.3 required
   - Request signing for sensitive operations
   - Payload encryption for sensitive data

## Best Practices

1. **Error Handling**
   - Always check error responses
   - Implement exponential backoff for retries
   - Handle rate limiting appropriately

2. **Performance**
   - Use pagination for large datasets
   - Implement caching where appropriate
   - Minimize payload sizes

3. **Security**
   - Rotate API keys regularly
   - Validate all input data
   - Use minimum required permissions 