-- This file is used to add the tags table.

CREATE TABLE
    IF NOT EXISTS biomedgps_tags (
        id BIGSERIAL NOT NULL,
        tag_name VARCHAR(255) NOT NULL,
        tag_description TEXT,
        created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
        updated_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
        CONSTRAINT biomedgps_tags_uniq_key UNIQUE (tag_name)
    );
