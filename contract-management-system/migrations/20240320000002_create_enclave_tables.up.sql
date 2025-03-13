-- Create enclaves table
CREATE TABLE enclaves (
    id UUID PRIMARY KEY,
    name VARCHAR(255) NOT NULL,
    provider_id UUID NOT NULL REFERENCES identities(id),
    status VARCHAR(50) NOT NULL,
    attestation JSONB,
    configuration JSONB NOT NULL,
    metrics JSONB,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP
);

-- Create indexes
CREATE INDEX idx_enclaves_provider_id ON enclaves(provider_id);
CREATE INDEX idx_enclaves_status ON enclaves(status);
CREATE INDEX idx_enclaves_created_at ON enclaves(created_at);

-- Create trigger for updated_at
CREATE OR REPLACE FUNCTION update_enclaves_updated_at()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = CURRENT_TIMESTAMP;
    RETURN NEW;
END;
$$ language 'plpgsql';

CREATE TRIGGER update_enclaves_updated_at
    BEFORE UPDATE ON enclaves
    FOR EACH ROW
    EXECUTE FUNCTION update_enclaves_updated_at(); 