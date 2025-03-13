-- Drop trigger
DROP TRIGGER IF EXISTS update_enclaves_updated_at ON enclaves;

-- Drop function
DROP FUNCTION IF EXISTS update_enclaves_updated_at();

-- Drop indexes
DROP INDEX IF EXISTS idx_enclaves_provider_id;
DROP INDEX IF EXISTS idx_enclaves_status;
DROP INDEX IF EXISTS idx_enclaves_created_at;

-- Drop table
DROP TABLE IF EXISTS enclaves; 