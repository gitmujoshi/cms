# Database Schema

## Overview

The Contract Management System uses a PostgreSQL database with the following main tables and relationships.

## Tables

### users
```sql
CREATE TABLE users (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    email VARCHAR(255) UNIQUE NOT NULL,
    first_name VARCHAR(100) NOT NULL,
    last_name VARCHAR(100) NOT NULL,
    password_hash VARCHAR(255) NOT NULL,
    role_id UUID REFERENCES roles(id),
    is_active BOOLEAN DEFAULT true,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP
);
```

### roles
```sql
CREATE TABLE roles (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name VARCHAR(100) UNIQUE NOT NULL,
    description TEXT,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP
);
```

### permissions
```sql
CREATE TABLE permissions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name VARCHAR(100) UNIQUE NOT NULL,
    description TEXT,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP
);
```

### role_permissions
```sql
CREATE TABLE role_permissions (
    role_id UUID REFERENCES roles(id),
    permission_id UUID REFERENCES permissions(id),
    PRIMARY KEY (role_id, permission_id)
);
```

### contracts
```sql
CREATE TABLE contracts (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    title VARCHAR(255) NOT NULL,
    description TEXT,
    template_id UUID REFERENCES contract_templates(id),
    status VARCHAR(50) NOT NULL,
    created_by UUID REFERENCES users(id),
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    expires_at TIMESTAMP WITH TIME ZONE,
    blockchain_hash VARCHAR(255)
);
```

### contract_templates
```sql
CREATE TABLE contract_templates (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name VARCHAR(255) NOT NULL,
    description TEXT,
    content TEXT NOT NULL,
    version INTEGER NOT NULL,
    is_active BOOLEAN DEFAULT true,
    created_by UUID REFERENCES users(id),
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP
);
```

### contract_signatures
```sql
CREATE TABLE contract_signatures (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    contract_id UUID REFERENCES contracts(id),
    user_id UUID REFERENCES users(id),
    signature_data TEXT NOT NULL,
    signed_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    blockchain_hash VARCHAR(255)
);
```

### audit_logs
```sql
CREATE TABLE audit_logs (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID REFERENCES users(id),
    action VARCHAR(100) NOT NULL,
    entity_type VARCHAR(100) NOT NULL,
    entity_id UUID NOT NULL,
    details JSONB,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP
);
```

## Indexes

```sql
-- Users
CREATE INDEX idx_users_email ON users(email);
CREATE INDEX idx_users_role ON users(role_id);

-- Contracts
CREATE INDEX idx_contracts_status ON contracts(status);
CREATE INDEX idx_contracts_created_by ON contracts(created_by);
CREATE INDEX idx_contracts_template ON contracts(template_id);

-- Contract Templates
CREATE INDEX idx_templates_name ON contract_templates(name);
CREATE INDEX idx_templates_version ON contract_templates(version);

-- Audit Logs
CREATE INDEX idx_audit_user ON audit_logs(user_id);
CREATE INDEX idx_audit_entity ON audit_logs(entity_type, entity_id);
CREATE INDEX idx_audit_created ON audit_logs(created_at);
```

## Constraints

```sql
-- Contract Status Check
ALTER TABLE contracts
ADD CONSTRAINT check_contract_status
CHECK (status IN ('draft', 'pending', 'active', 'expired', 'terminated'));

-- Template Version Check
ALTER TABLE contract_templates
ADD CONSTRAINT check_template_version
CHECK (version > 0);

-- Unique Template Version
ALTER TABLE contract_templates
ADD CONSTRAINT unique_template_version
UNIQUE (name, version);
```

## Relationships

1. **Users to Roles**: Many-to-One
   - One user can have one role
   - One role can have many users

2. **Roles to Permissions**: Many-to-Many
   - One role can have many permissions
   - One permission can belong to many roles

3. **Contracts to Templates**: Many-to-One
   - One contract is based on one template
   - One template can be used for many contracts

4. **Contracts to Users**: Many-to-One
   - One contract is created by one user
   - One user can create many contracts

5. **Contract Signatures to Contracts**: Many-to-One
   - One signature belongs to one contract
   - One contract can have many signatures

## Data Types

1. **UUID**: Used for primary keys
2. **VARCHAR**: Used for text fields with length limits
3. **TEXT**: Used for unlimited text fields
4. **TIMESTAMP WITH TIME ZONE**: Used for all date/time fields
5. **BOOLEAN**: Used for true/false flags
6. **JSONB**: Used for flexible data storage (audit details)

## Validation Rules

1. **Email Format**: Must be a valid email address
2. **Password**: Must meet security requirements
3. **Contract Status**: Must be one of the allowed values
4. **Template Version**: Must be positive integer
5. **Dates**: Must be valid timestamps

## Security

1. **Row Level Security**: Enabled on sensitive tables
2. **Encryption**: Sensitive data is encrypted at rest
3. **Access Control**: Based on user roles and permissions
4. **Audit Trail**: All changes are logged 