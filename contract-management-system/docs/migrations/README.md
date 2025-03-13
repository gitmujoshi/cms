# Database Migrations Guide

## Overview
This guide covers the database migration system for the Digital Contract Management System (DCMS). Migrations are used to manage database schema changes and data transformations in a version-controlled, repeatable manner.

## Migration Structure

### Directory Layout
```
migrations/
├── YYYYMMDDHHMMSS_migration_name.up.sql   # Forward migration
├── YYYYMMDDHHMMSS_migration_name.down.sql  # Rollback migration
└── schema.rs                              # Generated schema file
```

### Naming Convention
- Format: `YYYYMMDDHHMMSS_descriptive_name.up.sql`
- Example: `20240320000000_create_iam_tables.up.sql`

## Creating Migrations

### 1. Generate Migration Files

```bash
# Generate new migration files
cargo run --bin migration-cli -- create "migration_name"
```

This creates two files:
- `YYYYMMDDHHMMSS_migration_name.up.sql`: Forward migration
- `YYYYMMDDHHMMSS_migration_name.down.sql`: Rollback migration

### 2. Writing Migrations

#### Forward Migration (`.up.sql`)
```sql
-- Example: Creating a new table
CREATE TABLE IF NOT EXISTS users (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    email VARCHAR(255) NOT NULL UNIQUE,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP
);

-- Add indexes
CREATE INDEX idx_users_email ON users(email);

-- Add foreign key constraints
ALTER TABLE user_roles
ADD CONSTRAINT fk_user_roles_user_id
FOREIGN KEY (user_id) REFERENCES users(id)
ON DELETE CASCADE;
```

#### Rollback Migration (`.down.sql`)
```sql
-- Remove in reverse order
ALTER TABLE user_roles
DROP CONSTRAINT IF EXISTS fk_user_roles_user_id;

DROP INDEX IF EXISTS idx_users_email;

DROP TABLE IF EXISTS users;
```

### 3. Best Practices

#### Schema Changes
1. Always include both `up` and `down` migrations
2. Make migrations idempotent using `IF EXISTS/IF NOT EXISTS`
3. Add appropriate indexes for performance
4. Consider foreign key constraints
5. Include default values where appropriate

#### Data Migrations
1. Handle large datasets in batches
2. Include progress logging
3. Make operations reversible
4. Consider data validation
5. Handle NULL values appropriately

## Running Migrations

### 1. Development Environment

```bash
# Apply all pending migrations
cargo run --bin migration-cli -- up

# Rollback last migration
cargo run --bin migration-cli -- down

# Rollback specific number of migrations
cargo run --bin migration-cli -- down 2

# Check migration status
cargo run --bin migration-cli -- status
```

### 2. Production Environment

```bash
# Check pending migrations
cargo run --bin migration-cli -- status --env production

# Apply migrations with confirmation
cargo run --bin migration-cli -- up --env production --confirm

# Emergency rollback
cargo run --bin migration-cli -- down --env production --confirm
```

### 3. CI/CD Pipeline

```bash
# Non-interactive migration in CI
cargo run --bin migration-cli -- up --non-interactive

# Verify migrations
cargo run --bin migration-cli -- verify
```

## Migration Table

The system maintains a `_migrations` table to track applied migrations:

```sql
CREATE TABLE _migrations (
    id SERIAL PRIMARY KEY,
    version VARCHAR(255) NOT NULL,
    name VARCHAR(255) NOT NULL,
    applied_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    checksum VARCHAR(64) NOT NULL,
    execution_time INTEGER NOT NULL
);
```

## Error Handling

### Common Issues and Solutions

1. **Migration Failed Mid-way**
   ```bash
   # Check migration status
   cargo run --bin migration-cli -- status
   
   # Repair broken migration
   cargo run --bin migration-cli -- repair
   ```

2. **Version Conflicts**
   ```bash
   # Force specific version
   cargo run --bin migration-cli -- force VERSION
   ```

3. **Checksum Mismatch**
   ```bash
   # Verify migration integrity
   cargo run --bin migration-cli -- verify
   
   # Update checksums
   cargo run --bin migration-cli -- repair --update-checksums
   ```

## Testing Migrations

### 1. Unit Tests
```rust
#[cfg(test)]
mod tests {
    #[test]
    fn test_migration_up() {
        let mut conn = establish_test_connection();
        assert!(run_migration_up(&mut conn).is_ok());
        // Verify schema changes
    }

    #[test]
    fn test_migration_down() {
        let mut conn = establish_test_connection();
        assert!(run_migration_down(&mut conn).is_ok());
        // Verify rollback
    }
}
```

### 2. Integration Tests
```bash
# Run migration tests
cargo test --test '*_migration'

# Test with production data subset
cargo test --test '*_migration' -- --production-data
```

## Backup and Recovery

### 1. Pre-migration Backup
```bash
# Create backup before migration
cargo run --bin migration-cli -- backup

# Apply migration with automatic backup
cargo run --bin migration-cli -- up --with-backup
```

### 2. Recovery Process
```bash
# Restore from backup
cargo run --bin migration-cli -- restore BACKUP_ID

# Verify after restore
cargo run --bin migration-cli -- verify
```

## Performance Considerations

### 1. Large Tables
- Use batching for data migrations
- Consider off-peak execution
- Monitor system resources
- Use appropriate indexes

### 2. Locking
- Minimize table locks
- Use transaction blocks appropriately
- Consider online schema changes

### 3. Monitoring
- Track execution time
- Monitor system metrics
- Log progress for long migrations

## Security

### 1. Access Control
- Use least privilege accounts
- Audit migration execution
- Secure credential management

### 2. Validation
- Validate data integrity
- Check constraints
- Verify foreign keys

## Troubleshooting

### Common Commands
```bash
# Get detailed migration info
cargo run --bin migration-cli -- info VERSION

# Show migration logs
cargo run --bin migration-cli -- logs

# Repair broken state
cargo run --bin migration-cli -- repair
```

### Debugging Tips
1. Check migration logs
2. Verify database connections
3. Monitor system resources
4. Review transaction logs
5. Check for locks 