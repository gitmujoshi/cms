# Contract Management API

## Overview

The Contract Management API provides endpoints for creating, managing, and tracking digital contracts. All endpoints require authentication using JWT tokens.

## Authentication

All API requests must include an Authorization header:
```
Authorization: Bearer <jwt_token>
```

## Endpoints

### 1. List Contracts

```http
GET /api/v1/contracts
```

#### Query Parameters
- `page` (optional): Page number (default: 1)
- `per_page` (optional): Items per page (default: 10, max: 100)
- `status` (optional): Filter by contract status
- `search` (optional): Search term for title/description

#### Example Request
```bash
curl -X GET "https://api.example.com/api/v1/contracts?page=1&per_page=10" \
     -H "Authorization: Bearer <token>"
```

#### Example Response
```json
{
  "contracts": [
    {
      "id": "550e8400-e29b-41d4-a716-446655440000",
      "title": "Data Processing Agreement",
      "description": "Agreement for processing training data",
      "status": "Active",
      "created_at": "2024-03-21T10:00:00Z",
      "provider_name": "Data Provider Inc",
      "consumer_name": "AI Company LLC"
    }
  ],
  "total": 1,
  "page": 1,
  "per_page": 10
}
```

### 2. Create Contract

```http
POST /api/v1/contracts
```

#### Request Body
```json
{
  "title": "Data Processing Agreement",
  "description": "Agreement for processing training data",
  "contract_type": "DataSharing",
  "provider_id": "550e8400-e29b-41d4-a716-446655440000",
  "consumer_id": "550e8400-e29b-41d4-a716-446655440001",
  "terms": {
    "data_access_scope": ["training_data", "validation_data"],
    "usage_restrictions": ["no_redistribution", "no_commercial_use"],
    "retention_period_days": 365,
    "security_requirements": {
      "encryption_required": true,
      "min_encryption_level": "High",
      "audit_logging_required": true,
      "network_isolation_required": true
    },
    "compliance_requirements": ["GDPR", "CCPA"]
  },
  "valid_from": "2024-03-21T00:00:00Z",
  "valid_until": "2025-03-21T00:00:00Z"
}
```

#### Example Request
```bash
curl -X POST "https://api.example.com/api/v1/contracts" \
     -H "Authorization: Bearer <token>" \
     -H "Content-Type: application/json" \
     -d '{"title": "Data Processing Agreement", ...}'
```

#### Example Response
```json
{
  "id": "550e8400-e29b-41d4-a716-446655440000",
  "status": "Draft",
  "created_at": "2024-03-21T10:00:00Z"
}
```

### 3. Get Contract Details

```http
GET /api/v1/contracts/{id}
```

#### Example Request
```bash
curl -X GET "https://api.example.com/api/v1/contracts/550e8400-e29b-41d4-a716-446655440000" \
     -H "Authorization: Bearer <token>"
```

#### Example Response
```json
{
  "id": "550e8400-e29b-41d4-a716-446655440000",
  "title": "Data Processing Agreement",
  "description": "Agreement for processing training data",
  "status": "Active",
  "parties": [
    {
      "id": "550e8400-e29b-41d4-a716-446655440000",
      "role": "Provider",
      "name": "Data Provider Inc",
      "signed_at": "2024-03-21T10:30:00Z"
    },
    {
      "id": "550e8400-e29b-41d4-a716-446655440001",
      "role": "Consumer",
      "name": "AI Company LLC",
      "signed_at": "2024-03-21T11:00:00Z"
    }
  ],
  "terms": {
    "data_access_scope": ["training_data", "validation_data"],
    "usage_restrictions": ["no_redistribution", "no_commercial_use"],
    "retention_period_days": 365,
    "security_requirements": {
      "encryption_required": true,
      "min_encryption_level": "High",
      "audit_logging_required": true,
      "network_isolation_required": true
    },
    "compliance_requirements": ["GDPR", "CCPA"]
  },
  "valid_from": "2024-03-21T00:00:00Z",
  "valid_until": "2025-03-21T00:00:00Z",
  "created_at": "2024-03-21T10:00:00Z",
  "updated_at": "2024-03-21T11:00:00Z"
}
```

### 4. Sign Contract

```http
POST /api/v1/contracts/{id}/sign
```

#### Request Body
```json
{
  "signature": "base64_encoded_signature",
  "signature_type": "Digital"
}
```

#### Example Request
```bash
curl -X POST "https://api.example.com/api/v1/contracts/550e8400-e29b-41d4-a716-446655440000/sign" \
     -H "Authorization: Bearer <token>" \
     -H "Content-Type: application/json" \
     -d '{"signature": "base64_encoded_signature", "signature_type": "Digital"}'
```

#### Example Response
```json
{
  "status": "PendingSignatures",
  "signed_at": "2024-03-21T10:30:00Z"
}
```

### 5. Terminate Contract

```http
POST /api/v1/contracts/{id}/terminate
```

#### Request Body
```json
{
  "reason": "Contract terms violated",
  "effective_date": "2024-03-21T12:00:00Z"
}
```

#### Example Request
```bash
curl -X POST "https://api.example.com/api/v1/contracts/550e8400-e29b-41d4-a716-446655440000/terminate" \
     -H "Authorization: Bearer <token>" \
     -H "Content-Type: application/json" \
     -d '{"reason": "Contract terms violated", "effective_date": "2024-03-21T12:00:00Z"}'
```

#### Example Response
```json
{
  "status": "Terminated",
  "terminated_at": "2024-03-21T12:00:00Z",
  "reason": "Contract terms violated"
}
```

## Error Handling

All API endpoints return standard HTTP status codes and error responses in the following format:

```json
{
  "error": {
    "code": "ERROR_CODE",
    "message": "Human readable error message",
    "details": {
      "field": "Additional error details"
    }
  }
}
```

### Common Error Codes
- `400`: Bad Request - Invalid input data
- `401`: Unauthorized - Invalid or missing authentication
- `403`: Forbidden - Insufficient permissions
- `404`: Not Found - Resource not found
- `409`: Conflict - Resource state conflict
- `500`: Internal Server Error - Server-side error

## Rate Limiting

API requests are rate-limited to prevent abuse:
- 100 requests per minute per IP
- 1000 requests per hour per user

Rate limit headers are included in all responses:
```
X-RateLimit-Limit: 100
X-RateLimit-Remaining: 99
X-RateLimit-Reset: 1616320000
```

## Versioning

The API is versioned through the URL path:
- Current version: `/api/v1/`
- Future versions will use `/api/v2/`, etc.

## Pagination

List endpoints support cursor-based pagination:
```json
{
  "data": [...],
  "next_cursor": "base64_encoded_cursor",
  "has_more": true
}
```

## Webhooks

The API supports webhooks for contract events:
- Contract created
- Contract signed
- Contract terminated
- Contract expired

Webhook payload format:
```json
{
  "event": "contract.signed",
  "data": {
    "contract_id": "550e8400-e29b-41d4-a716-446655440000",
    "signed_by": "550e8400-e29b-41d4-a716-446655440000",
    "signed_at": "2024-03-21T10:30:00Z"
  },
  "timestamp": "2024-03-21T10:30:00Z"
}
``` 