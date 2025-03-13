# Migration CLI Tool Guide

## Overview
The Migration CLI tool is a command-line utility for managing database migrations in the DCMS system. It provides commands for creating, running, and managing database migrations.

## Installation

```bash
# Build the migration CLI tool
cargo build --bin migration-cli

# Optional: Install globally
cargo install --path ./crates/migration-cli
```

## Configuration

### Environment Variables
Create a `.env` file in your project root:
```env
DATABASE_URL=postgres://user:password@localhost:5432/dcms
MIGRATION_DIR=./migrations
LOG_LEVEL=info
```

### Configuration File
Create `migration-config.toml`:
```toml
[database]
url = "postgres://user:password@localhost:5432/dcms"
max_connections = 10
connection_timeout = 30

[migrations]
directory = "./migrations"
table_name = "_migrations"
lock_timeout = 300

[logging]
level = "info"
file = "migrations.log"
```

## Commands Reference

### Create Migration

Create a new migration file:
```bash
# Basic usage
migration-cli create "create_users_table"

# With template
migration-cli create "add_user_roles" --template "table"

# With description
migration-cli create "update_user_schema" --description "Add email verification fields"
```

### Run Migrations

Apply migrations:
```bash
# Apply all pending migrations
migration-cli up

# Apply specific number of migrations
migration-cli up 2

# Apply migrations up to specific version
migration-cli up --to 20240320000000
```

### Rollback Migrations

Rollback migrations:
```bash
# Rollback last migration
migration-cli down

# Rollback specific number of migrations
migration-cli down 3

# Rollback to specific version
migration-cli down --to 20240320000000
```

### Check Status

View migration status:
```bash
# Show all migrations
migration-cli status

# Show pending migrations
migration-cli status --pending

# Show applied migrations
migration-cli status --applied

# Detailed status with timestamps
migration-cli status --verbose
```

### Verify Migrations

Check migration integrity:
```bash
# Verify all migrations
migration-cli verify

# Verify specific migration
migration-cli verify 20240320000000

# Update checksums
migration-cli verify --fix
```

### Repair Operations

Fix migration issues:
```bash
# Basic repair
migration-cli repair

# Force specific version
migration-cli repair --force 20240320000000

# Update checksums
migration-cli repair --update-checksums

# Clean up failed migrations
migration-cli repair --clean
```

### Backup Management

Manage database backups:
```bash
# Create backup
migration-cli backup

# Create backup with custom name
migration-cli backup --name "pre_user_migration"

# List backups
migration-cli backup --list

# Restore from backup
migration-cli restore BACKUP_ID
```

## Advanced Usage

### Transaction Control
```bash
# Run migration in transaction
migration-cli up --transaction

# Run without transaction
migration-cli up --no-transaction
```

### Dry Run
```bash
# Show what would be executed
migration-cli up --dry-run

# Verify migration without applying
migration-cli verify --dry-run
```

### Environment-specific Execution
```bash
# Run in specific environment
migration-cli up --env production

# Use specific config
migration-cli up --config prod-migrations.toml
```

### Logging and Output
```bash
# Increase verbosity
migration-cli up --verbose

# JSON output
migration-cli status --format json

# Custom log file
migration-cli up --log migrations.log
```

## Error Codes

The CLI uses the following exit codes:

| Code | Description |
|------|-------------|
| 0    | Success |
| 1    | General error |
| 2    | Configuration error |
| 3    | Database error |
| 4    | Migration error |
| 5    | Validation error |

## Examples

### 1. Complete Migration Workflow
```bash
# Create migration
migration-cli create "add_user_profiles"

# Edit migration files
vim migrations/YYYYMMDDHHMMSS_add_user_profiles.up.sql
vim migrations/YYYYMMDDHHMMSS_add_user_profiles.down.sql

# Verify migration
migration-cli verify

# Create backup
migration-cli backup

# Apply migration
migration-cli up

# Verify status
migration-cli status
```

### 2. Emergency Rollback
```bash
# Check current status
migration-cli status

# Create backup
migration-cli backup --name "pre_rollback"

# Rollback last migration
migration-cli down --confirm

# Verify rollback
migration-cli status
```

### 3. Production Deployment
```bash
# Check pending migrations
migration-cli status --env production

# Verify migrations
migration-cli verify --env production

# Create backup
migration-cli backup --env production

# Apply migrations
migration-cli up --env production --confirm
```

## Best Practices

### 1. Version Control
- Commit migration files
- Include checksums
- Document changes

### 2. Testing
- Test migrations locally
- Use representative data
- Verify rollbacks

### 3. Production Deployment
- Create backups
- Use maintenance windows
- Monitor performance

### 4. Security
- Use secure connections
- Manage credentials safely
- Audit migration execution

## Troubleshooting

### Common Issues

1. **Connection Failures**
   ```bash
   # Test connection
   migration-cli test-connection
   
   # Show connection details
   migration-cli status --show-connection
   ```

2. **Lock Timeouts**
   ```bash
   # Increase lock timeout
   migration-cli up --lock-timeout 600
   
   # Force unlock
   migration-cli repair --force-unlock
   ```

3. **Checksum Mismatches**
   ```bash
   # Show checksum details
   migration-cli verify --show-checksums
   
   # Update checksums
   migration-cli repair --update-checksums
   ```

### Debug Mode
```bash
# Enable debug logging
migration-cli up --debug

# Show SQL commands
migration-cli up --show-sql

# Full trace
migration-cli up --trace
``` 