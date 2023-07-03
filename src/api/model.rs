use crate::api::util::get_delimiter;
use crate::query::sql_builder::{ComposeQuery, QueryItem};
use anyhow::Ok as AnyOk;
use chrono::serde::ts_seconds;
use chrono::{DateTime, Utc};
use lazy_static::lazy_static;
use log::{debug, error, info, warn};
use poem_openapi::Object;
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::{error::Error, fmt, path::PathBuf};
use validator::Validate;

const ENTITY_NAME_MAX_LENGTH: u64 = 64;
const DEFAULT_MAX_LENGTH: u64 = 64;

lazy_static! {
    static ref ENTITY_LABEL_REGEX: Regex = Regex::new(r"^[A-Z][a-z]+$").unwrap();
    static ref ENTITY_ID_REGEX: Regex = Regex::new(r"^[A-Z0-9\-]+:[a-z0-9A-Z\.]+$").unwrap();
}

#[derive(Debug)]
pub struct ValidationError {
    details: String,
}

impl ValidationError {
    pub fn new(msg: &str) -> ValidationError {
        ValidationError {
            details: msg.to_string(),
        }
    }
}

impl fmt::Display for ValidationError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.details)
    }
}

impl Error for ValidationError {
    fn description(&self) -> &str {
        &self.details
    }

    fn cause(&self) -> Option<&dyn Error> {
        // Generic error, underlying cause isn't tracked.
        None
    }
}

pub trait CheckData {
    fn check_csv_is_valid(filepath: &PathBuf) -> Vec<Box<dyn Error>>;

    // Implement the check function
    fn check_csv_is_valid_default<
        S: for<'de> serde::Deserialize<'de> + Validate + std::fmt::Debug,
    >(
        filepath: &PathBuf,
    ) -> Vec<Box<dyn Error>> {
        let mut validation_errors: Vec<Box<dyn Error>> = vec![];
        let delimiter = match get_delimiter(filepath) {
            Ok(d) => d,
            Err(e) => {
                validation_errors.push(Box::new(ValidationError::new(&format!(
                    "Failed to get delimiter: {}",
                    e
                ))));
                return validation_errors;
            }
        };

        debug!("The delimiter is: {:?}", delimiter as char);
        // Build the CSV reader
        let mut reader = match csv::ReaderBuilder::new()
            .delimiter(delimiter)
            .from_path(filepath)
        {
            Ok(r) => r,
            Err(e) => {
                validation_errors.push(Box::new(ValidationError::new(&format!(
                    "Failed to read CSV: {}",
                    e
                ))));
                return validation_errors;
            }
        };

        // Try to deserialize each record
        info!("Start to deserialize the CSV file: {:?}", reader.headers());
        let mut line_number = 1;
        for result in reader.deserialize::<S>() {
            match result {
                Ok(data) => match data.validate() {
                    Ok(_) => {
                        continue;
                    }
                    Err(e) => {
                        validation_errors.push(Box::new(ValidationError::new(&format!(
                            "Failed to validate the data, line: {}, details: {}",
                            line_number, e
                        ))));
                        continue;
                    }
                },
                Err(e) => {
                    let columns = match Self::get_column_names(filepath) {
                        Ok(c) => c,
                        Err(e) => {
                            validation_errors.push(Box::new(ValidationError::new(&format!(
                                "Failed to get column names: {}",
                                e
                            ))));
                            continue;
                        }
                    };

                    match *e.kind() {
                        csv::ErrorKind::Deserialize {
                            pos: Some(ref pos),
                            ref err,
                            ..
                        } => {
                            validation_errors.push(Box::new(ValidationError::new(&format!(
                                "Failed to deserialize the data, line: {}, column: {}, details: {}",
                                pos.line(),
                                columns[err.field().unwrap() as usize],
                                err.kind()
                            ))));
                        }
                        _ => {
                            validation_errors.push(Box::new(ValidationError::new(&format!(
                                "Failed to parse CSV: {}",
                                e
                            ))));
                        }
                    }

                    continue;
                }
            };
        }

        validation_errors
    }

    fn fields() -> Vec<String>;

    /// Select the columns to keep
    /// Return the path of the output file which is a temporary file
    fn select_expected_columns(
        in_filepath: &PathBuf,
        out_filepath: &PathBuf,
    ) -> Result<(), Box<dyn Error>> {
        let delimiter = get_delimiter(in_filepath)?;
        let mut reader = csv::ReaderBuilder::new()
            .delimiter(delimiter)
            .from_path(in_filepath)?;

        let headers = reader.headers()?.clone();

        // Identify the indices of the columns to keep
        let indices_to_keep: Vec<usize> = headers
            .iter()
            .enumerate()
            .filter_map(|(i, h)| {
                if Self::fields().contains(&h.to_string()) {
                    Some(i)
                } else {
                    None
                }
            })
            .collect();

        let mut wtr = csv::WriterBuilder::new()
            .delimiter(delimiter)
            .from_writer(std::fs::File::create(out_filepath)?);

        // Write the headers of the columns to keep to the output file
        let headers_to_keep: Vec<&str> = indices_to_keep.iter().map(|&i| &headers[i]).collect();
        wtr.write_record(&headers_to_keep)?;

        // Read each record, keep only the desired fields, and write to the output file
        for result in reader.records() {
            let record = result?;
            let record_to_keep: Vec<&str> = indices_to_keep.iter().map(|&i| &record[i]).collect();
            wtr.write_record(&record_to_keep)?;
        }

        // Flush the writer to ensure all output is written
        wtr.flush()?;

        info!("Select the columns to keep successfully.");
        debug!(
            "The path of the temporary file is: {}",
            out_filepath.display()
        );

        Ok(())
    }

    fn get_column_names(filepath: &PathBuf) -> Result<Vec<String>, Box<dyn Error>> {
        let delimiter = get_delimiter(filepath)?;
        let mut reader = csv::ReaderBuilder::new()
            .delimiter(delimiter)
            .from_path(filepath)?;

        let headers = reader.headers()?;
        let mut column_names = Vec::new();
        let expected_columns = Self::fields();
        for header in headers {
            let column = header.to_string();
            // Don't need to check whether all the columns are in the input file, because we have already checked it in the function `check_csv_is_valid`.
            if expected_columns.contains(&column) {
                column_names.push(column);
            } else {
                continue;
            }
        }

        Ok(column_names)
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

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Object, sqlx::FromRow, Validate)]
pub struct Entity {
    #[validate(length(max = "DEFAULT_MAX_LENGTH"))]
    #[validate(regex = "ENTITY_ID_REGEX")]
    #[oai(validator(max_length = 64, pattern = "^[A-Z0-9\\-]+:[a-z0-9A-Z\\.]+$"))]
    pub id: String,

    #[oai(validator(max_length = 255))]
    #[validate(length(max = "ENTITY_NAME_MAX_LENGTH"))]
    pub name: String,

    #[validate(length(max = "DEFAULT_MAX_LENGTH"))]
    #[validate(regex = "ENTITY_LABEL_REGEX")]
    #[oai(validator(max_length = 64, pattern = "^[A-Z][a-z]+$"))]
    pub label: String,

    #[validate(length(max = "DEFAULT_MAX_LENGTH"))]
    #[oai(validator(max_length = 64))]
    pub resource: String,

    pub description: Option<String>,
}

impl CheckData for Entity {
    fn check_csv_is_valid(filepath: &PathBuf) -> Vec<Box<dyn Error>> {
        Self::check_csv_is_valid_default::<Entity>(filepath)
    }

    fn fields() -> Vec<String> {
        vec![
            "id".to_string(),
            "name".to_string(),
            "label".to_string(),
            "resource".to_string(),
            "description".to_string(),
        ]
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Object, sqlx::FromRow, Validate)]
pub struct EntityMetadata {
    #[oai(read_only)]
    // Ignore this field when deserialize from json
    #[serde(skip_deserializing)]
    pub id: i32,

    #[validate(length(max = "DEFAULT_MAX_LENGTH"))]
    #[oai(validator(max_length = 64))]
    pub resource: String,

    #[validate(regex = "ENTITY_LABEL_REGEX")]
    #[validate(length(max = "DEFAULT_MAX_LENGTH"))]
    #[oai(validator(max_length = 64, pattern = "^[A-Z][a-z]+$"))]
    pub entity_type: String,

    pub entity_count: i64,
}

impl CheckData for EntityMetadata {
    fn check_csv_is_valid(filepath: &PathBuf) -> Vec<Box<dyn Error>> {
        Self::check_csv_is_valid_default::<EntityMetadata>(filepath)
    }

    fn fields() -> Vec<String> {
        vec![
            "resource".to_string(),
            "entity_type".to_string(),
            "entity_count".to_string(),
        ]
    }
}

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

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Object, sqlx::FromRow, Validate)]
pub struct RelationMetadata {
    #[oai(read_only)]
    // Ignore this field when deserialize from json
    #[serde(skip_deserializing)]
    pub id: i32,

    #[validate(length(max = "DEFAULT_MAX_LENGTH"))]
    #[oai(validator(max_length = 64))]
    pub resource: String,

    #[validate(length(max = "DEFAULT_MAX_LENGTH"))]
    #[oai(validator(max_length = 64))]
    pub relation_type: String,

    pub relation_count: i64,

    #[validate(regex = "ENTITY_LABEL_REGEX")]
    #[validate(length(max = "DEFAULT_MAX_LENGTH"))]
    #[oai(validator(max_length = 64, pattern = "^[A-Z][a-z]+$"))]
    pub start_entity_type: String,

    #[validate(regex = "ENTITY_LABEL_REGEX")]
    #[validate(length(max = "DEFAULT_MAX_LENGTH"))]
    #[oai(validator(max_length = 64, pattern = "^[A-Z][a-z]+$"))]
    pub end_entity_type: String,
}

impl CheckData for RelationMetadata {
    fn check_csv_is_valid(filepath: &PathBuf) -> Vec<Box<dyn Error>> {
        Self::check_csv_is_valid_default::<RelationMetadata>(filepath)
    }

    fn fields() -> Vec<String> {
        vec![
            "resource".to_string(),
            "relation_type".to_string(),
            "relation_count".to_string(),
            "start_entity_type".to_string(),
            "end_entity_type".to_string(),
        ]
    }
}

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

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Object, sqlx::FromRow, Validate)]
pub struct KnowledgeCuration {
    pub relation_id: i32,

    #[validate(length(max = "DEFAULT_MAX_LENGTH"))]
    #[oai(validator(max_length = 64))]
    pub relation_type: String,

    #[validate(length(max = "ENTITY_NAME_MAX_LENGTH"))]
    #[oai(validator(max_length = 255))]
    pub source_name: String,

    #[validate(regex = "ENTITY_LABEL_REGEX")]
    #[validate(length(max = "DEFAULT_MAX_LENGTH"))]
    #[oai(validator(max_length = 64, pattern = "^[A-Z][a-z]+$"))]
    pub source_type: String,

    #[validate(length(max = "DEFAULT_MAX_LENGTH"))]
    #[validate(regex = "ENTITY_ID_REGEX")]
    #[oai(validator(max_length = 64, pattern = "^[A-Z0-9\\-]+:[a-z0-9A-Z\\.]+$"))]
    pub source_id: String,

    #[validate(length(max = "ENTITY_NAME_MAX_LENGTH"))]
    #[oai(validator(max_length = 255))]
    pub target_name: String,

    #[validate(regex = "ENTITY_LABEL_REGEX")]
    #[validate(length(max = "DEFAULT_MAX_LENGTH"))]
    #[oai(validator(max_length = 64, pattern = "^[A-Z][a-z]+$"))]
    pub target_type: String,

    #[validate(length(max = "DEFAULT_MAX_LENGTH"))]
    #[validate(regex = "ENTITY_ID_REGEX")]
    #[oai(validator(max_length = 64, pattern = "^[A-Z0-9\\-]+:[a-z0-9A-Z\\.]+$"))]
    pub target_id: String,

    pub key_sentence: String,

    #[oai(read_only)]
    #[serde(skip_deserializing)]
    #[serde(with = "ts_seconds")]
    pub created_at: DateTime<Utc>,

    #[validate(length(max = "DEFAULT_MAX_LENGTH"))]
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

    pub async fn delete(pool: &sqlx::PgPool, id: &str) -> Result<KnowledgeCuration, anyhow::Error> {
        let sql_str = "DELETE FROM biomedgps_knowledge_curation WHERE id = $1 RETURNING *";
        let knowledge_curation = sqlx::query_as::<_, KnowledgeCuration>(sql_str)
            .bind(id)
            .fetch_one(pool)
            .await?;

        AnyOk(knowledge_curation)
    }
}

impl CheckData for KnowledgeCuration {
    fn check_csv_is_valid(filepath: &PathBuf) -> Vec<Box<dyn Error>> {
        Self::check_csv_is_valid_default::<KnowledgeCuration>(filepath)
    }

    fn fields() -> Vec<String> {
        vec![
            "relation_type".to_string(),
            "source_name".to_string(),
            "source_type".to_string(),
            "source_id".to_string(),
            "target_name".to_string(),
            "target_type".to_string(),
            "target_id".to_string(),
            "key_sentence".to_string(),
            "curator".to_string(),
            "pmid".to_string(),
        ]
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Object, sqlx::FromRow, Validate)]
pub struct Relation {
    #[oai(read_only)]
    // Ignore this field when deserialize from json
    #[serde(skip_deserializing)]
    pub id: i32,

    #[validate(length(max = "DEFAULT_MAX_LENGTH"))]
    #[oai(validator(max_length = 64))]
    pub relation_type: String,

    #[validate(length(max = "DEFAULT_MAX_LENGTH"))]
    #[validate(regex = "ENTITY_ID_REGEX")]
    #[oai(validator(max_length = 64, pattern = "^[A-Z0-9\\-]+:[a-z0-9A-Z\\.]+$"))]
    pub source_id: String,

    #[validate(length(max = "DEFAULT_MAX_LENGTH"))]
    #[validate(regex = "ENTITY_LABEL_REGEX")]
    #[oai(validator(max_length = 64, pattern = "^[A-Z][a-z]+$"))]
    pub source_type: String,

    #[validate(length(max = "DEFAULT_MAX_LENGTH"))]
    #[validate(regex = "ENTITY_ID_REGEX")]
    #[oai(validator(max_length = 64, pattern = "^[A-Z0-9\\-]+:[a-z0-9A-Z\\.]+$"))]
    pub target_id: String,

    #[validate(length(max = "DEFAULT_MAX_LENGTH"))]
    #[validate(regex = "ENTITY_LABEL_REGEX")]
    #[oai(validator(max_length = 64, pattern = "^[A-Z][a-z]+$"))]
    pub target_type: String,

    pub resource: String,
}

impl CheckData for Relation {
    fn check_csv_is_valid(filepath: &PathBuf) -> Vec<Box<dyn Error>> {
        Self::check_csv_is_valid_default::<Relation>(filepath)
    }

    fn fields() -> Vec<String> {
        vec![
            "relation_type".to_string(),
            "source_id".to_string(),
            "source_type".to_string(),
            "target_id".to_string(),
            "target_type".to_string(),
            "resource".to_string(),
        ]
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Object, sqlx::FromRow, Validate)]
pub struct Entity2D {
    pub embedding_id: i64,

    #[validate(length(max = "DEFAULT_MAX_LENGTH"))]
    #[validate(regex = "ENTITY_ID_REGEX")]
    #[oai(validator(max_length = 64, pattern = "^[A-Z0-9\\-]+:[a-z0-9A-Z\\.]+$"))]
    pub entity_id: String,

    #[validate(length(max = "DEFAULT_MAX_LENGTH"))]
    #[validate(regex = "ENTITY_LABEL_REGEX")]
    #[oai(validator(max_length = 64, pattern = "^[A-Z][a-z]+$"))]
    pub entity_type: String,

    #[validate(length(max = "DEFAULT_MAX_LENGTH"))]
    #[oai(validator(max_length = 255))]
    pub entity_name: String,

    pub umap_x: f64,

    pub umap_y: f64,

    pub tsne_x: f64,

    pub tsne_y: f64,
}

impl CheckData for Entity2D {
    fn check_csv_is_valid(filepath: &PathBuf) -> Vec<Box<dyn Error>> {
        Self::check_csv_is_valid_default::<Entity2D>(filepath)
    }

    fn fields() -> Vec<String> {
        vec![
            "embedding_id".to_string(),
            "entity_id".to_string(),
            "entity_type".to_string(),
            "entity_name".to_string(),
            "umap_x".to_string(),
            "umap_y".to_string(),
            "tsne_x".to_string(),
            "tsne_y".to_string(),
        ]
    }
}

// UUID Pattern: https://stackoverflow.com/questions/136505/searching-for-uuids-in-text-with-regex

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Object, sqlx::FromRow, Validate)]
pub struct Subgraph {
    #[oai(read_only)]
    #[oai(validator(
        max_length = 36,
        pattern = "^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$"
    ))]
    pub id: String,

    #[validate(length(max = "DEFAULT_MAX_LENGTH"))]
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

impl CheckData for Subgraph {
    fn check_csv_is_valid(filepath: &PathBuf) -> Vec<Box<dyn Error>> {
        Self::check_csv_is_valid_default::<Subgraph>(filepath)
    }

    fn fields() -> Vec<String> {
        vec![
            "name".to_string(),
            "description".to_string(),
            "payload".to_string(),
            "owner".to_string(),
            "version".to_string(),
            "db_version".to_string(),
            "parent".to_string(),
        ]
    }
}

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
