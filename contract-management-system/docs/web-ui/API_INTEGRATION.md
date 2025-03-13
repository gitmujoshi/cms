# API Integration Documentation

## Overview
The API integration layer handles all communication between the web UI and the backend services. It provides a type-safe and secure way to interact with the server-side APIs.

## Architecture

### Module Structure
```
api/
├── mod.rs           # Main API module
├── endpoints.rs     # API endpoint definitions
├── types.rs        # Shared request/response types
├── auth.rs         # Authentication handling
└── error.rs        # Error types and handling
```

## Core Features

### HTTP Client
- Type-safe request/response handling
- Automatic JSON serialization/deserialization
- Error handling and retry logic
- Request interceptors
- Response transformers

### Authentication
- JWT token management
- Token refresh handling
- Session management
- Secure token storage
- Authorization header injection

### Error Handling
```rust
#[derive(Debug, Serialize, Deserialize)]
pub struct ApiError {
    pub code: String,
    pub message: String,
    pub details: Option<Value>,
}

pub type ApiResult<T> = Result<T, ApiError>;
```

## Endpoints

### Authentication
```rust
// Login
POST /api/auth/login
Content-Type: application/json
{
    "email": string,
    "password": string
}

// Register
POST /api/register
Content-Type: application/json
{
    "organization_name": string,
    "email": string,
    "password": string,
    "participant_type": string[],
    "contact_info": {
        "full_name": string,
        "phone": string,
        "address": string
    }
}
```

### Contract Management
```rust
// List contracts
GET /api/contracts
Authorization: Bearer <token>

// Create contract
POST /api/contracts
Authorization: Bearer <token>
Content-Type: application/json
{
    "template_id": string,
    "parties": Party[],
    "terms": object
}

// Get contract details
GET /api/contracts/:id
Authorization: Bearer <token>
```

## Usage Examples

### Making API Calls
```rust
// Basic GET request
pub async fn get_contracts() -> ApiResult<Vec<Contract>> {
    api::get("/api/contracts").await
}

// POST with body
pub async fn create_contract(contract: NewContract) -> ApiResult<Contract> {
    api::post("/api/contracts", &contract).await
}

// With query parameters
pub async fn search_contracts(query: &str) -> ApiResult<Vec<Contract>> {
    api::get_with_params("/api/contracts/search", &[("q", query)]).await
}
```

### Error Handling
```rust
match api::post("/api/contracts", &new_contract).await {
    Ok(contract) => {
        // Handle success
    }
    Err(ApiError { code, message, .. }) => {
        match code.as_str() {
            "UNAUTHORIZED" => redirect_to_login(),
            "VALIDATION_ERROR" => show_validation_errors(message),
            _ => show_error_message(message),
        }
    }
}
```

## Security

### Request Security
- CSRF token inclusion
- Content-Security-Policy compliance
- XSS prevention
- Input validation
- Rate limiting handling

### Response Security
- Response validation
- Secure error handling
- Sensitive data filtering
- Token security

## Testing

### Unit Tests
```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_api_authentication() {
        // Test authentication flow
    }

    #[tokio::test]
    async fn test_error_handling() {
        // Test error scenarios
    }
}
```

### Mock Server
```rust
// Configure mock server for testing
pub fn setup_mock_server() -> MockServer {
    let server = MockServer::start();
    server.mock(|when, then| {
        when.method(POST)
            .path("/api/contracts");
        then.status(201)
            .json_body(json!({
                "id": "contract-123",
                "status": "draft"
            }));
    });
    server
}
```

## Performance

### Optimization Strategies
- Request caching
- Connection pooling
- Request batching
- Response compression
- Lazy loading

### Monitoring
- Request timing
- Error tracking
- Performance metrics
- Network usage
- Cache hit rates

## Future Enhancements
- GraphQL integration
- WebSocket support
- Offline support
- Request queuing
- Advanced caching strategies 