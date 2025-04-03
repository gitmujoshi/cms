# Contract Management System IAM

This is the Identity and Access Management (IAM) system for the Contract Management System, built on Keycloak.

## Prerequisites

- Docker
- Docker Compose
- jq (for JSON processing in scripts)

## Setup Instructions

1. Start the services:
```bash
cd docker
docker-compose up -d
```

2. Import the realm configuration:
```bash
cd ../scripts
chmod +x import-realm.sh
./import-realm.sh
```

3. Access Keycloak Admin Console:
- URL: http://localhost:8080
- Username: admin
- Password: admin

## Configuration

### Realm Settings
- Realm Name: contract-management
- Default Roles: user, manager, admin

### Client Configuration
- Client ID: contract-management-client
- Access Type: confidential
- Standard Flow Enabled: true
- Direct Access Grants Enabled: true

## Security Features

1. **Authentication**
   - Username/Password
   - OAuth 2.0
   - OpenID Connect
   - Social Login (configurable)

2. **Authorization**
   - Role-based Access Control (RBAC)
   - Group-based permissions
   - Fine-grained access control

3. **Security Features**
   - Password policies
   - Brute force protection
   - Session management
   - Audit logging

## Integration

### API Endpoints
- Authorization: http://localhost:8080/auth/realms/contract-management/protocol/openid-connect/auth
- Token: http://localhost:8080/auth/realms/contract-management/protocol/openid-connect/token
- User Info: http://localhost:8080/auth/realms/contract-management/protocol/openid-connect/userinfo

### Client Credentials
To obtain client credentials:
1. Log in to the Keycloak Admin Console
2. Navigate to Clients > contract-management-client
3. Go to the Credentials tab
4. Copy the Client ID and Secret

## Maintenance

### Backup
```bash
docker-compose exec postgres pg_dump -U keycloak keycloak > backup.sql
```

### Restore
```bash
docker-compose exec -T postgres psql -U keycloak keycloak < backup.sql
```

## Troubleshooting

1. **Service Not Starting**
   - Check Docker logs: `docker-compose logs`
   - Verify port availability
   - Check database connection

2. **Authentication Issues**
   - Verify client credentials
   - Check realm configuration
   - Validate token expiration

## Security Notes

- Change default admin credentials in production
- Enable HTTPS in production
- Configure proper CORS settings
- Set up proper password policies
- Enable MFA for sensitive operations 