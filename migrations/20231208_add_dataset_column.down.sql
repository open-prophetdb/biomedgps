-- Revert: 20231208_add_dataset_column.up.sql

ALTER TABLE biomedgps_relation DROP COLUMN dataset;

ALTER TABLE biomedgps_relation DROP CONSTRAINT biomedgps_relation_uniq_key;

ALTER TABLE biomedgps_relation ADD CONSTRAINT biomedgps_relation_uniq_key UNIQUE (
    relation_type,
    source_id,
    source_type,
    target_id,
    target_type
);