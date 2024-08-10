-- Add a prompt template column into the biomedgps_relation_metadata table for describing the prompt template of the relation type.
ALTER TABLE biomedgps_relation_metadata
ADD COLUMN prompt_template TEXT DEFAULT NULL;
