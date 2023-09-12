DROP INDEX IF EXISTS idx_trgm_name_entity_table;
DROP INDEX IF EXISTS idx_trgm_xrefs_entity_table;
DROP INDEX IF EXISTS idx_trgm_synonyms_entity_table;

DROP EXTENSION pg_trgm;

-- biomedgps_knowledge_curation
-- Remove the payload field
ALTER TABLE biomedgps_knowledge_curation DROP COLUMN payload;