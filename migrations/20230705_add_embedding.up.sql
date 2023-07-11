-- We use the pgvector extension to store the embedding of the entities and relations, more details on https://github.com/pgvector/pgvector

-- We need to use the postgresml docker image to instead of the postgres docker image, because the postgresml docker image has installed the pgvector and pgml extensions. More details on https://github.com/postgresml/postgresml#installation
-- Install the pgvector extension
CREATE EXTENSION vector;

-- Install the postgresml extension
CREATE EXTENSION pgml;

-- biomedgps_entity_embedding table is used to store the embedding of the entities for computing the similarity of the entities
CREATE TABLE
  IF NOT EXISTS biomedgps_entity_embedding (
    embedding_id BIGINT PRIMARY KEY, -- The embedding ID
    entity_id VARCHAR(64) NOT NULL, -- The entity ID
    entity_type VARCHAR(64) NOT NULL, -- The entity type, such as Anatomy, Disease, Gene, Compound, Biological Process, etc.
    entity_name VARCHAR(255) NOT NULL, -- The entity name
    embedding vector(400), -- The embedding array, the length of the embedding array is 400. It is related with the knowledge graph model, such as TransE, DistMult, etc.
    UNIQUE (entity_id, entity_type)
  );

-- biomedgps_relation_embedding table is used to store the embedding of the relations for predicting the relations
CREATE TABLE
  IF NOT EXISTS biomedgps_relation_embedding (
    embedding_id BIGINT PRIMARY KEY, -- The embedding ID
    relation_type VARCHAR(64) NOT NULL, -- The relation type, such as ACTIVATOR::Gene:Compound, INHIBITOR::Gene:Compound, etc.
    embedding vector(400), -- The embedding array
    UNIQUE (
      relation_type
    )
  );