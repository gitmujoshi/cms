-- Drop indexes
DROP INDEX IF EXISTS idx_audit_timestamp;
DROP INDEX IF EXISTS idx_audit_resource;
DROP INDEX IF EXISTS idx_audit_identity;
DROP INDEX IF EXISTS idx_audit_type;
DROP INDEX IF EXISTS idx_policies_priority;
DROP INDEX IF EXISTS idx_policies_name;
DROP INDEX IF EXISTS idx_role_assignments_identity;
DROP INDEX IF EXISTS idx_roles_name;
DROP INDEX IF EXISTS idx_credentials_expiry;
DROP INDEX IF EXISTS idx_credentials_type;
DROP INDEX IF EXISTS idx_identities_status;
DROP INDEX IF EXISTS idx_identities_type;

-- Drop tables in reverse order of creation to handle dependencies
DROP TABLE IF EXISTS audit_events;
DROP TABLE IF EXISTS policies;
DROP TABLE IF EXISTS role_assignments;
DROP TABLE IF EXISTS roles;
DROP TABLE IF EXISTS credentials;
DROP TABLE IF EXISTS identities; 