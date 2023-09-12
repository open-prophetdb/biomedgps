-- biomedgps_knowledge_curation
-- We need to add a payload field to store the context information of the relation, such as the organization id, the project id and the task id, etc.
ALTER TABLE biomedgps_knowledge_curation 
ADD COLUMN payload JSONB DEFAULT '{"project_id": "0", "organization_id": "0"}';

-- Enable intelligent searching for the entity table
CREATE EXTENSION pg_trgm;

CREATE INDEX IF NOT EXISTS idx_trgm_name_entity_table ON biomedgps_entity USING gin(name gin_trgm_ops);
CREATE INDEX IF NOT EXISTS idx_trgm_xrefs_entity_table ON biomedgps_entity USING gin(xrefs gin_trgm_ops);
CREATE INDEX IF NOT EXISTS idx_trgm_synonyms_entity_table ON biomedgps_entity USING gin(synonyms gin_trgm_ops);