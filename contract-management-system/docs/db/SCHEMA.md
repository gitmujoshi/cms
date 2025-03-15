# Database Schema Reference

## Overview

The Contract Management System uses PostgreSQL as its primary database. This document details the database schema, relationships, and indexing strategies.

## Tables

### Contracts

```sql
CREATE TABLE contracts (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    title VARCHAR(255) NOT NULL,
    description TEXT,
    provider_id UUID NOT NULL REFERENCES identities(id),
    consumer_id UUID NOT NULL REFERENCES identities(id),
    status contract_status NOT NULL DEFAULT 'draft',
    terms JSONB NOT NULL,
    signatures JSONB,
    metadata JSONB,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_contracts_provider ON contracts(provider_id);
CREATE INDEX idx_contracts_consumer ON contracts(consumer_id);
CREATE INDEX idx_contracts_status ON contracts(status);
CREATE INDEX idx_contracts_created_at ON contracts(created_at);
```

### Identities

```sql
CREATE TABLE identities (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name VARCHAR(255) NOT NULL UNIQUE,
    type identity_type NOT NULL,
    status identity_status NOT NULL DEFAULT 'active',
    metadata JSONB,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_identities_type ON identities(type);
CREATE INDEX idx_identities_status ON identities(status);
```

### Credentials

```sql
CREATE TABLE credentials (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    identity_id UUID NOT NULL REFERENCES identities(id),
    type credential_type NOT NULL,
    value TEXT NOT NULL,
    expires_at TIMESTAMPTZ NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    metadata JSONB
);

CREATE INDEX idx_credentials_identity ON credentials(identity_id);
CREATE INDEX idx_credentials_expires ON credentials(expires_at);
```

### Training Jobs

```sql
CREATE TABLE training_jobs (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    model_name VARCHAR(255) NOT NULL,
    status job_status NOT NULL DEFAULT 'pending',
    config JSONB NOT NULL,
    metrics JSONB,
    privacy_budget FLOAT NOT NULL,
    privacy_budget_spent FLOAT DEFAULT 0.0,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    completed_at TIMESTAMPTZ
);

CREATE INDEX idx_training_jobs_status ON training_jobs(status);
CREATE INDEX idx_training_jobs_model ON training_jobs(model_name);
```

### Audit Logs

```sql
CREATE TABLE audit_logs (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    event_type VARCHAR(50) NOT NULL,
    resource_type VARCHAR(50) NOT NULL,
    resource_id UUID NOT NULL,
    actor_id UUID NOT NULL REFERENCES identities(id),
    details JSONB NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_audit_logs_event ON audit_logs(event_type);
CREATE INDEX idx_audit_logs_resource ON audit_logs(resource_type, resource_id);
CREATE INDEX idx_audit_logs_actor ON audit_logs(actor_id);
CREATE INDEX idx_audit_logs_created ON audit_logs(created_at);
```

## Enums

### Contract Status
```sql
CREATE TYPE contract_status AS ENUM (
    'draft',
    'review',
    'signed',
    'active',
    'completed',
    'terminated',
    'expired'
);
```

### Identity Type
```sql
CREATE TYPE identity_type AS ENUM (
    'user',
    'machine',
    'enclave'
);
```

### Identity Status
```sql
CREATE TYPE identity_status AS ENUM (
    'active',
    'inactive',
    'suspended'
);
```

### Job Status
```sql
CREATE TYPE job_status AS ENUM (
    'pending',
    'running',
    'completed',
    'failed',
    'stopped'
);
```

## Relationships

### Contract Relationships
- `provider_id` → `identities(id)`
- `consumer_id` → `identities(id)`

### Credential Relationships
- `identity_id` → `identities(id)`

### Audit Log Relationships
- `actor_id` → `identities(id)`

## Indexing Strategy

### Primary Keys
- All tables use UUID primary keys
- Generated using `gen_random_uuid()`

### Foreign Keys
- Indexed for efficient joins
- Cascade delete disabled for data integrity

### Performance Indexes
- Status fields for filtering
- Timestamp fields for range queries
- JSONB fields use GIN indexes where needed

## JSONB Fields

### Contract Terms
```json
{
    "start_date": "2024-03-20",
    "end_date": "2025-03-20",
    "conditions": [{
        "type": "payment",
        "amount": 1000,
        "currency": "USD",
        "schedule": "monthly"
    }],
    "sla": {
        "availability": 99.9,
        "response_time": 200
    }
}
```

### Training Config
```json
{
    "model": {
        "architecture": "resnet50",
        "parameters": {...}
    },
    "privacy": {
        "epsilon": 1.0,
        "delta": 1e-5,
        "noise_mechanism": "gaussian"
    },
    "training": {
        "batch_size": 32,
        "epochs": 10,
        "learning_rate": 0.001
    }
}
```

## Migrations

### Creating Migrations
```bash
# Generate new migration
cargo run --bin migrations new add_contract_table

# Apply migrations
cargo run --bin migrations up

# Rollback last migration
cargo run --bin migrations down
```

### Migration Files
```rust
use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Implementation
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Implementation
    }
}
```

## Backup and Recovery

### Backup Strategy
1. **Full Backups**
   ```bash
   # Daily full backup
   pg_dump -Fc contract_management > backup.dump
   ```

2. **WAL Archiving**
   ```bash
   # Archive WAL files
   archive_command = 'cp %p /archive/%f'
   ```

### Recovery Procedures
1. **Full Restore**
   ```bash
   pg_restore -d contract_management backup.dump
   ```

2. **Point-in-Time Recovery**
   ```bash
   # Recover to specific timestamp
   recovery_target_time = '2024-03-20 15:00:00'
   ```

## Performance Considerations

### Query Optimization
1. **Use Prepared Statements**
   ```rust
   let stmt = db.prepare("SELECT * FROM contracts WHERE status = $1")?;
   ```

2. **Batch Operations**
   ```rust
   // Use bulk insert
   INSERT INTO audit_logs (event_type, details)
   SELECT unnest($1::text[]), unnest($2::jsonb[])
   ```

3. **Efficient Pagination**
   ```sql
   SELECT * FROM contracts
   WHERE id > last_id
   ORDER BY id
   LIMIT 20;
   ```

### Maintenance
1. **Regular VACUUM**
   ```sql
   VACUUM ANALYZE contracts;
   ```

2. **Index Maintenance**
   ```sql
   REINDEX TABLE contracts;
   ```

## Security

### Access Control
```sql
-- Role-based access
CREATE ROLE app_user;
GRANT SELECT, INSERT ON contracts TO app_user;

-- Row-level security
ALTER TABLE contracts ENABLE ROW LEVEL SECURITY;
CREATE POLICY contract_access ON contracts
    USING (provider_id = current_user_id() OR consumer_id = current_user_id());
```

### Encryption
- Sensitive data encrypted at rest
- TLS required for connections
- Column-level encryption where needed 