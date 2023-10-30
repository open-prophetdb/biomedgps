-- Revert: 20231030_alter_emb_size.up.sql

ALTER TABLE biomedgps_entity_embedding ALTER COLUMN embedding TYPE vector(400);
ALTER TABLE biomedgps_relation_embedding ALTER COLUMN embedding TYPE vector(400);