-- Add a prompt template column into the biomedgps_relation_metadata table for describing the prompt template of the relation type.
ALTER TABLE biomedgps_relation_metadata
ADD COLUMN prompt_template TEXT DEFAULT NULL;


-- biomedgps_configuration table is created to store the configuration of the biomedgps.
CREATE TABLE IF NOT EXISTS biomedgps_configuration (
    id BIGSERIAL PRIMARY KEY, -- The configuration ID
    config_name VARCHAR(64) NOT NULL, -- The name of the configuration
    config_title VARCHAR(64) NOT NULL, -- The title of the configuration
    config_description TEXT NOT NULL, -- The description of the configuration
    category VARCHAR(64) NOT NULL, -- The category of the configuration, such as customized_metadata field.
    owner VARCHAR(64), -- The owner of the configuration. If not specified, it is a public configuration for all users.

    CONSTRAINT biomedgps_configuration_uniq_key UNIQUE (config_name, category, owner)
);


-- biomedgps_entity_metadata_curation table is created to store metadata for the curation of the entity metadata.
CREATE TABLE IF NOT EXISTS biomedgps_entity_metadata_curation (
    id BIGSERIAL PRIMARY KEY, -- The entity metadata curation ID
    entity_id VARCHAR(64) NOT NULL, -- The entity ID, format: <DATABASE_NAME>:<DATABASE_ID>, such as ENTREZ:1234, MESH:D000003
    entity_type VARCHAR(64) NOT NULL, -- The entity type, such as Anatomy, Disease, Gene, Compound, Biological Process, etc.
    entity_name VARCHAR(255) NOT NULL, -- The name of the entity
    field_name VARCHAR(64) NOT NULL, -- The field name of the entity metadata
    field_value TEXT NOT NULL, -- The field value of the entity metadata
    field_title TEXT NOT NULL, -- The field title of the entity metadata
    key_sentence TEXT NOT NULL, -- The key sentence of the entity metadata
    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP, -- The created time of the entity metadata

    curator VARCHAR(64) NOT NULL, -- The curator of the entity metadata
    fingerprint VARCHAR(1024) NOT NULL, -- The fingerprint of the entity metadata, such as pmid:5678, doi:1234, http://www.example.com, etc. The priority is pmid > doi > http.
    payload JSONB DEFAULT '{"project_id": "0", "organization_id": "0"}', -- The payload of the entity metadata, such as the organization id, the project id and the task id, etc.
    annotation JSONB DEFAULT NULL, -- The annotation of the entity metadata to record the labeling data, such as the xpath, offset, etc.

    -- Why entity_name? because sometimes we don't have the formatted id and type for the entity, we will use the Unknown, then we may make the entity_id unique.
    -- Why field_value? We allow to have multiple fields with the same field_name, but with different field_value.
    CONSTRAINT biomedgps_entity_metadata_curation_uniq_key UNIQUE (entity_id, entity_type, entity_name, field_name, field_value, curator, fingerprint)
);

-- biomedgps_knowledge_curation
-- We need to add a annotation field to store the labeling data, such as the xpath, offset, etc.
ALTER TABLE biomedgps_knowledge_curation 
ADD COLUMN annotation JSONB DEFAULT NULL;

BEGIN;

ALTER TABLE biomedgps_knowledge_curation
RENAME COLUMN pmid TO fingerprint;

ALTER TABLE biomedgps_knowledge_curation
ALTER COLUMN fingerprint TYPE VARCHAR(1024);  -- The fingerprint of the knowledge, such as pmid:5678, doi:1234, http://www.example.com, etc. The priority is pmid > doi > http.

COMMIT;

-- biomedgps_entity_curation
CREATE TABLE IF NOT EXISTS biomedgps_entity_curation (
    id BIGSERIAL PRIMARY KEY, -- The entity curation ID
    entity_id VARCHAR(64) NOT NULL, -- The entity ID, format: <DATABASE_NAME>:<DATABASE_ID>, such as ENTREZ:1234, MESH:D000003
    entity_type VARCHAR(64) NOT NULL, -- The entity type, such as Anatomy, Disease, Gene, Compound, Biological Process, etc.
    entity_name VARCHAR(255) NOT NULL, -- The name of the entity
    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP, -- The created time of the entity

    curator VARCHAR(64) NOT NULL, -- The curator of the entity
    fingerprint VARCHAR(1024) NOT NULL, -- The fingerprint of the entity, such as pmid:5678, doi:1234, http://www.example.com, etc. The priority is pmid > doi > http.
    payload JSONB DEFAULT '{"project_id": "0", "organization_id": "0"}', -- The payload of the entity, such as the organization id, the project id and the task id, etc.
    annotation JSONB DEFAULT NULL, -- The annotation of the entity, such as the xpath, offset, etc.

    CONSTRAINT biomedgps_entity_curation_uniq_key UNIQUE (entity_id, entity_type, entity_name, curator, fingerprint)
);