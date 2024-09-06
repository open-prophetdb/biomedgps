-- biomedgps_key_sentence_curation is used to store the key sentence curation data.
CREATE TABLE IF NOT EXISTS biomedgps_key_sentence_curation (
    id BIGSERIAL PRIMARY KEY, -- The key sentence curation ID
    fingerprint VARCHAR(1024) NOT NULL, -- The fingerprint of the knowledge, such as pmid:5678, doi:1234, http://www.example.com, etc. The priority is pmid > doi > http.
    curator VARCHAR(64) NOT NULL, -- The curator of the key sentence
    key_sentence TEXT NOT NULL, -- The key sentence
    description TEXT NOT NULL, -- The user's note for the key sentence
    payload JSONB DEFAULT '{"project_id": "0", "organization_id": "0"}', -- The payload of the key sentence, such as the organization id, the project id and the task id, etc.
    annotation JSONB DEFAULT NULL, -- The annotation of the key sentence, such as the xpath, offset, etc.
    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP, -- The created time of the key sentence

    CONSTRAINT biomedgps_key_sentence_curation_uniq_key UNIQUE (fingerprint, curator, key_sentence)
);


-- biomedgps_webpage_metadata is used to store the webpage metadata.
CREATE TABLE IF NOT EXISTS biomedgps_webpage_metadata (
    id BIGSERIAL PRIMARY KEY, -- The webpage metadata ID
    fingerprint VARCHAR(1024) NOT NULL, -- The fingerprint of the knowledge, such as pmid:5678, doi:1234, http://www.example.com, etc. The priority is pmid > doi > http.
    curator VARCHAR(64) NOT NULL, -- The curator of the webpage metadata
    note TEXT NOT NULL, -- The user's note for the webpage
    metadata JSONB NOT NULL, -- The metadata of the website
    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP, -- The created time of the website metadata

    CONSTRAINT biomedgps_webpage_metadata_uniq_key UNIQUE (fingerprint, curator)
);