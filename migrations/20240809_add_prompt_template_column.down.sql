-- Revert: 20240809_add_prompt_template_column.up.sql

ALTER TABLE biomedgps_relation_metadata DROP COLUMN prompt_template;