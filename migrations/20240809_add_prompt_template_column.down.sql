-- Revert: 20240809_add_prompt_template_column.up.sql

-- biomedgps_entity_curation table
DROP TABLE IF EXISTS biomedgps_entity_curation;

-- biomedgps_knowledge_curation table modifications
BEGIN;

ALTER TABLE biomedgps_knowledge_curation
ALTER COLUMN fingerprint TYPE INTEGER USING fingerprint::BIGINT;

ALTER TABLE biomedgps_knowledge_curation
RENAME COLUMN fingerprint TO pmid;

ALTER TABLE biomedgps_knowledge_curation 
DROP COLUMN annotation;

COMMIT;

-- biomedgps_entity_metadata_curation table
DROP TABLE IF EXISTS biomedgps_entity_metadata_curation;

-- biomedgps_configuration table
DROP TABLE IF EXISTS biomedgps_configuration;

-- biomedgps_relation_metadata table modifications
ALTER TABLE biomedgps_relation_metadata
DROP COLUMN prompt_template;
