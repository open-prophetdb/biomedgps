-- A relational database is used to store the entities, relations, and the metadata of the entities and relations. If we need to add more detailed information of the entities and relations, we can add more tables in the relational database. such as adding more fields for the Gene, Compound, Biological Process entity, etc.
-- biomedgps_entity table is used to store the entities, it's same with the nodes in the biomedgps neo4j database
CREATE TABLE
  IF NOT EXISTS biomedgps_entity (
    idx BIGSERIAL PRIMARY KEY, -- The entity index
    id VARCHAR(64) NOT NULL, -- The entity ID
    name VARCHAR(255) NOT NULL, -- The name of the entity
    label VARCHAR(64) NOT NULL, -- The label of the entity, such as Anatomy, Disease, Gene, Compound, Biological Process, etc.
    resource VARCHAR(64) NOT NULL, -- The resource of the entity, such as UBERON, DOID, HGNC, CHEBI, GO, etc.
    description TEXT, -- The description of the entity
    taxid VARCHAR(64), -- The taxonomy ID of the entity
    synonyms TEXT, -- The synonyms of the entity
    pmids TEXT, -- The PMIDs which mentions the entity
    xrefs TEXT, -- The cross references of the entity
    UNIQUE (id, label) -- The unique constraint of the entity
  );

-- biomedgps_entity_metadata table is used to store the metadata of the entities, it is used to visualize the statistics of the entities on the statistics page
CREATE TABLE
  IF NOT EXISTS biomedgps_entity_metadata (
    id BIGSERIAL PRIMARY KEY, -- The entity metadata ID
    resource VARCHAR(64) NOT NULL, -- The source of the entity metadata
    entity_type VARCHAR(64) NOT NULL, -- The entity type of the entity metadata, such as Anatomy, Disease, Gene, Compound, Biological Process, etc.
    entity_count BIGINT NOT NULL, -- The entity count of the entity metadata
    UNIQUE (resource, entity_type)
  );

-- biomedgps_relation_metadata table is used to store the metadata of the relations, it is used to visualize the statistics of the relations on the statistics page
CREATE TABLE
  IF NOT EXISTS biomedgps_relation_metadata (
    id BIGSERIAL PRIMARY KEY, -- The relation metadata ID
    resource VARCHAR(64) NOT NULL, -- The resource of the relation
    relation_type VARCHAR(64) NOT NULL, -- The relation type, such as ACTIVATOR::Gene:Compound, INHIBITOR::Gene:Compound, etc.
    relation_count BIGINT NOT NULL, -- The relation count
    start_entity_type VARCHAR(64) NOT NULL, -- The start entity type, such as Anatomy, Disease, Gene, Compound, Biological Process, etc.
    end_entity_type VARCHAR(64) NOT NULL, -- The end entity type, such as Anatomy, Disease, Gene, Compound, Biological Process, etc.
    UNIQUE (
      resource,
      relation_type,
      start_entity_type,
      end_entity_type
    )
  );

-- biomedgps_knowledge_curation table is used to store the knowledges which are curated by the curators from the literature
CREATE TABLE
  IF NOT EXISTS biomedgps_knowledge_curation (
    id BIGSERIAL PRIMARY KEY, -- The knowledge curation ID
    relation_type VARCHAR(64) NOT NULL, -- The relation type, such as ACTIVATOR::Gene:Compound, INHIBITOR::Gene:Compound, etc.
    source_name VARCHAR(255) NOT NULL, -- The name of the start entity
    source_type VARCHAR(64) NOT NULL, -- The entity type, such as Gene, Compound, Biological Process, etc.
    source_id VARCHAR(64) NOT NULL, -- The ID of the start entity
    target_name VARCHAR(255) NOT NULL, -- The name of the end entity
    target_type VARCHAR(64) NOT NULL, -- The entity type, such as Gene, Compound, Biological Process, etc.
    target_id VARCHAR(64) NOT NULL, -- The ID of the end entity, format: <DATABASE_NAME>:<DATABASE_ID>, such as ENTREZ:1234, MESH:D000003
    key_sentence TEXT NOT NULL, -- The key sentence of the relation
    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP, -- The created time of the relation
    curator VARCHAR(64) NOT NULL, -- The curator of the relation
    pmid BIGINT NOT NULL, -- The PMID of the relation
    UNIQUE (
      relation_type,
      source_name,
      source_type,
      source_id,
      target_name,
      target_type,
      target_id,
      curator,
      pmid
    )
  );

-- biomedgps_relation table is used to store the relations between the entities, it's same with the edges in the biomedgps neo4j database
CREATE TABLE
  IF NOT EXISTS biomedgps_relation (
    id BIGSERIAL PRIMARY KEY, -- The relation ID
    relation_type VARCHAR(64) NOT NULL, -- The relation type, such as ACTIVATOR::Gene:Compound, INHIBITOR::Gene:Compound, etc.
    source_id VARCHAR(64) NOT NULL, -- The ID of the start entity
    source_type VARCHAR(64) NOT NULL, -- The entity type, such as Gene, Compound, Biological Process, etc.
    target_id VARCHAR(64) NOT NULL, -- The ID of the end entity, format: <DATABASE_NAME>:<DATABASE_ID>, such as ENTREZ:1234, MESH:D000003
    target_type VARCHAR(64) NOT NULL, -- The entity type, such as Gene, Compound, Biological Process, etc.
    resource VARCHAR(64) NOT NULL, -- The resource of the relation
    key_sentence TEXT, -- The key sentence of the relation
    pmids TEXT, -- The PMIDs which mentions the relation
    score FLOAT, -- The score of the relation
    UNIQUE (
      relation_type,
      source_id,
      source_type,
      target_id,
      target_type
    )
  );

-- biomedgps_entity2d table is used to store the 2D embedding of the entities for computing the similarity of the entities
CREATE TABLE
  IF NOT EXISTS biomedgps_entity2d (
    embedding_id BIGINT PRIMARY KEY, -- The embedding ID
    entity_id VARCHAR(64) NOT NULL, -- The entity ID
    entity_type VARCHAR(64) NOT NULL, -- The entity type, such as Anatomy, Disease, Gene, Compound, Biological Process, etc.
    entity_name VARCHAR(255) NOT NULL, -- The entity name
    umap_x FLOAT NOT NULL, -- The UMAP X coordinate
    umap_y FLOAT NOT NULL, -- The UMAP Y coordinate
    tsne_x FLOAT NOT NULL, -- The t-SNE X coordinate
    tsne_y FLOAT NOT NULL, -- The t-SNE Y coordinate
    UNIQUE (entity_id, entity_type)
  );

-- biomedgps_subgraph table is used to store the subgraph which is created by the user
CREATE TABLE
  IF NOT EXISTS biomedgps_subgraph (
    id VARCHAR(36) PRIMARY KEY,
    name VARCHAR(64) NOT NULL,
    description TEXT,
    payload TEXT NOT NULL,
    created_time TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    owner VARCHAR(36) NOT NULL,
    version VARCHAR(36) NOT NULL,
    db_version VARCHAR(36) NOT NULL,
    parent VARCHAR(36) REFERENCES biomedgps_subgraph (id) ON DELETE CASCADE ON UPDATE CASCADE
  );