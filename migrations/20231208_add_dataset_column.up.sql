-- Add a dataset column into the biomedgps_relation table for describing the dataset the relation belongs to.
ALTER TABLE biomedgps_relation
ADD COLUMN dataset VARCHAR(64) NOT NULL;

-- Alter UNIQUE constraint of the biomedgps_relation table to include the dataset and resource columns.
ALTER TABLE biomedgps_relation
DROP CONSTRAINT biomedgps_relation_uniq_key;

ALTER TABLE biomedgps_relation ADD CONSTRAINT biomedgps_relation_uniq_key UNIQUE (
    dataset,
    resource,
    relation_type,
    source_id,
    source_type,
    target_id,
    target_type
)