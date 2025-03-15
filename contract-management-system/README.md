# Contract Management System

A robust contract management system built with Rust, featuring user management, organization management, and comprehensive contract lifecycle management.

## Features

### User Management
- User authentication with JWT
- Role-based access control (Admin, Manager, User)
- User status management (Active, Inactive, Suspended)
- Password hashing with Argon2
- Organization-based user grouping

### Organization Management
- Organization types (Business, Government, Non-Profit, Individual)
- Organization status management
- Contact information and address management
- Organization-level access control

### Contract Management
- Full contract lifecycle management
  - Draft creation
  - Signature collection
  - Contract activation
  - Contract suspension
  - Contract termination
- Contract types and terms
- Provider and consumer roles
- Digital signature support
- Contract validity period tracking

### Audit Logging
- Comprehensive audit trail for all operations
- User action tracking
- Resource modification history
- IP address and user agent logging

## Technology Stack

- **Backend Framework**: Actix-web
- **Database**: PostgreSQL with SeaORM
- **Authentication**: JWT (JSON Web Tokens)
- **Password Hashing**: Argon2
- **API Documentation**: OpenAPI/Swagger
- **Frontend**: Leptos (Web UI)

## Project Structure

```
contract-management-system/
├── src/
│   ├── api/           # API endpoints
│   ├── models/        # Database models
│   ├── services/      # Business logic
│   ├── auth/          # Authentication
│   └── error.rs       # Error handling
├── migrations/        # Database migrations
├── docs/             # Documentation
│   └── api/          # API documentation
└── web-ui/           # Frontend application
```

## Getting Started

### Prerequisites

- Rust (latest stable version)
- PostgreSQL
- Node.js (for web UI development)

### Setup

1. Clone the repository:
   ```bash
   git clone https://github.com/yourusername/contract-management-system.git
   cd contract-management-system
   ```

2. Set up the database:
   ```bash
   # Create PostgreSQL database
   createdb contract_management
   
   # Run migrations
   cargo run --bin migrate
   ```

3. Configure environment variables:
   ```bash
   cp .env.example .env
   # Edit .env with your configuration
   ```

4. Run the application:
   ```bash
   # Run the backend server
   cargo run

   # In another terminal, run the web UI
   cd web-ui
   trunk serve
   ```

5. Access the application:
   - Backend API: http://localhost:8080
   - Web UI: http://localhost:8081
   - API Documentation: http://localhost:8080/api/docs

## API Documentation

The API is documented using OpenAPI/Swagger specification. You can view the interactive documentation at:

- Development: http://localhost:8080/api/docs
- Production: https://your-domain.com/api/docs

### API Examples

#### Authentication
```bash
# Login
curl -X POST http://localhost:8080/api/v1/auth/login \
  -H "Content-Type: application/json" \
  -d '{
    "email": "user@example.com",
    "password": "your-password"
  }'
```

#### Organizations
```bash
# List organizations
curl -X GET http://localhost:8080/api/v1/organizations \
  -H "Authorization: Bearer YOUR_JWT_TOKEN"

# Create organization
curl -X POST http://localhost:8080/api/v1/organizations \
  -H "Authorization: Bearer YOUR_JWT_TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "name": "Example Corp",
    "type": "business",
    "email": "contact@example.com"
  }'

# Get organization details
curl -X GET http://localhost:8080/api/v1/organizations/YOUR_ORG_ID \
  -H "Authorization: Bearer YOUR_JWT_TOKEN"
```

#### Users
```bash
# List users
curl -X GET http://localhost:8080/api/v1/users \
  -H "Authorization: Bearer YOUR_JWT_TOKEN"

# Create user
curl -X POST http://localhost:8080/api/v1/users \
  -H "Authorization: Bearer YOUR_JWT_TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "organization_id": "YOUR_ORG_ID",
    "email": "newuser@example.com",
    "password": "user-password",
    "first_name": "John",
    "last_name": "Doe",
    "role": "user"
  }'
```

#### Contracts
```bash
# List contracts
curl -X GET http://localhost:8080/api/v1/contracts \
  -H "Authorization: Bearer YOUR_JWT_TOKEN"

# Get contract details
curl -X GET http://localhost:8080/api/v1/contracts/YOUR_CONTRACT_ID \
  -H "Authorization: Bearer YOUR_JWT_TOKEN"

# Sign contract
curl -X POST http://localhost:8080/api/v1/contracts/YOUR_CONTRACT_ID/sign \
  -H "Authorization: Bearer YOUR_JWT_TOKEN"
```

For more details, see [API Documentation](docs/api/openapi.yaml).

## Development Guide

### Setting Up Development Environment

1. **Install Required Tools**
   ```bash
   # Install Rust
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
   
   # Install PostgreSQL
   brew install postgresql@14  # macOS
   sudo apt install postgresql-14  # Ubuntu/Debian
   
   # Install development tools
   cargo install cargo-watch  # For development auto-reload
   cargo install sea-orm-cli  # For database migrations
   cargo install trunk  # For web UI development
   ```

2. **Database Setup**
   ```bash
   # Start PostgreSQL service
   brew services start postgresql  # macOS
   sudo service postgresql start   # Ubuntu/Debian
   
   # Create development database
   createdb contract_management_dev
   
   # Run migrations
   sea-orm-cli migrate up
   ```

3. **Environment Configuration**
   - Copy `.env.example` to `.env`
   - Update database connection string
   - Generate and set JWT secret
   - Configure other environment variables as needed

### Development Workflow

1. **Running in Development Mode**
   ```bash
   # Terminal 1: Run backend with auto-reload
   cargo watch -x run
   
   # Terminal 2: Run web UI with hot reload
   cd web-ui
   trunk serve
   ```

2. **Database Migrations**
   ```bash
   # Create new migration
   sea-orm-cli migrate generate create_new_table
   
   # Apply migrations
   sea-orm-cli migrate up
   
   # Rollback last migration
   sea-orm-cli migrate down
   ```

3. **Code Quality**
   ```bash
   # Format code
   cargo fmt
   
   # Run clippy linter
   cargo clippy -- -D warnings
   
   # Run tests
   cargo test
   
   # Run specific test
   cargo test test_name
   ```

### Testing

1. **Unit Tests**
   - Write tests in the same file as the code
   - Use `#[cfg(test)]` module attribute
   - Follow naming convention: `test_<function_name>_<scenario>`

2. **Integration Tests**
   - Create tests in `tests/` directory
   - Use `#[tokio::test]` for async tests
   - Mock external services when necessary

3. **API Tests**
   - Use `actix_web::test` for HTTP tests
   - Create test database for integration tests
   - Clean up test data after each test

Example test:
```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_create_contract_success() {
        // Setup
        let app = test::init_service(
            App::new()
                .service(create_contract)
        ).await;
        
        // Execute
        let req = test::TestRequest::post()
            .uri("/contracts")
            .set_json(&contract_data)
            .to_request();
        let resp = test::call_service(&app, req).await;
        
        // Verify
        assert_eq!(resp.status(), StatusCode::CREATED);
    }
}
```

### Debugging

1. **Logging**
   - Use `log` crate macros (`error!`, `warn!`, `info!`, `debug!`, `trace!`)
   - Configure log level in `.env` file
   - Use `RUST_LOG=debug` for detailed logging

2. **Error Handling**
   - Use custom error types
   - Implement `std::error::Error` trait
   - Add context to errors using `anyhow`

3. **Database Debugging**
   - Enable SQL query logging
   - Use database transaction for atomic operations
   - Add indexes for performance optimization

### Performance Optimization

1. **Database**
   - Use appropriate indexes
   - Implement pagination
   - Use database transactions
   - Optimize queries using EXPLAIN

2. **Caching**
   - Implement Redis caching for frequently accessed data
   - Use in-memory caching for static data
   - Add cache headers for API responses

3. **API**
   - Implement rate limiting
   - Use connection pooling
   - Optimize payload size
   - Enable compression

### Security Best Practices

1. **Authentication**
   - Use secure password hashing (Argon2)
   - Implement JWT token rotation
   - Add rate limiting for auth endpoints
   - Use secure session management

2. **Authorization**
   - Implement role-based access control
   - Validate user permissions
   - Use principle of least privilege
   - Add audit logging

3. **Data Protection**
   - Encrypt sensitive data
   - Sanitize user input
   - Implement CORS policies
   - Use HTTPS in production

## Database Schema

### Organizations
- id (UUID)
- name (String)
- description (Optional String)
- type (String: business, government, non_profit, individual)
- status (String: active, inactive, suspended)
- website (Optional String)
- email (String)
- phone (Optional String)
- address (Optional JSON)
- created_at (Timestamp)
- updated_at (Timestamp)

### Users
- id (UUID)
- organization_id (UUID)
- email (String)
- password_hash (String)
- first_name (String)
- last_name (String)
- role (String: admin, manager, user)
- status (String: active, inactive, suspended)
- last_login_at (Optional Timestamp)
- created_at (Timestamp)
- updated_at (Timestamp)

### Contracts
- id (UUID)
- title (String)
- description (String)
- contract_type (String)
- provider_id (UUID)
- consumer_id (UUID)
- terms (JSON)
- status (String: draft, pending_signature, active, suspended, terminated)
- valid_from (Timestamp)
- valid_until (Optional Timestamp)
- created_at (Timestamp)
- updated_at (Timestamp)

### Contract Signatures
- id (UUID)
- contract_id (UUID)
- signer_id (UUID)
- signature_type (String)
- signature (String)
- signed_at (Timestamp)

### Audit Logs
- id (UUID)
- user_id (UUID)
- resource_type (String)
- resource_id (Optional UUID)
- action (String)
- details (JSON)
- ip_address (Optional String)
- user_agent (Optional String)
- created_at (Timestamp)

## Contributing

1. Fork the repository
2. Create your feature branch (`git checkout -b feature/amazing-feature`)
3. Commit your changes (`git commit -m 'Add some amazing feature'`)
4. Push to the branch (`git push origin feature/amazing-feature`)
5. Open a Pull Request

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.