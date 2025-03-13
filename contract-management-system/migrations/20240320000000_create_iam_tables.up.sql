-- Create identities table
CREATE TABLE identities (
    id UUID PRIMARY KEY,
    name VARCHAR(255) NOT NULL,
    identity_type VARCHAR(50) NOT NULL,
    status VARCHAR(50) NOT NULL,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL,
    last_modified TIMESTAMP WITH TIME ZONE NOT NULL,
    metadata JSONB NOT NULL DEFAULT '{}'
);

-- Create credentials table
CREATE TABLE credentials (
    identity_id UUID PRIMARY KEY REFERENCES identities(id) ON DELETE CASCADE,
    credential_type VARCHAR(50) NOT NULL,
    encrypted_data BYTEA NOT NULL,
    expires_at TIMESTAMP WITH TIME ZONE,
    last_rotated TIMESTAMP WITH TIME ZONE NOT NULL,
    max_age_days INTEGER NOT NULL,
    require_rotation BOOLEAN NOT NULL,
    notify_before_days INTEGER NOT NULL
);

-- Create roles table
CREATE TABLE roles (
    id UUID PRIMARY KEY,
    name VARCHAR(255) NOT NULL UNIQUE,
    description TEXT,
    permissions JSONB NOT NULL,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL,
    last_modified TIMESTAMP WITH TIME ZONE NOT NULL
);

-- Create role assignments table
CREATE TABLE role_assignments (
    role_id UUID REFERENCES roles(id) ON DELETE CASCADE,
    identity_id UUID REFERENCES identities(id) ON DELETE CASCADE,
    assigned_at TIMESTAMP WITH TIME ZONE NOT NULL,
    PRIMARY KEY (role_id, identity_id)
);

-- Create policies table
CREATE TABLE policies (
    id UUID PRIMARY KEY,
    name VARCHAR(255) NOT NULL UNIQUE,
    description TEXT,
    effect VARCHAR(50) NOT NULL,
    resources JSONB NOT NULL,
    actions JSONB NOT NULL,
    conditions JSONB NOT NULL,
    priority INTEGER NOT NULL,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL,
    last_modified TIMESTAMP WITH TIME ZONE NOT NULL
);

-- Create audit events table
CREATE TABLE audit_events (
    id UUID PRIMARY KEY,
    event_type VARCHAR(50) NOT NULL,
    identity_id UUID REFERENCES identities(id) ON DELETE SET NULL,
    resource_type VARCHAR(255) NOT NULL,
    resource_id VARCHAR(255),
    action VARCHAR(255) NOT NULL,
    status VARCHAR(50) NOT NULL,
    error_message TEXT,
    request_metadata JSONB NOT NULL,
    timestamp TIMESTAMP WITH TIME ZONE NOT NULL
);

-- Create indexes
CREATE INDEX idx_identities_type ON identities(identity_type);
CREATE INDEX idx_identities_status ON identities(status);
CREATE INDEX idx_credentials_type ON credentials(credential_type);
CREATE INDEX idx_credentials_expiry ON credentials(expires_at);
CREATE INDEX idx_roles_name ON roles(name);
CREATE INDEX idx_role_assignments_identity ON role_assignments(identity_id);
CREATE INDEX idx_policies_name ON policies(name);
CREATE INDEX idx_policies_priority ON policies(priority);
CREATE INDEX idx_audit_type ON audit_events(event_type);
CREATE INDEX idx_audit_identity ON audit_events(identity_id);
CREATE INDEX idx_audit_resource ON audit_events(resource_type, resource_id);
CREATE INDEX idx_audit_timestamp ON audit_events(timestamp);

-- Create default admin role
INSERT INTO roles (
    id,
    name,
    description,
    permissions,
    created_at,
    last_modified
) VALUES (
    '00000000-0000-0000-0000-000000000000',
    'admin',
    'System administrator role with full access',
    '["*"]',
    CURRENT_TIMESTAMP,
    CURRENT_TIMESTAMP
); 