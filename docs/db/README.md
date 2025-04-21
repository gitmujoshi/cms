# Database Documentation

## Overview

The Contract Management System uses PostgreSQL as its primary database, with additional support for Redis caching and blockchain storage for immutable records.

## Table of Contents

1. [Database Schema](schema.md)
   - Tables and Relationships
   - Indexes and Constraints
   - Data Types and Validation

2. [Migrations](migrations/README.md)
   - Migration History
   - Migration Process
   - Rollback Procedures

3. [Backup and Recovery](backup.md)
   - Backup Strategies
   - Recovery Procedures
   - Disaster Recovery

4. [Performance Optimization](performance.md)
   - Query Optimization
   - Indexing Strategy
   - Connection Pooling

5. [Security](security.md)
   - Access Control
   - Encryption
   - Audit Logging

## Database Architecture

### PostgreSQL
- Primary database for all contract and user data
- Supports ACID transactions
- Implements row-level security
- Uses table partitioning for large datasets

### Redis
- Caching layer for frequently accessed data
- Session management
- Rate limiting
- Real-time notifications

### Blockchain
- Immutable contract storage
- Digital signature verification
- Audit trail maintenance

## Common Operations

### Backup
```bash
# Daily backup
pg_dump -U postgres -d contract_management > backup.sql

# Point-in-time recovery
pg_basebackup -D /backup/location -Ft -z -P
```

### Migration
```bash
# Run migrations
cargo run --bin migrate up

# Rollback last migration
cargo run --bin migrate down
```

### Monitoring
```sql
-- Check active connections
SELECT count(*) FROM pg_stat_activity;

-- Monitor slow queries
SELECT * FROM pg_stat_activity 
WHERE state = 'active' 
AND now() - query_start > interval '5 minutes';
```

## Best Practices

1. **Schema Design**
   - Use appropriate data types
   - Implement proper constraints
   - Create necessary indexes
   - Document all tables and columns

2. **Performance**
   - Regular vacuum and analyze
   - Monitor query performance
   - Use connection pooling
   - Implement proper indexing

3. **Security**
   - Use row-level security
   - Implement proper access control
   - Encrypt sensitive data
   - Regular security audits

4. **Maintenance**
   - Regular backups
   - Monitor disk space
   - Update statistics
   - Check for bloat

## Troubleshooting

### Common Issues

1. **Connection Issues**
   - Check connection pool settings
   - Verify network connectivity
   - Review firewall rules

2. **Performance Issues**
   - Analyze slow queries
   - Check index usage
   - Monitor resource usage

3. **Data Corruption**
   - Verify backup integrity
   - Check for disk errors
   - Monitor system logs

### Recovery Procedures

1. **Point-in-Time Recovery**
   ```bash
   # Stop the database
   pg_ctl stop -D $PGDATA

   # Restore from backup
   pg_restore -d contract_management backup.dump

   # Start the database
   pg_ctl start -D $PGDATA
   ```

2. **Data Recovery**
   ```sql
   -- Check for corrupted data
   SELECT * FROM pg_catalog.pg_class 
   WHERE relkind = 'r' 
   AND pg_relation_size(oid) > 0;

   -- Repair corrupted tables
   REINDEX TABLE table_name;
   ```

## Support

For database-related issues:
1. Check the logs in `/var/log/postgresql`
2. Review monitoring dashboards
3. Contact the database team
4. Submit a support ticket 