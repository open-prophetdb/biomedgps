-- Alter the embedding column type from vector(400) to vector(768)

ALTER TABLE biomedgps_entity_embedding ALTER COLUMN embedding TYPE vector(768);
ALTER TABLE biomedgps_relation_embedding ALTER COLUMN embedding TYPE vector(768);
