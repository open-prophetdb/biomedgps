use crate::query::sql_builder::{ComposeQuery, QueryItem};
use anyhow::Ok;
use chrono::serde::ts_seconds;
use chrono::{DateTime, Utc};
use log::{debug, info, warn};
use poem_openapi::Object;
use serde::{Deserialize, Serialize};

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
        &self,
        pool: &sqlx::PgPool,
        database_name: &str,
        query: Option<&ComposeQuery>,
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

        Ok(RecordResponse {
            records: records,
            total: total.0 as u64,
            page: page,
            page_size: page_size,
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Object, sqlx::FromRow)]
pub struct Entity {
    #[oai(read_only)]
    // Ignore this field when deserialize from json
    #[serde(skip_deserializing)]
    pub id: i32,
    #[oai(validator(max_length = 64))]
    pub name: String,
    #[oai(validator(max_length = 64))]
    pub label: String,
    #[oai(validator(max_length = 64))]
    pub resource: String,
    pub description: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Object)]
pub struct EntityRecordResponse {
    /// data
    pub records: Vec<Entity>,
    /// total num
    pub total: u64,
    /// current page index
    pub page: u64,
    /// default 10
    pub page_size: u64,
}

impl EntityRecordResponse {
    pub async fn get_entities(
        pool: &sqlx::PgPool,
        query: &ComposeQuery,
        page: u64,
        page_size: u64,
    ) -> Result<EntityRecordResponse, anyhow::Error> {
        let mut query_str = match query {
            ComposeQuery::QueryItem(item) => item.format(),
            ComposeQuery::ComposeQueryItem(item) => item.format(),
        };
        if query_str.is_empty() {
            query_str = "1=1".to_string();
        };

        let sql_str = format!(
            "SELECT * FROM biomedgps_entity WHERE {} ORDER BY id LIMIT {} OFFSET {}",
            query_str,
            page_size,
            (page - 1) * page_size
        );

        let entities = sqlx::query_as::<_, Entity>(sql_str.as_str())
            .fetch_all(pool)
            .await?;

        let sql_str = format!("SELECT COUNT(*) FROM biomedgps_entity WHERE {}", query_str);

        let total = sqlx::query_as::<_, (i64,)>(sql_str.as_str())
            .fetch_one(pool)
            .await?;

        Ok(EntityRecordResponse {
            records: entities,
            total: total.0 as u64,
            page: page,
            page_size: page_size,
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Object, sqlx::FromRow)]
pub struct EntityMetadata {
    #[oai(read_only)]
    // Ignore this field when deserialize from json
    #[serde(skip_deserializing)]
    pub id: i32,
    #[oai(validator(max_length = 64))]
    pub resource: String,
    #[oai(validator(max_length = 64))]
    pub entity_type: String,
    pub entity_count: i64,
}

impl EntityMetadata {
    pub async fn get_entity_metadata(
        pool: &sqlx::PgPool,
    ) -> Result<Vec<EntityMetadata>, anyhow::Error> {
        let sql_str = "SELECT * FROM biomedgps_entity_metadata";
        let entity_metadata = sqlx::query_as::<_, EntityMetadata>(sql_str)
            .fetch_all(pool)
            .await?;

        Ok(entity_metadata)
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
    #[oai(validator(max_length = 64))]
    pub start_entity_type: String,
    #[oai(validator(max_length = 64))]
    pub end_entity_type: String,
}

impl RelationMetadata {
    pub async fn get_relation_metadata(
        pool: &sqlx::PgPool,
    ) -> Result<Vec<RelationMetadata>, anyhow::Error> {
        let sql_str = "SELECT * FROM bioemdgps_relation_metadata";
        let relation_metadata = sqlx::query_as::<_, RelationMetadata>(sql_str)
            .fetch_all(pool)
            .await?;

        Ok(relation_metadata)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Object, sqlx::FromRow)]
pub struct KnowledgeCuration {
    #[oai(read_only)]
    // Ignore this field when deserialize from json
    #[serde(skip_deserializing)]
    pub relation_id: i32,
    #[oai(validator(max_length = 64))]
    pub relation_type: String,
    #[oai(validator(max_length = 64))]
    pub source_name: String,
    #[oai(validator(max_length = 64))]
    pub source_type: String,
    #[oai(validator(max_length = 64))]
    pub source_id: String,
    #[oai(validator(max_length = 64))]
    pub target_name: String,
    #[oai(validator(max_length = 64))]
    pub target_type: String,
    #[oai(validator(max_length = 64))]
    pub target_id: String,
    pub key_sentence: String,
    #[serde(with = "ts_seconds")]
    pub created_at: DateTime<Utc>,
    #[oai(validator(max_length = 64))]
    pub curator: String,
    pub pmid: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Object)]
pub struct KnowledgeCurationRecordResponse {
    /// data
    pub records: Vec<KnowledgeCuration>,
    /// total num
    pub total: u64,
    /// current page index
    pub page: u64,
    /// default 10
    pub page_size: u64,
}

impl KnowledgeCurationRecordResponse {
    pub async fn get_knowledges(
        pool: &sqlx::PgPool,
        query: &ComposeQuery,
        page: u64,
        page_size: u64,
    ) -> Result<KnowledgeCurationRecordResponse, anyhow::Error> {
        let mut query_str = match query {
            ComposeQuery::QueryItem(item) => item.format(),
            ComposeQuery::ComposeQueryItem(item) => item.format(),
        };
        if query_str.is_empty() {
            query_str = "1=1".to_string();
        };

        let sql_str = format!(
            "SELECT * FROM biomedgps_knowledge_curation WHERE {} ORDER BY id LIMIT {} OFFSET {}",
            query_str,
            page_size,
            (page - 1) * page_size
        );

        let knowledges = sqlx::query_as::<_, KnowledgeCuration>(sql_str.as_str())
            .fetch_all(pool)
            .await?;

        let sql_str = format!(
            "SELECT COUNT(*) FROM biomedgps_knowledge_curation WHERE {}",
            query_str
        );

        let total = sqlx::query_as::<_, (i64,)>(sql_str.as_str())
            .fetch_one(pool)
            .await?;

        Ok(KnowledgeCurationRecordResponse {
            records: knowledges,
            total: total.0 as u64,
            page: page,
            page_size: page_size,
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Object, sqlx::FromRow)]
pub struct Relation {
    #[oai(read_only)]
    // Ignore this field when deserialize from json
    #[serde(skip_deserializing)]
    pub id: i32,
    #[oai(validator(max_length = 64))]
    pub relation_type: String,
    #[oai(validator(max_length = 64))]
    pub source_id: String,
    #[oai(validator(max_length = 64))]
    pub source_type: String,
    #[oai(validator(max_length = 64))]
    pub target_id: String,
    #[oai(validator(max_length = 64))]
    pub target_type: String,
    pub resource: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Object)]
pub struct RelationRecordResponse {
    /// data
    pub records: Vec<Relation>,
    /// total num
    pub total: u64,
    /// current page index
    pub page: u64,
    /// default 10
    pub page_size: u64,
}

impl RelationRecordResponse {
    pub async fn get_relations(
        pool: &sqlx::PgPool,
        query: &ComposeQuery,
        page: u64,
        page_size: u64,
    ) -> Result<RelationRecordResponse, anyhow::Error> {
        let mut query_str = match query {
            ComposeQuery::QueryItem(item) => item.format(),
            ComposeQuery::ComposeQueryItem(item) => item.format(),
        };
        if query_str.is_empty() {
            query_str = "1=1".to_string();
        };

        let sql_str = format!(
            "SELECT * FROM biomedgps_relation WHERE {} ORDER BY id LIMIT {} OFFSET {}",
            query_str,
            page_size,
            (page - 1) * page_size
        );

        let relations = sqlx::query_as::<_, Relation>(sql_str.as_str())
            .fetch_all(pool)
            .await?;

        let sql_str = format!(
            "SELECT COUNT(*) FROM biomedgps_relation WHERE {}",
            query_str
        );

        let total = sqlx::query_as::<_, (i64,)>(sql_str.as_str())
            .fetch_one(pool)
            .await?;

        Ok(RelationRecordResponse {
            records: relations,
            total: total.0 as u64,
            page: page,
            page_size: page_size,
        })
    }
}
