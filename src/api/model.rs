use crate::query::sql_builder::{ComposeQuery, QueryItem};
use anyhow::Ok as AnyOk;
use chrono::serde::ts_seconds;
use chrono::{DateTime, Utc};
use log::{debug, error, info, warn};
use poem_openapi::Object;
use serde::{Deserialize, Serialize};
use std::{error::Error, path::PathBuf};

pub fn get_column_names(filepath: &PathBuf) -> Result<Vec<String>, Box<dyn Error>> {
    let mut reader = csv::ReaderBuilder::new()
        .delimiter(b'\t')
        .from_path(filepath)?; // Use tab as delimiter

    let headers = reader.headers()?;
    let mut column_names = Vec::new();
    for header in headers {
        column_names.push(header.to_string());
    }

    Ok(column_names)
}

pub trait CheckData {
    // Implement the check function
    fn check_csv_is_valid<S: for<'de> serde::Deserialize<'de>>(filepath: &PathBuf) -> bool {
        // Build the CSV reader
        let mut reader = match csv::ReaderBuilder::new()
            .delimiter(b'\t')
            .from_path(filepath)
        {
            Ok(r) => r,
            Err(e) => {
                error!("Failed to read CSV: {}", e);
                return false;
            }
        };

        let mut any_error = false;
        // Try to deserialize each record
        for result in reader.deserialize::<S>() {
            match result {
                Ok(_) => (),
                Err(e) => {
                    any_error = true;
                    let columns = match get_column_names(filepath) {
                        Ok(c) => c,
                        Err(e) => {
                            error!("Failed to get column names: {}", e);
                            continue;
                        }
                    };

                    match *e.kind() {
                        csv::ErrorKind::Deserialize {
                            pos: Some(ref pos),
                            ref err,
                            ..
                        } => {
                            error!(
                                "CSV does not match the related table, line: {}, field: {}, details: {}",
                                pos.line(),
                                columns[err.field().unwrap() as usize],
                                err.kind()
                            )
                        }
                        _ => {
                            error!("Failed to parse CSV: {}", e);
                        }
                    }

                    continue;
                }
            }
        }

        if !any_error {
            info!("{}", format!("{} is valid.", filepath.display()));
            return true;
        } else {
            error!("{}", format!("{} is invalid.", filepath.display()));
            return false;
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Object)]
pub struct RecordResponse<S>
where
    S: Serialize
        + for<'r> sqlx::FromRow<'r, sqlx::postgres::PgRow>
        + std::fmt::Debug
        + std::marker::Unpin
        + Send
        + Sync
        + poem_openapi::types::Type
        + poem_openapi::types::ParseFromJSON
        + poem_openapi::types::ToJSON,
{
    /// data
    pub records: Vec<S>,
    /// total num
    pub total: u64,
    /// current page index
    pub page: u64,
    /// default 10
    pub page_size: u64,
}

impl<
        S: Serialize
            + for<'r> sqlx::FromRow<'r, sqlx::postgres::PgRow>
            + std::fmt::Debug
            + std::marker::Unpin
            + Send
            + Sync
            + poem_openapi::types::Type
            + poem_openapi::types::ParseFromJSON
            + poem_openapi::types::ToJSON,
    > RecordResponse<S>
{
    pub async fn get_records(
        pool: &sqlx::PgPool,
        database_name: &str,
        query: &Option<ComposeQuery>,
        page: Option<u64>,
        page_size: Option<u64>,
    ) -> Result<RecordResponse<S>, anyhow::Error> {
        let mut query_str = match query {
            Some(ComposeQuery::QueryItem(item)) => item.format(),
            Some(ComposeQuery::ComposeQueryItem(item)) => item.format(),
            None => "".to_string(),
        };

        if query_str.is_empty() {
            query_str = "1=1".to_string();
        };

        let page = match page {
            Some(page) => page,
            None => 1,
        };

        let page_size = match page_size {
            Some(page_size) => page_size,
            None => 10,
        };

        let sql_str = format!(
            "SELECT * FROM {} WHERE {} ORDER BY id LIMIT {} OFFSET {}",
            database_name,
            query_str,
            page_size,
            (page - 1) * page_size
        );

        let records = sqlx::query_as::<_, S>(sql_str.as_str())
            .fetch_all(pool)
            .await?;

        let sql_str = format!("SELECT COUNT(*) FROM {} WHERE {}", database_name, query_str);

        let total = sqlx::query_as::<_, (i64,)>(sql_str.as_str())
            .fetch_one(pool)
            .await?;

        AnyOk(RecordResponse {
            records: records,
            total: total.0 as u64,
            page: page,
            page_size: page_size,
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Object, sqlx::FromRow)]
pub struct Entity {
    #[oai(validator(max_length = 64, pattern = "^[A-Z0-9\\-]+:[a-z0-9A-Z\\.]+$"))]
    pub id: String,
    #[oai(validator(max_length = 64))]
    pub name: String,
    #[oai(validator(max_length = 64))]
    pub label: String,
    #[oai(validator(max_length = 64))]
    pub resource: String,
    pub description: Option<String>,
}

impl CheckData for Entity {}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Object, sqlx::FromRow)]
pub struct EntityMetadata {
    #[oai(read_only)]
    // Ignore this field when deserialize from json
    #[serde(skip_deserializing)]
    pub id: i32,
    #[oai(validator(max_length = 64))]
    pub resource: String,
    #[oai(validator(max_length = 64, pattern = "^[A-Z][a-z]+$"))]
    pub entity_type: String,
    pub entity_count: i64,
}

impl CheckData for EntityMetadata {}

impl EntityMetadata {
    pub async fn get_entity_metadata(
        pool: &sqlx::PgPool,
    ) -> Result<Vec<EntityMetadata>, anyhow::Error> {
        let sql_str = "SELECT * FROM biomedgps_entity_metadata";
        let entity_metadata = sqlx::query_as::<_, EntityMetadata>(sql_str)
            .fetch_all(pool)
            .await?;

        AnyOk(entity_metadata)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Object, sqlx::FromRow)]
pub struct RelationMetadata {
    #[oai(read_only)]
    // Ignore this field when deserialize from json
    #[serde(skip_deserializing)]
    pub id: i32,
    #[oai(validator(max_length = 64))]
    pub resource: String,
    #[oai(validator(max_length = 64))]
    pub relation_type: String,
    pub relation_count: i64,
    #[oai(validator(max_length = 64, pattern = "^[A-Z][a-z]+$"))]
    pub start_entity_type: String,
    #[oai(validator(max_length = 64, pattern = "^[A-Z][a-z]+$"))]
    pub end_entity_type: String,
}

impl CheckData for RelationMetadata {}

impl RelationMetadata {
    pub async fn get_relation_metadata(
        pool: &sqlx::PgPool,
    ) -> Result<Vec<RelationMetadata>, anyhow::Error> {
        let sql_str = "SELECT * FROM bioemdgps_relation_metadata";
        let relation_metadata = sqlx::query_as::<_, RelationMetadata>(sql_str)
            .fetch_all(pool)
            .await?;

        AnyOk(relation_metadata)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Object, sqlx::FromRow)]
pub struct KnowledgeCuration {
    pub relation_id: i32,
    #[oai(validator(max_length = 64))]
    pub relation_type: String,
    #[oai(validator(max_length = 64))]
    pub source_name: String,
    #[oai(validator(max_length = 64, pattern = "^[A-Z][a-z]+$"))]
    pub source_type: String,
    #[oai(validator(max_length = 64, pattern = "^[A-Z0-9\\-]+:[a-z0-9A-Z\\.]+$"))]
    pub source_id: String,
    #[oai(validator(max_length = 64))]
    pub target_name: String,
    #[oai(validator(max_length = 64, pattern = "^[A-Z][a-z]+$"))]
    pub target_type: String,
    #[oai(validator(max_length = 64, pattern = "^[A-Z0-9\\-]+:[a-z0-9A-Z\\.]+$"))]
    pub target_id: String,
    pub key_sentence: String,
    #[oai(read_only)]
    #[serde(skip_deserializing)]
    #[serde(with = "ts_seconds")]
    pub created_at: DateTime<Utc>,
    #[oai(validator(max_length = 64))]
    pub curator: String,
    pub pmid: i64,
}

impl KnowledgeCuration {
    pub async fn insert(&self, pool: &sqlx::PgPool) -> Result<KnowledgeCuration, anyhow::Error> {
        let sql_str = "INSERT INTO biomedgps_knowledge_curation (relation_id, relation_type, source_name, source_type, source_id, target_name, target_type, target_id, key_sentence, created_at, curator, pmid) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, now(), $10, $11) RETURNING *";
        let knowledge_curation = sqlx::query_as::<_, KnowledgeCuration>(sql_str)
            .bind(&self.relation_id)
            .bind(&self.relation_type)
            .bind(&self.source_name)
            .bind(&self.source_type)
            .bind(&self.source_id)
            .bind(&self.target_name)
            .bind(&self.target_type)
            .bind(&self.target_id)
            .bind(&self.key_sentence)
            .bind(&self.curator)
            .bind(&self.pmid)
            .fetch_one(pool)
            .await?;

        AnyOk(knowledge_curation)
    }

    pub async fn update(
        &self,
        pool: &sqlx::PgPool,
        id: &str,
    ) -> Result<KnowledgeCuration, anyhow::Error> {
        let sql_str = "UPDATE biomedgps_knowledge_curation SET relation_type = $1, source_name = $2, source_type = $3, source_id = $4, target_name = $5, target_type = $6, target_id = $7, key_sentence = $8, created_at = now(), pmid = $9 WHERE id = $10 RETURNING *";
        let knowledge_curation = sqlx::query_as::<_, KnowledgeCuration>(sql_str)
            .bind(&self.relation_type)
            .bind(&self.source_name)
            .bind(&self.source_type)
            .bind(&self.source_id)
            .bind(&self.target_name)
            .bind(&self.target_type)
            .bind(&self.target_id)
            .bind(&self.key_sentence)
            .bind(&self.pmid)
            .bind(id)
            .fetch_one(pool)
            .await?;

        AnyOk(knowledge_curation)
    }

    pub async fn delete(
        pool: &sqlx::PgPool,
        id: &str,
    ) -> Result<KnowledgeCuration, anyhow::Error> {
        let sql_str = "DELETE FROM biomedgps_knowledge_curation WHERE id = $1 RETURNING *";
        let knowledge_curation = sqlx::query_as::<_, KnowledgeCuration>(sql_str)
            .bind(id)
            .fetch_one(pool)
            .await?;

        AnyOk(knowledge_curation)
    }
}

impl CheckData for KnowledgeCuration {}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Object, sqlx::FromRow)]
pub struct Relation {
    #[oai(read_only)]
    // Ignore this field when deserialize from json
    #[serde(skip_deserializing)]
    pub id: i32,
    #[oai(validator(max_length = 64))]
    pub relation_type: String,
    #[oai(validator(max_length = 64, pattern = "^[A-Z0-9\\-]+:[a-z0-9A-Z\\.]+$"))]
    pub source_id: String,
    #[oai(validator(max_length = 64, pattern = "^[A-Z][a-z]+$"))]
    pub source_type: String,
    #[oai(validator(max_length = 64, pattern = "^[A-Z0-9\\-]+:[a-z0-9A-Z\\.]+$"))]
    pub target_id: String,
    #[oai(validator(max_length = 64, pattern = "^[A-Z][a-z]+$"))]
    pub target_type: String,
    pub resource: String,
}

impl CheckData for Relation {}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Object, sqlx::FromRow)]
pub struct Entity2D {
    pub embedding_id: i64,
    #[oai(validator(max_length = 64, pattern = "^[A-Z0-9\\-]+:[a-z0-9A-Z\\.]+$"))]
    pub entity_id: String,
    #[oai(validator(max_length = 64, pattern = "^[A-Z][a-z]+$"))]
    pub entity_type: String,
    #[oai(validator(max_length = 64))]
    pub entity_name: String,
    pub umap_x: f64,
    pub umap_y: f64,
    pub tsne_x: f64,
    pub tsne_y: f64,
}

impl CheckData for Entity2D {}

// UUID Pattern: https://stackoverflow.com/questions/136505/searching-for-uuids-in-text-with-regex

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Object, sqlx::FromRow)]
pub struct Subgraph {
    #[oai(read_only)]
    #[oai(validator(
        max_length = 36,
        pattern = "^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$"
    ))]
    pub id: String,
    #[oai(validator(max_length = 64))]
    pub name: String,
    pub description: Option<String>,
    pub payload: String, // json string, e.g. {"nodes": [], "edges": []}. how to validate json string?
    #[oai(read_only)]
    #[serde(skip_deserializing)]
    #[serde(with = "ts_seconds")]
    pub created_time: DateTime<Utc>,
    #[oai(validator(max_length = 36))]
    pub owner: String,
    #[oai(validator(max_length = 36))]
    pub version: String,
    #[oai(validator(max_length = 36))]
    pub db_version: String,
    #[oai(validator(
        max_length = 36,
        pattern = "^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$"
    ))]
    pub parent: Option<String>, // parent subgraph id, it is same as id if it is a root subgraph (no parent), otherwise it is the parent subgraph id
}

impl CheckData for Subgraph {}

impl Subgraph {
    pub async fn insert(&self, pool: &sqlx::PgPool) -> Result<Subgraph, anyhow::Error> {
        let id = uuid::Uuid::new_v4().to_string();
        let parent = if self.parent.is_none() {
            id.clone()
        } else {
            self.parent.clone().unwrap()
        };

        let sql_str = "INSERT INTO biomedgps_subgraph (id, name, description, payload, created_time, owner, version, db_version, parent) VALUES ($1, $2, $3, $4, now(), $5, $6, $7, $8) RETURNING *";
        let subgraph = sqlx::query_as::<_, Subgraph>(sql_str)
            .bind(id)
            .bind(&self.name)
            .bind(&self.description)
            .bind(&self.payload)
            .bind(&self.owner)
            .bind(&self.version)
            .bind(&self.db_version)
            .bind(parent)
            .fetch_one(pool)
            .await?;

        AnyOk(subgraph)
    }

    pub async fn update(&self, pool: &sqlx::PgPool, id: &str) -> Result<Subgraph, anyhow::Error> {
        let sql_str = "UPDATE biomedgps_subgraph SET name = $1, description = $2, payload = $3, WHERE id = $4 RETURNING *";
        let subgraph = sqlx::query_as::<_, Subgraph>(sql_str)
            .bind(&self.name)
            .bind(&self.description)
            .bind(&self.payload)
            .bind(id)
            .fetch_one(pool)
            .await?;

        AnyOk(subgraph)
    }

    pub async fn delete(pool: &sqlx::PgPool, id: &str) -> Result<Subgraph, anyhow::Error> {
        let sql_str = "DELETE FROM biomedgps_subgraph WHERE id = $1 RETURNING *";
        let subgraph = sqlx::query_as::<_, Subgraph>(sql_str)
            .bind(id)
            .fetch_one(pool)
            .await?;

        AnyOk(subgraph)
    }
}
