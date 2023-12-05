-- Add a description column into the biomedgps_relation_metadata table for describing the relation type with a human-readable sentence.

ALTER TABLE biomedgps_relation_metadata ADD COLUMN description TEXT;