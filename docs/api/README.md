# Contract Management System API Documentation

## Overview

The Contract Management System API provides a comprehensive set of endpoints for managing contracts, documents, and related operations. This documentation covers all available endpoints, authentication methods, and example usage.

## Table of Contents

1. [Authentication](#authentication)
2. [Base URL](#base-url)
3. [Endpoints](#endpoints)
4. [Error Handling](#error-handling)
5. [Rate Limiting](#rate-limiting)
6. [Webhooks](#webhooks)

## Authentication

### JWT Authentication
All API requests must include a valid JWT token in the Authorization header:

```http
Authorization: Bearer <your_jwt_token>
```

### Obtaining a Token
1. Send a POST request to `/auth/login`:
```http
POST /auth/login
Content-Type: application/json

{
    "email": "user@example.com",
    "password": "your_password"
}
```

2. Response:
```json
{
    "token": "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...",
    "expires_in": 3600
}
```

## Base URL

The base URL for all API endpoints is:
```
https://api.contractmanagement.com/v1
```

## Endpoints

### Contracts

#### List Contracts
```http
GET /contracts
```

Query Parameters:
- `page` (optional): Page number (default: 1)
- `limit` (optional): Items per page (default: 20)
- `status` (optional): Filter by status
- `type` (optional): Filter by contract type

Response:
```json
{
    "data": [
        {
            "id": "contract_123",
            "title": "Service Agreement",
            "status": "active",
            "created_at": "2024-03-20T10:00:00Z",
            "parties": [
                {
                    "name": "Company A",
                    "role": "client"
                },
                {
                    "name": "Company B",
                    "role": "provider"
                }
            ]
        }
    ],
    "pagination": {
        "total": 100,
        "page": 1,
        "limit": 20
    }
}
```

#### Create Contract
```http
POST /contracts
Content-Type: application/json

{
    "title": "New Service Agreement",
    "template_id": "template_123",
    "parties": [
        {
            "name": "Company A",
            "email": "contact@companya.com",
            "role": "client"
        }
    ],
    "terms": {
        "start_date": "2024-04-01",
        "end_date": "2025-03-31",
        "payment_terms": "Net 30"
    }
}
```

Response:
```json
{
    "id": "contract_456",
    "status": "draft",
    "created_at": "2024-03-20T11:00:00Z",
    "url": "https://contractmanagement.com/contracts/contract_456"
}
```

#### Get Contract Details
```http
GET /contracts/{contract_id}
```

Response:
```json
{
    "id": "contract_123",
    "title": "Service Agreement",
    "status": "active",
    "created_at": "2024-03-20T10:00:00Z",
    "updated_at": "2024-03-20T11:00:00Z",
    "parties": [
        {
            "name": "Company A",
            "email": "contact@companya.com",
            "role": "client",
            "signed_at": "2024-03-20T10:30:00Z"
        }
    ],
    "documents": [
        {
            "id": "doc_123",
            "name": "agreement.pdf",
            "type": "contract",
            "url": "https://contractmanagement.com/documents/doc_123"
        }
    ]
}
```

### Documents

#### Upload Document
```http
POST /documents
Content-Type: multipart/form-data

{
    "file": <binary_file>,
    "contract_id": "contract_123",
    "type": "supporting",
    "description": "Supporting documentation"
}
```

Response:
```json
{
    "id": "doc_456",
    "name": "document.pdf",
    "size": 1024000,
    "url": "https://contractmanagement.com/documents/doc_456"
}
```

#### Get Document
```http
GET /documents/{document_id}
```

Response:
```json
{
    "id": "doc_123",
    "name": "agreement.pdf",
    "type": "contract",
    "size": 1024000,
    "created_at": "2024-03-20T10:00:00Z",
    "url": "https://contractmanagement.com/documents/doc_123"
}
```

## Error Handling

### Error Response Format
```json
{
    "error": {
        "code": "ERROR_CODE",
        "message": "Human-readable error message",
        "details": {
            "field": "specific error details"
        }
    }
}
```

### Common Error Codes
- `AUTH_REQUIRED`: Authentication required
- `INVALID_TOKEN`: Invalid or expired token
- `NOT_FOUND`: Resource not found
- `VALIDATION_ERROR`: Invalid request data
- `RATE_LIMITED`: Too many requests

## Rate Limiting

- 100 requests per minute per user
- 1000 requests per minute per IP
- Headers included in responses:
  ```http
  X-RateLimit-Limit: 100
  X-RateLimit-Remaining: 95
  X-RateLimit-Reset: 1616245200
  ```

## Webhooks

### Available Events
- `contract.created`
- `contract.updated`
- `contract.signed`
- `document.uploaded`
- `document.deleted`

### Webhook Payload
```json
{
    "event": "contract.signed",
    "data": {
        "contract_id": "contract_123",
        "signed_by": "user@example.com",
        "signed_at": "2024-03-20T10:30:00Z"
    },
    "timestamp": "2024-03-20T10:30:00Z"
}
```

### Security
- Webhook endpoints must be HTTPS
- Include signature header for verification:
  ```http
  X-Contract-Signature: sha256=<signature>
  ``` 