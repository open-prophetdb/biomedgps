//! The embedding model is used to store the information of embeddings which are related to user's personalized knowledge graph. We used two strategies to generate embeddings:
//! 1. Generate embedding for each text when the text is created.
//! 2. [Optional] Run a background job to generate embedding for all texts in the system periodically. At least, we need to run it to check whether the embeddings exist for several fields in specified tables.

use anyhow::Ok as AnyOk;
use chrono::{serde::ts_seconds, DateTime, Utc};
use log::warn;
use poem_openapi::Object;
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;
use validator::Validate;

const DEFAULT_LENGTH_1: usize = 1;
const DEFAULT_LENGTH_16: usize = 16;
const DEFAULT_LENGTH_64: usize = 64;
const DEFAULT_LENGTH_255: usize = 255;

pub const EMBEDDING_DEFAULT_MODEL_NAME: &str = "intfloat/e5-small-v2";

const SOURCE_TYPE_TABLE_MAPPING: &[(&str, &str)] = &[
    ("entity", "biomedgps_entity_curation"),
    ("key_sentence", "biomedgps_key_sentence_curation"),
    ("knowledge", "biomedgps_knowledge_curation"),
];

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Object, sqlx::FromRow, Validate)]
pub struct Embedding {
    // Ignore this field when deserialize from json
    #[serde(skip_deserializing)]
    #[oai(read_only)]
    #[oai(skip)]
    pub id: i64,

    text: String,

    embedding: Option<Vec<f32>>,

    #[serde(skip_deserializing)]
    #[serde(with = "ts_seconds")]
    #[oai(read_only)]
    created_time: DateTime<Utc>,

    #[validate(length(
        max = "DEFAULT_LENGTH_255",
        min = "DEFAULT_LENGTH_1",
        message = "The length of model_name should be between 1 and 255."
    ))]
    model_name: String, // The name of the embedding model, such as "intfloat/e5-small-v2", "intfloat/e5-large-v2", etc.

    #[validate(length(
        max = "DEFAULT_LENGTH_64",
        min = "DEFAULT_LENGTH_1",
        message = "The length of text_source should be between 1 and 64."
    ))]
    text_source_type: String, // The type of the text source, such as "entity", "key_sentence", "knowledge", etc.

    #[validate(length(
        max = "DEFAULT_LENGTH_64",
        min = "DEFAULT_LENGTH_1",
        message = "The length of text_source should be between 1 and 64."
    ))]
    text_source_field: String, // The field of the text source, such as "key_sentence", "description", etc.

    #[validate(length(
        max = "DEFAULT_LENGTH_64",
        min = "DEFAULT_LENGTH_1",
        message = "The length of text_source should be between 1 and 64."
    ))]
    text_source_id: String, // The id of the text source, such as the id of the key sentence, the id of the abstract, the id of the note, etc.

    // The payload is used to store the context information of the embedding, such as the metadata of the related paper, etc.
    #[oai(skip_serializing_if_is_none)]
    payload: Option<JsonValue>,

    owner: String,
    groups: Vec<String>,

    #[oai(skip_serializing_if_is_none)]
    distance: Option<f32>,
}

impl Embedding {
    pub async fn check_table_exists(pool: &sqlx::PgPool) -> Result<bool, anyhow::Error> {
        let query = "SELECT EXISTS (
            SELECT 1
            FROM information_schema.tables
            WHERE table_name = 'biomedgps_text_embedding'
        )";
        let result = sqlx::query_scalar::<_, bool>(query).fetch_one(pool).await?;

        AnyOk(result)
    }

    /// Create a table named `biomedgps_text_embedding` if not exists. It is used to store all embeddings of texts from different tables.
    /// !!!NOTE: You need to run this function at the launch of the system.!!!
    pub async fn upcreate_table(pool: &sqlx::PgPool) -> Result<(), anyhow::Error> {
        if !Self::check_table_exists(pool).await? {
            let query = format!("
                    CREATE TABLE IF NOT EXISTS biomedgps_text_embedding (
                        id BIGSERIAL PRIMARY KEY,
                        text TEXT NOT NULL,
                        embedding FLOAT4[] GENERATED ALWAYS AS (pgml.embed('{model_name}', text)) STORED,
                        created_time TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
                        model_name VARCHAR(255) NOT NULL,
                        text_source_type VARCHAR(64) NOT NULL,
                        text_source_field VARCHAR(64) NOT NULL,
                        text_source_id VARCHAR(64) NOT NULL,
                        payload JSONB,
                        owner VARCHAR(32) NOT NULL,
                        groups VARCHAR(32)[] NOT NULL
                    )
                ",
                model_name = EMBEDDING_DEFAULT_MODEL_NAME
            );

            sqlx::query(&query).execute(pool).await?;
        } else {
            warn!("Table biomedgps_text_embedding already exists, if you want to use a different model to generate embedding, please delete the existing table and recreate it. The embeddings will not be generated automatically for existing data. So we need to remember to generate all embeddings again.");
        }

        AnyOk(())
    }

    pub async fn get_records(
        pool: &sqlx::PgPool,
        question: &str,
        text_source_type: &str,
        text_source_field: Option<Vec<String>>,
        owner: &str,
        top_k: usize,
    ) -> Result<Vec<Embedding>, anyhow::Error> {
        let where_str = match text_source_field {
            Some(fields) => format!(" AND text_source_field IN ('{}')", fields.join("', '")),
            None => "".to_string(),
        };

        let query = format!(
            "
            SELECT *, pgml.distance_l2(embedding, pgml.embed('{model_name}', $1)) AS distance
            FROM biomedgps_text_embedding
            WHERE text_source_type = $2 AND owner = $3 {where_str}
            ORDER BY distance ASC
            LIMIT {top_k}
        ",
            model_name = EMBEDDING_DEFAULT_MODEL_NAME,
            where_str = where_str,
            top_k = top_k
        );

        let records = sqlx::query_as::<_, Embedding>(&query)
            .bind(question)
            .bind(text_source_type)
            .bind(owner)
            .fetch_all(pool)
            .await?;

        AnyOk(records)
    }

    pub async fn insert_record(
        tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
        text: &str,
        text_source_type: &str,
        text_source_field: &str,
        text_source_id: &str,
        owner: &str,
        groups: Option<Vec<String>>,
        payload: Option<JsonValue>,
    ) -> Result<(), anyhow::Error> {
        let query = "INSERT INTO biomedgps_text_embedding (text, model_name, text_source_type, text_source_field, text_source_id, payload, owner, groups) VALUES ($1, $2, $3, $4, $5, $6, $7, $8)";

        sqlx::query(&query)
            .bind(text)
            .bind(EMBEDDING_DEFAULT_MODEL_NAME)
            .bind(text_source_type)
            .bind(text_source_field)
            .bind(text_source_id)
            .bind(payload)
            .bind(owner)
            .bind(groups)
            .execute(tx)
            .await?;

        AnyOk(())
    }
}
