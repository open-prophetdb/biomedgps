DROP TABLE IF EXISTS biomedgps_embedding_metadata;

ALTER TABLE biomedgps_relation_embedding DROP COLUMN formatted_relation_type;

ALTER TABLE biomedgps_relation_embedding DROP CONSTRAINT biomedgps_relation_embedding_uniq_key;

ALTER TABLE biomedgps_relation_embedding ADD CONSTRAINT biomedgps_relation_embedding_uniq_key UNIQUE (
    relation_type,
);

ALTER TABLE biomedgps_relation DROP COLUMN formatted_relation_type;

ALTER TABLE biomedgps_relation DROP CONSTRAINT biomedgps_relation_uniq_key;

ALTER TABLE biomedgps_relation ADD CONSTRAINT biomedgps_relation_uniq_key UNIQUE (
    dataset,
    resource,
    relation_type,
    source_id,
    source_type,
    target_id,
    target_type
);

ALTER TABLE biomedgps_relation_metadata DROP COLUMN formatted_relation_type;

ALTER TABLE biomedgps_relation_metadata DROP COLUMN dataset;

ALTER TABLE biomedgps_relation_metadata DROP CONSTRAINT biomedgps_relation_metadata_uniq_key;

ALTER TABLE biomedgps_relation_metadata ADD CONSTRAINT biomedgps_relation_metadata_uniq_key UNIQUE (
    resource,
    relation_type,
    start_entity_type,
    end_entity_type
);
