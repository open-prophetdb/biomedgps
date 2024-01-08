-- biomedgps_embedding_metadata table is used to store the metadata of the embeddings, such as the name of the embedding table, the name of the embedding model, the name of the embedding model type, etc.
CREATE TABLE
  IF NOT EXISTS biomedgps_embedding_metadata (
    id BIGSERIAL PRIMARY KEY, -- The entity metadata ID
    table_name VARCHAR(64) NOT NULL UNIQUE, -- The name of the embedding table. It is a prefix of the real embedding table name, such as if you use biomedgps as the table name, the real embedding table name is biomedgps_entity_embedding, biomedgps_relation_embedding, etc.
    model_name VARCHAR(64) NOT NULL UNIQUE, -- The name of the embedding model
    model_type VARCHAR(64) NOT NULL, -- The type of the embedding model, such as TransE, DistMult, etc.
    description TEXT NOT NULL, -- The description of the embedding model
    datasets ARRAY (TEXT) NOT NULL, -- The datasets which are used to train the embedding model. The dataset is the same as the 
    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP, -- The created time of the embedding metadata
    dimension INTEGER NOT NULL, -- The dimension of the embedding
    metadata TEXT NOT NULL, -- The metadata of the embedding model
    CONSTRAINT biomedgps_embedding_metadata_uniq_key UNIQUE (table_name, model_name)
  );

-- Add a formatted_relation_type column into the biomedgps_relation_embedding table for formatting the relation type.
ALTER TABLE biomedgps_relation_embedding
ADD COLUMN formatted_relation_type VARCHAR(64) NOT NULL;

ALTER TABLE biomedgps_relation_embedding
DROP CONSTRAINT biomedgps_relation_embedding_uniq_key;

ALTER TABLE biomedgps_relation_embedding ADD CONSTRAINT biomedgps_relation_embedding_uniq_key UNIQUE (
  relation_type,
  formatted_relation_type,
);

-- Add a formatted_relation_type column into the biomedgps_relation table for formatting the relation type.
ALTER TABLE biomedgps_relation
ADD COLUMN formatted_relation_type VARCHAR(64) NOT NULL;

-- The formatted relation type, such as BIOMEDGPS::ACTIVATOR::Gene:Compound, BIOMEDGPS::INHIBITOR::Gene:Compound, etc.
-- Alter UNIQUE constraint of the biomedgps_relation table to include the dataset and resource columns.
ALTER TABLE biomedgps_relation
DROP CONSTRAINT biomedgps_relation_uniq_key;

ALTER TABLE biomedgps_relation ADD CONSTRAINT biomedgps_relation_uniq_key UNIQUE (
  dataset,
  resource,
  relation_type,
  formatted_relation_type,
  source_id,
  source_type,
  target_id,
  target_type
);

-- Add a formatted_relation_type column into the biomedgps_relation_metadata table for formatting the relation type.
ALTER TABLE biomedgps_relation_metadata
ADD COLUMN formatted_relation_type VARCHAR(64) NOT NULL;

-- Add a dataset column into the biomedgps_relation_metadata table for describing the dataset the relation metadata belongs to.
ALTER TABLE biomedgps_relation_metadata
ADD COLUMN dataset VARCHAR(64) NOT NULL;

-- The formatted relation type, such as BIOMEDGPS::ACTIVATOR::Gene:Compound, BIOMEDGPS::INHIBITOR::Gene:Compound, etc.
-- Alter UNIQUE constraint of the biomedgps_relation table to include the dataset and resource columns.
ALTER TABLE biomedgps_relation_metadata
DROP CONSTRAINT biomedgps_relation_metadata_uniq_key;

ALTER TABLE biomedgps_relation_metadata ADD CONSTRAINT biomedgps_relation_metadata_uniq_key UNIQUE (
  dataset,
  resource,
  relation_type,
  formatted_relation_type,
  start_entity_type,
  end_entity_type
);
