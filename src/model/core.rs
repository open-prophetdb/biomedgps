//! The database schema for the application. These are the models that will be used to interact with the database.

use super::util::{drop_table, get_delimiter, parse_csv_error};
use crate::model::util::match_color;
use crate::pgvector::Vector;
use crate::query_builder::sql_builder::{ComposeQuery, QueryItem};
use anyhow::Ok as AnyOk;
use chrono::serde::ts_seconds;
use chrono::{DateTime, Utc};
use lazy_static::lazy_static;
use log::{debug, error, info, warn};
use poem_openapi::Object;
use regex::Regex;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::{error::Error, fmt, option::Option, path::PathBuf};
use validator::Validate;

const ENTITY_NAME_MAX_LENGTH: u64 = 255;
const DEFAULT_MAX_LENGTH: u64 = 64;
const DEFAULT_MIN_LENGTH: u64 = 1;

lazy_static! {
    pub static ref ENTITY_LABEL_REGEX: Regex = Regex::new(r"^[A-Za-z]+$").unwrap();
    pub static ref ENTITY_ID_REGEX: Regex = Regex::new(r"^[A-Za-z0-9\-]+:[a-z0-9A-Z\.\-_]+$").unwrap();
    // 1.23|-4.56|7.89
    pub static ref EMBEDDING_REGEX: Regex = Regex::new(r"^(?:-?\d+(?:\.\d+)?\|)*-?\d+(?:\.\d+)?$").unwrap();
    pub static ref SUBGRAPH_UUID_REGEX: Regex = Regex::new(r"^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$").unwrap();
    pub static ref JSON_REGEX: Regex = Regex::new(r"^(\{.*\}|\[.*\])$").expect("Failed to compile regex");
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
        info!("Start to check the csv file: {:?}", filepath);
        let mut validation_errors: Vec<Box<dyn Error>> = vec![];
        let delimiter = match get_delimiter(filepath) {
            Ok(d) => d,
            Err(e) => {
                validation_errors.push(Box::new(ValidationError::new(&format!(
                    "Failed to get delimiter: ({})",
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
                    "Failed to read CSV: ({})",
                    e
                ))));
                return validation_errors;
            }
        };

        // Try to deserialize each record
        debug!(
            "Start to deserialize the csv file, real columns: {:?}, expected columns: {:?}",
            reader.headers().unwrap().into_iter().collect::<Vec<_>>(),
            Self::fields()
        );
        let mut line_number = 1;
        for result in reader.deserialize::<S>() {
            line_number += 1;

            match result {
                Ok(data) => match data.validate() {
                    Ok(_) => {
                        continue;
                    }
                    Err(e) => {
                        validation_errors.push(Box::new(ValidationError::new(&format!(
                            "Failed to validate the data, line: {}, details: ({})",
                            line_number, e
                        ))));
                        continue;
                    }
                },
                Err(e) => {
                    let error_msg = parse_csv_error(&e);

                    validation_errors.push(Box::new(ValidationError::new(&error_msg)));

                    continue;
                }
            };
        }

        validation_errors
    }

    fn fields() -> Vec<String>;

    fn unique_fields() -> Vec<String>;

    /// Select the columns to keep
    /// Return the path of the output file which is a temporary file
    fn select_expected_columns(
        in_filepath: &PathBuf,
        out_filepath: &PathBuf,
    ) -> Result<(), Box<dyn Error>> {
        let delimiter = get_delimiter(in_filepath)?;
        debug!("The delimiter is: {:?}", delimiter as char);
        let mut reader = csv::ReaderBuilder::new()
            .delimiter(delimiter)
            .from_path(in_filepath)?;

        let headers = reader.headers()?.clone();
        debug!("The headers are: {:?}", headers);

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

        debug!(
            "The indices of the columns to keep are: {:?}",
            indices_to_keep
        );
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
        table_name: &str,
        query: &Option<ComposeQuery>,
        page: Option<u64>,
        page_size: Option<u64>,
        order_by: Option<&str>,
    ) -> Result<RecordResponse<S>, anyhow::Error> {
        let mut query_str = match query {
            Some(ComposeQuery::QueryItem(item)) => item.format(),
            Some(ComposeQuery::ComposeQueryItem(item)) => item.format(),
            None => "".to_string(),
        };

        if query_str.is_empty() {
            query_str = "1=1".to_string();
        };

        let order_by_str = if order_by.is_none() {
            "".to_string()
        } else {
            format!("ORDER BY {}", order_by.unwrap())
        };

        let pagination_str = if page.is_none() && page_size.is_none() {
            "LIMIT 10 OFFSET 0".to_string()
        } else {
            let page = match page {
                Some(page) => page,
                None => 1,
            };

            let page_size = match page_size {
                Some(page_size) => page_size,
                None => 10,
            };

            let limit = page_size;
            let offset = (page - 1) * page_size;

            format!("LIMIT {} OFFSET {}", limit, offset)
        };

        let sql_str = format!(
            "SELECT * FROM {} WHERE {} {} {}",
            table_name, query_str, order_by_str, pagination_str
        );

        let records = sqlx::query_as::<_, S>(sql_str.as_str())
            .fetch_all(pool)
            .await?;

        let sql_str = format!("SELECT COUNT(*) FROM {} WHERE {}", table_name, query_str);

        let total = sqlx::query_as::<_, (i64,)>(sql_str.as_str())
            .fetch_one(pool)
            .await?;

        AnyOk(RecordResponse {
            records: records,
            total: total.0 as u64,
            page: page.unwrap_or(1),
            page_size: page_size.unwrap_or(10),
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Object, sqlx::FromRow, Validate)]
pub struct Entity {
    // Ignore this field when deserialize from json
    #[serde(skip_deserializing)]
    #[oai(read_only)]
    pub idx: i64,

    #[validate(length(
        max = "DEFAULT_MAX_LENGTH",
        min = "DEFAULT_MIN_LENGTH",
        message = "The length of id should be between 1 and 64."
    ))]
    #[validate(regex(
        path = "ENTITY_ID_REGEX",
        message = "The entity id is invalid. It should match ^[A-Za-z0-9\\-]+:[a-z0-9A-Z\\.\\-_]+$. Such as 'MESH:D000001'."
    ))]
    pub id: String,

    #[validate(length(
        max = "ENTITY_NAME_MAX_LENGTH",
        min = "DEFAULT_MIN_LENGTH",
        message = "The length of name should be between 1 and 64."
    ))]
    pub name: String,

    #[validate(length(
        max = "DEFAULT_MAX_LENGTH",
        min = "DEFAULT_MIN_LENGTH",
        message = "The length of label should be between 1 and 64."
    ))]
    #[validate(regex = "ENTITY_LABEL_REGEX")]
    pub label: String,

    #[validate(length(
        max = "DEFAULT_MAX_LENGTH",
        min = "DEFAULT_MIN_LENGTH",
        message = "The length of resource should be between 1 and 64."
    ))]
    pub resource: String,

    #[oai(skip_serializing_if_is_none)]
    pub description: Option<String>,

    #[oai(skip_serializing_if_is_none)]
    pub taxid: Option<String>,

    #[oai(skip_serializing_if_is_none)]
    pub synonyms: Option<String>,

    #[oai(skip_serializing_if_is_none)]
    pub pmids: Option<String>,

    #[oai(skip_serializing_if_is_none)]
    pub xrefs: Option<String>,
}

impl CheckData for Entity {
    fn check_csv_is_valid(filepath: &PathBuf) -> Vec<Box<dyn Error>> {
        Self::check_csv_is_valid_default::<Entity>(filepath)
    }

    fn unique_fields() -> Vec<String> {
        vec!["id".to_string(), "label".to_string()]
    }

    fn fields() -> Vec<String> {
        vec![
            "id".to_string(),
            "name".to_string(),
            "label".to_string(),
            "resource".to_string(),
            "description".to_string(),
            "taxid".to_string(),
            "synonyms".to_string(),
            "pmids".to_string(),
            "xrefs".to_string(),
        ]
    }
}

fn text2vector<'de, D>(deserializer: D) -> Result<Vector, D::Error>
where
    D: Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    match s
        .split('|')
        .map(|s| s.parse().map_err(serde::de::Error::custom))
        .collect::<Result<Vec<f32>, D::Error>>()
    {
        // More details on https://github.com/pgvector/pgvector-rust#sqlx
        Ok(vec) => Ok(Vector::from(vec)),
        Err(e) => Err(e),
    }
}

#[derive(Debug, Clone, Deserialize, PartialEq)]
pub struct EmbeddingRecordResponse<S>
where
    S: for<'r> sqlx::FromRow<'r, sqlx::postgres::PgRow>
        + std::fmt::Debug
        + std::marker::Unpin
        + Send
        + Sync,
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
        S: for<'r> sqlx::FromRow<'r, sqlx::postgres::PgRow>
            + std::fmt::Debug
            + std::marker::Unpin
            + Send
            + Sync,
    > EmbeddingRecordResponse<S>
{
    pub async fn get_records(
        pool: &sqlx::PgPool,
        table_name: &str,
        query: &Option<ComposeQuery>,
        page: Option<u64>,
        page_size: Option<u64>,
        order_by: Option<&str>,
    ) -> Result<EmbeddingRecordResponse<S>, anyhow::Error> {
        let mut query_str = match query {
            Some(ComposeQuery::QueryItem(item)) => item.format(),
            Some(ComposeQuery::ComposeQueryItem(item)) => item.format(),
            None => "".to_string(),
        };

        if query_str.is_empty() {
            query_str = "1=1".to_string();
        };

        let order_by_str = if order_by.is_none() {
            "".to_string()
        } else {
            format!("ORDER BY {}", order_by.unwrap())
        };

        let pagination_str = if page.is_none() && page_size.is_none() {
            "LIMIT 10 OFFSET 0".to_string()
        } else {
            let page = match page {
                Some(page) => page,
                None => 1,
            };

            let page_size = match page_size {
                Some(page_size) => page_size,
                None => 10,
            };

            let limit = page_size;
            let offset = (page - 1) * page_size;

            format!("LIMIT {} OFFSET {}", limit, offset)
        };

        let sql_str = format!(
            "SELECT * FROM {} WHERE {} {} {}",
            table_name, query_str, order_by_str, pagination_str
        );

        let records = sqlx::query_as::<_, S>(sql_str.as_str())
            .fetch_all(pool)
            .await?;

        let sql_str = format!("SELECT COUNT(*) FROM {} WHERE {}", table_name, query_str);

        let total = sqlx::query_as::<_, (i64,)>(sql_str.as_str())
            .fetch_one(pool)
            .await?;

        AnyOk(EmbeddingRecordResponse {
            records: records,
            total: total.0 as u64,
            page: page.unwrap_or(1),
            page_size: page_size.unwrap_or(10),
        })
    }
}

/// A struct for entity embedding, it is used for import entity embeddings into database from csv file.
/// Only for internal use, not for api.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, sqlx::FromRow, Validate)]
pub struct EntityEmbedding {
    pub embedding_id: i64,

    #[validate(length(
        max = "DEFAULT_MAX_LENGTH",
        min = "DEFAULT_MIN_LENGTH",
        message = "The length of entity_id should be between 1 and 64."
    ))]
    #[validate(regex(
        path = "ENTITY_ID_REGEX",
        message = "The entity id should match ^[A-Za-z0-9\\-]+:[a-z0-9A-Z\\.\\-_]+$. Such as 'MESH:D00001'."
    ))]
    pub entity_id: String,

    #[validate(length(
        max = "ENTITY_NAME_MAX_LENGTH",
        min = "DEFAULT_MIN_LENGTH",
        message = "The length of entity_name should be between 1 and 64."
    ))]
    pub entity_name: String,

    #[validate(length(
        max = "DEFAULT_MAX_LENGTH",
        min = "DEFAULT_MIN_LENGTH",
        message = "The length of entity_type should be between 1 and 64."
    ))]
    #[validate(regex(
        path = "ENTITY_LABEL_REGEX",
        message = "The entity type should match ^[A-Za-z]+$. Such as Disease."
    ))]
    pub entity_type: String,

    #[serde(deserialize_with = "text2vector")]
    pub embedding: Vector,
}

impl EntityEmbedding {
    pub fn new(
        embedding_id: i64,
        entity_id: &str,
        entity_name: &str,
        entity_type: &str,
        embedding: &Vec<f32>,
    ) -> EntityEmbedding {
        EntityEmbedding {
            embedding_id: embedding_id,
            entity_id: entity_id.to_string(),
            entity_name: entity_name.to_string(),
            entity_type: entity_type.to_string(),
            embedding: Vector::from(embedding.clone()),
        }
    }

    pub async fn import_entity_embeddings(
        pool: &sqlx::PgPool,
        filepath: &PathBuf,
        delimiter: u8,
        drop: bool,
    ) -> Result<(), Box<dyn Error>> {
        if drop {
            drop_table(&pool, "biomedgps_entity_embedding").await;
        };

        // Build the CSV reader
        let mut reader = match csv::ReaderBuilder::new()
            .delimiter(delimiter)
            .from_path(filepath)
        {
            Ok(r) => r,
            Err(e) => {
                return Err(Box::new(e));
            }
        };

        for result in reader.deserialize() {
            let record: EntityEmbedding = match result {
                Ok(r) => r,
                Err(e) => {
                    let error_msg = parse_csv_error(&e);
                    return Err(Box::new(ValidationError::new(&error_msg)));
                }
            };

            let sql_str = "INSERT INTO biomedgps_entity_embedding (embedding_id, entity_id, entity_type, entity_name, embedding) VALUES ($1, $2, $3, $4, $5)";

            let query = sqlx::query(&sql_str)
                .bind(record.embedding_id)
                .bind(record.entity_id)
                .bind(record.entity_type)
                .bind(record.entity_name)
                .bind(record.embedding);

            match query.execute(pool).await {
                Ok(_) => {}
                Err(e) => {
                    return Err(Box::new(e));
                }
            };
        }

        Ok(())
    }
}

impl CheckData for EntityEmbedding {
    fn check_csv_is_valid(filepath: &PathBuf) -> Vec<Box<dyn Error>> {
        Self::check_csv_is_valid_default::<EntityEmbedding>(filepath)
    }

    fn unique_fields() -> Vec<String> {
        vec!["entity_id".to_string(), "entity_type".to_string()]
    }

    fn fields() -> Vec<String> {
        vec![
            "embedding_id".to_string(),
            "entity_id".to_string(),
            "entity_type".to_string(),
            "entity_name".to_string(),
            "embedding".to_string(),
        ]
    }
}

#[derive(Debug, Clone, Deserialize, PartialEq, sqlx::FromRow, Validate)]
pub struct RelationEmbedding {
    pub embedding_id: i64,

    #[validate(length(
        max = "DEFAULT_MAX_LENGTH",
        min = "DEFAULT_MIN_LENGTH",
        message = "The length of relation_type should be between 1 and 64."
    ))]
    pub relation_type: String,

    #[serde(deserialize_with = "text2vector")]
    pub embedding: Vector,
}

impl RelationEmbedding {
    pub async fn import_relation_embeddings(
        pool: &sqlx::PgPool,
        filepath: &PathBuf,
        delimiter: u8,
        drop: bool,
    ) -> Result<(), Box<dyn Error>> {
        if drop {
            drop_table(&pool, "biomedgps_relation_embedding").await;
        };

        // Build the CSV reader
        let mut reader = match csv::ReaderBuilder::new()
            .delimiter(delimiter)
            .from_path(filepath)
        {
            Ok(r) => r,
            Err(e) => {
                return Err(Box::new(e));
            }
        };

        for result in reader.deserialize() {
            let record: RelationEmbedding = match result {
                Ok(r) => r,
                Err(e) => {
                    let error_msg = parse_csv_error(&e);
                    return Err(Box::new(ValidationError::new(&error_msg)));
                }
            };

            let sql_str = "INSERT INTO biomedgps_relation_embedding (embedding_id, relation_type, embedding) VALUES ($1, $2, $3)";

            let query = sqlx::query(&sql_str)
                .bind(record.embedding_id)
                .bind(record.relation_type)
                .bind(record.embedding);

            match query.execute(pool).await {
                Ok(_) => {}
                Err(e) => {
                    return Err(Box::new(e));
                }
            };
        }

        Ok(())
    }
}

impl CheckData for RelationEmbedding {
    fn check_csv_is_valid(filepath: &PathBuf) -> Vec<Box<dyn Error>> {
        Self::check_csv_is_valid_default::<RelationEmbedding>(filepath)
    }

    fn unique_fields() -> Vec<String> {
        vec![
            "relation_type".to_string(),
            "source_id".to_string(),
            "source_type".to_string(),
            "target_id".to_string(),
            "target_type".to_string(),
        ]
    }

    fn fields() -> Vec<String> {
        vec![
            "embedding_id".to_string(),
            "relation_type".to_string(),
            "source_id".to_string(),
            "source_type".to_string(),
            "target_id".to_string(),
            "target_type".to_string(),
            "embedding".to_string(),
        ]
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Object)]
pub struct Statistics {
    entity_stat: Vec<EntityMetadata>,
    relation_stat: Vec<RelationMetadata>,
}

impl Statistics {
    pub fn new(
        entity_stat: Vec<EntityMetadata>,
        relation_stat: Vec<RelationMetadata>,
    ) -> Statistics {
        Statistics {
            entity_stat: entity_stat,
            relation_stat: relation_stat,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Object, sqlx::FromRow, Validate)]
pub struct EntityMetadata {
    // Ignore this field when deserialize from json
    #[serde(skip_deserializing)]
    #[oai(read_only)]
    pub id: i64,

    #[validate(length(
        max = "DEFAULT_MAX_LENGTH",
        min = "DEFAULT_MIN_LENGTH",
        message = "The length of resource should be between 1 and 64."
    ))]
    pub resource: String,

    #[validate(regex(
        path = "ENTITY_LABEL_REGEX",
        message = "The entity type should match ^[A-Za-z]+$. Such as Disease."
    ))]
    #[validate(length(
        max = "DEFAULT_MAX_LENGTH",
        min = "DEFAULT_MIN_LENGTH",
        message = "The length of entity_type should be between 1 and 64."
    ))]
    pub entity_type: String,

    pub entity_count: i64,
}

impl CheckData for EntityMetadata {
    fn check_csv_is_valid(filepath: &PathBuf) -> Vec<Box<dyn Error>> {
        Self::check_csv_is_valid_default::<EntityMetadata>(filepath)
    }

    fn unique_fields() -> Vec<String> {
        vec!["resource".to_string(), "entity_type".to_string()]
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
    // Ignore this field when deserialize from json
    #[serde(skip_deserializing)]
    #[oai(read_only)]
    pub id: i64,

    #[validate(length(
        max = "DEFAULT_MAX_LENGTH",
        min = "DEFAULT_MIN_LENGTH",
        message = "The length of resource should be between 1 and 64."
    ))]
    pub resource: String,

    #[validate(length(
        max = "DEFAULT_MAX_LENGTH",
        min = "DEFAULT_MIN_LENGTH",
        message = "The length of relation_type should be between 1 and 64."
    ))]
    pub relation_type: String,

    pub relation_count: i64,

    #[validate(regex = "ENTITY_LABEL_REGEX")]
    #[validate(length(
        max = "DEFAULT_MAX_LENGTH",
        min = "DEFAULT_MIN_LENGTH",
        message = "The length of start_entity_type should be between 1 and 64."
    ))]
    pub start_entity_type: String,

    #[validate(regex = "ENTITY_LABEL_REGEX")]
    #[validate(length(
        max = "DEFAULT_MAX_LENGTH",
        min = "DEFAULT_MIN_LENGTH",
        message = "The length of end_entity_type should be between 1 and 64."
    ))]
    pub end_entity_type: String,
}

impl CheckData for RelationMetadata {
    fn check_csv_is_valid(filepath: &PathBuf) -> Vec<Box<dyn Error>> {
        Self::check_csv_is_valid_default::<RelationMetadata>(filepath)
    }

    fn unique_fields() -> Vec<String> {
        vec![
            "resource".to_string(),
            "relation_type".to_string(),
            "start_entity_type".to_string(),
            "end_entity_type".to_string(),
        ]
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
        let sql_str = "SELECT * FROM biomedgps_relation_metadata";
        let relation_metadata = sqlx::query_as::<_, RelationMetadata>(sql_str)
            .fetch_all(pool)
            .await?;

        AnyOk(relation_metadata)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Object, PartialEq, Eq)]
pub struct Payload {
    pub project_id: String,
    pub organization_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Object, sqlx::FromRow, Validate)]
pub struct KnowledgeCuration {
    // Ignore this field when deserialize from json
    #[serde(skip_deserializing)]
    #[oai(read_only)]
    pub id: i64,

    #[validate(length(
        max = "DEFAULT_MAX_LENGTH",
        min = "DEFAULT_MIN_LENGTH",
        message = "The length of resource must be between 1 and 64."
    ))]
    pub relation_type: String,

    #[validate(length(
        max = "ENTITY_NAME_MAX_LENGTH",
        min = "DEFAULT_MIN_LENGTH",
        message = "The length of source_name must be between 1 and 64."
    ))]
    pub source_name: String,

    #[validate(regex(
        path = "ENTITY_LABEL_REGEX",
        message = "The source_type must be a valid entity type. The regex pattern is `^[A-Za-z]+$`, such as `Gene`."
    ))]
    #[validate(length(
        max = "DEFAULT_MAX_LENGTH",
        min = "DEFAULT_MIN_LENGTH",
        message = "The length of source_type must be between 1 and 64."
    ))]
    pub source_type: String,

    #[validate(length(
        max = "DEFAULT_MAX_LENGTH",
        min = "DEFAULT_MIN_LENGTH",
        message = "The length of source_id must be between 1 and 64."
    ))]
    #[validate(regex(
        path = "ENTITY_ID_REGEX",
        message = "The source_id must match the pattern `^[A-Za-z0-9\\-]+:[a-z0-9A-Z\\.\\-_]+$`. Such as `UniProtKB:P12345`."
    ))]
    pub source_id: String,

    #[validate(length(
        max = "ENTITY_NAME_MAX_LENGTH",
        min = "DEFAULT_MIN_LENGTH",
        message = "The length of target_name must be between 1 and 64."
    ))]
    pub target_name: String,

    #[validate(regex(
        path = "ENTITY_LABEL_REGEX",
        message = "The target_type must be a valid entity label. The regex pattern is `^[A-Za-z]+$`, such as `Gene`."
    ))]
    #[validate(length(
        max = "DEFAULT_MAX_LENGTH",
        min = "DEFAULT_MIN_LENGTH",
        message = "The length of target_type must be between 1 and 64."
    ))]
    pub target_type: String,

    #[validate(length(
        max = "DEFAULT_MAX_LENGTH",
        min = "DEFAULT_MIN_LENGTH",
        message = "The length of target_id must be between 1 and 64."
    ))]
    #[validate(regex(
        path = "ENTITY_ID_REGEX",
        message = "The target_id must match the pattern `^[A-Za-z0-9\\-]+:[a-z0-9A-Z\\.\\-_]+$`. Such as `UniProtKB:P12345`."
    ))]
    pub target_id: String,

    pub key_sentence: String,

    #[serde(skip_deserializing)]
    #[serde(with = "ts_seconds")]
    #[oai(read_only)]
    pub created_at: DateTime<Utc>,

    #[validate(length(
        max = "DEFAULT_MAX_LENGTH",
        min = "DEFAULT_MIN_LENGTH",
        message = "The length of curator must be between 1 and 64."
    ))]
    pub curator: String,

    #[validate(range(min = 1, message = "pmid must be greater than 0"))]
    pub pmid: i64,

    // The payload field is a jsonb field which contains the project_id and organization_id.
    pub payload: Option<serde_json::Value>,
}

impl KnowledgeCuration {
    pub fn to_relation(&self) -> Relation {
        Relation {
            id: self.id,
            relation_type: self.relation_type.clone(),
            source_type: self.source_type.clone(),
            source_id: self.source_id.clone(),
            target_type: self.target_type.clone(),
            target_id: self.target_id.clone(),
            key_sentence: Some(self.key_sentence.clone()),
            resource: self.curator.clone(),
            pmids: Some(format!("{}", self.pmid)),
            score: None,
        }
    }

    pub async fn get_records(pool: &sqlx::PgPool) -> Result<Vec<KnowledgeCuration>, anyhow::Error> {
        let sql_str = "SELECT * FROM biomedgps_knowledge_curation";
        let records = sqlx::query_as::<_, KnowledgeCuration>(sql_str)
            .fetch_all(pool)
            .await?;

        AnyOk(records)
    }

    pub async fn get_records_by_owner(
        pool: &sqlx::PgPool,
        curator: &str,
        project_id: i32,
        organization_id: i32,
        page: Option<u64>,
        page_size: Option<u64>,
        order_by: Option<&str>,
    ) -> Result<RecordResponse<KnowledgeCuration>, anyhow::Error> {
        let project_id_qstr = if project_id >= 0 {
            format!("payload->>'project_id' = '{}'", project_id)
        } else {
            format!("payload->>'project_id' IS NOT NULL")
        };

        let organization_id_qstr = if organization_id >= 0 {
            format!("payload->>'organization_id' = '{}'", organization_id)
        } else {
            format!("payload->>'organization_id' IS NOT NULL")
        };

        let curator_qstr = if project_id < 0 && organization_id < 0 {
            format!("curator = '{}'", curator)
        } else {
            format!("curator IS NOT NULL")
        };

        let where_str = format!("{} AND {} AND {}", curator_qstr, project_id_qstr, organization_id_qstr);

        let page = match page {
            Some(page) => page,
            None => 1,
        };

        let page_size = match page_size {
            Some(page_size) => page_size,
            None => 10,
        };

        let limit = page_size;
        let offset = (page - 1) * page_size;

        let order_by_str = if order_by.is_none() {
            "".to_string()
        } else {
            format!("ORDER BY {}", order_by.unwrap())
        };

        let sql_str = format!(
            "SELECT * FROM biomedgps_knowledge_curation WHERE {} {} LIMIT {} OFFSET {}",
            where_str, order_by_str, limit, offset
        );

        let records = sqlx::query_as::<_, KnowledgeCuration>(sql_str.as_str())
            .fetch_all(pool)
            .await?;

        let sql_str = format!("SELECT COUNT(*) FROM biomedgps_knowledge_curation WHERE {}", where_str);

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

    fn get_value(key: &str, json: &serde_json::Value) -> Result<String, anyhow::Error> {
        match json[key].as_str() {
            Some(value) => Ok(value.to_string()),
            None => Err(anyhow::anyhow!("The {} field is missing.", key)),
        }
    }

    pub async fn insert(&self, pool: &sqlx::PgPool) -> Result<KnowledgeCuration, anyhow::Error> {
        let sql_str = "INSERT INTO biomedgps_knowledge_curation (relation_type, source_name, source_type, source_id, target_name, target_type, target_id, key_sentence, curator, pmid, payload) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11) RETURNING *";
        let payload = match &self.payload {
            Some(payload) => sqlx::types::Json(Payload {
                project_id: KnowledgeCuration::get_value("project_id", payload)?,
                organization_id: KnowledgeCuration::get_value("organization_id", payload)?,
            }),
            None => sqlx::types::Json(Payload {
                project_id: "0".to_string(),
                organization_id: "0".to_string(),
            }),
        };

        let knowledge_curation = sqlx::query_as::<_, KnowledgeCuration>(sql_str)
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
            .bind(&payload)
            .fetch_one(pool)
            .await?;

        AnyOk(knowledge_curation)
    }

    pub async fn update(
        &self,
        pool: &sqlx::PgPool,
        id: i64,
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

    pub async fn delete(pool: &sqlx::PgPool, id: i64) -> Result<KnowledgeCuration, anyhow::Error> {
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

    fn unique_fields() -> Vec<String> {
        vec![
            "relation_type".to_string(),
            "source_type".to_string(),
            "source_id".to_string(),
            "target_type".to_string(),
            "target_id".to_string(),
            "curator".to_string(),
            "pmid".to_string(),
        ]
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

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Object, sqlx::FromRow, Validate)]
pub struct Relation {
    // Ignore this field when deserialize from json
    #[serde(skip_deserializing)]
    #[oai(read_only)]
    pub id: i64,

    #[validate(length(
        max = "DEFAULT_MAX_LENGTH",
        min = "DEFAULT_MIN_LENGTH",
        message = "The length of relation_type must be between 1 and 64."
    ))]
    pub relation_type: String,

    #[validate(length(
        max = "DEFAULT_MAX_LENGTH",
        min = "DEFAULT_MIN_LENGTH",
        message = "The length of source_name must be between 1 and 64."
    ))]
    #[validate(regex(
        path = "ENTITY_ID_REGEX",
        message = "The source_id must match the ^[A-Za-z0-9\\-]+:[a-z0-9A-Z\\.\\-_]+$ pattern. eg: UniProtKB:P12345"
    ))]
    pub source_id: String,

    #[validate(length(
        max = "DEFAULT_MAX_LENGTH",
        min = "DEFAULT_MIN_LENGTH",
        message = "The length of source_name must be between 1 and 64."
    ))]
    #[validate(regex(
        path = "ENTITY_LABEL_REGEX",
        message = "The source_type must match the ^[A-Za-z]+$ pattern."
    ))]
    pub source_type: String,

    #[validate(length(
        max = "DEFAULT_MAX_LENGTH",
        min = "DEFAULT_MIN_LENGTH",
        message = "The length of source_name must be between 1 and 64."
    ))]
    #[validate(regex(
        path = "ENTITY_ID_REGEX",
        message = "The source_id must match the ^[A-Za-z0-9\\-]+:[a-z0-9A-Z\\.\\-_]+$ pattern. eg: UniProtKB:P12345"
    ))]
    pub target_id: String,

    #[validate(length(
        max = "DEFAULT_MAX_LENGTH",
        min = "DEFAULT_MIN_LENGTH",
        message = "The length of source_name must be between 1 and 64."
    ))]
    #[validate(regex(
        path = "ENTITY_LABEL_REGEX",
        message = "The target_type must match the ^[A-Za-z]+$ pattern."
    ))]
    pub target_type: String,

    #[oai(skip_serializing_if_is_none)]
    pub score: Option<f64>,

    #[oai(skip_serializing_if_is_none)]
    pub key_sentence: Option<String>,

    pub resource: String,

    #[oai(skip_serializing_if_is_none)]
    pub pmids: Option<String>,
}

impl CheckData for Relation {
    fn check_csv_is_valid(filepath: &PathBuf) -> Vec<Box<dyn Error>> {
        Self::check_csv_is_valid_default::<Relation>(filepath)
    }

    fn unique_fields() -> Vec<String> {
        vec![
            "relation_type".to_string(),
            "source_id".to_string(),
            "source_type".to_string(),
            "target_id".to_string(),
            "target_type".to_string(),
        ]
    }

    fn fields() -> Vec<String> {
        vec![
            "relation_type".to_string(),
            "source_id".to_string(),
            "source_type".to_string(),
            "target_id".to_string(),
            "target_type".to_string(),
            "score".to_string(),
            "key_sentence".to_string(),
            "resource".to_string(),
            "pmids".to_string(),
        ]
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Object, sqlx::FromRow, Validate)]
pub struct RelationCount {
    #[validate(length(
        max = "DEFAULT_MAX_LENGTH",
        min = "DEFAULT_MIN_LENGTH",
        message = "The length of relation_type must be between 1 and 64."
    ))]
    pub relation_type: String,

    #[validate(length(
        max = "DEFAULT_MAX_LENGTH",
        min = "DEFAULT_MIN_LENGTH",
        message = "The length of source_name must be between 1 and 64."
    ))]
    #[validate(regex(
        path = "ENTITY_LABEL_REGEX",
        message = "The target_type must match the ^[A-Za-z]+$ pattern."
    ))]
    pub target_type: String,

    #[validate(length(
        max = "DEFAULT_MAX_LENGTH",
        min = "DEFAULT_MIN_LENGTH",
        message = "The length of source_name must be between 1 and 64."
    ))]
    #[validate(regex(
        path = "ENTITY_LABEL_REGEX",
        message = "The source_type must match the ^[A-Za-z]+$ pattern."
    ))]
    pub source_type: String,

    pub resource: String,

    pub ncount: i64,
}

impl RelationCount {
    pub async fn get_records(
        pool: &sqlx::PgPool,
        query: &Option<ComposeQuery>,
    ) -> Result<Vec<RelationCount>, anyhow::Error> {
        let mut query_str = match query {
            Some(ComposeQuery::QueryItem(item)) => item.format(),
            Some(ComposeQuery::ComposeQueryItem(item)) => item.format(),
            None => "".to_string(),
        };

        if query_str.is_empty() {
            query_str = "1=1".to_string();
        };

        let sql_str = format!(
            "SELECT relation_type, source_type, target_type, resource, COUNT(*) as ncount FROM biomedgps_relation WHERE {} GROUP BY relation_type, source_type, target_type, resource",
            query_str
        );

        let records = sqlx::query_as::<_, RelationCount>(sql_str.as_str())
            .fetch_all(pool)
            .await?;

        AnyOk(records)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Object, sqlx::FromRow, Validate)]
pub struct Entity2D {
    pub embedding_id: i64,

    #[validate(length(
        max = "DEFAULT_MAX_LENGTH",
        min = "DEFAULT_MIN_LENGTH",
        message = "The length of entity_id must be between 1 and 64."
    ))]
    #[validate(regex(
        path = "ENTITY_ID_REGEX",
        message = "The entity_id must match the ^[A-Za-z0-9\\-]+:[a-z0-9A-Z\\.\\-_]+$ pattern. eg: UniProtKB:P12345"
    ))]
    pub entity_id: String,

    #[validate(length(
        max = "DEFAULT_MAX_LENGTH",
        min = "DEFAULT_MIN_LENGTH",
        message = "The length of entity_type must be between 1 and 64."
    ))]
    #[validate(regex(
        path = "ENTITY_LABEL_REGEX",
        message = "The entity_type must match the ^[A-Za-z]+$ pattern."
    ))]
    pub entity_type: String,

    #[validate(length(
        max = "ENTITY_NAME_MAX_LENGTH",
        min = "DEFAULT_MIN_LENGTH",
        message = "The length of entity_name must be between 1 and 255."
    ))]
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

    fn unique_fields() -> Vec<String> {
        vec![
            "embedding_id".to_string(),
            "entity_id".to_string(),
            "entity_type".to_string(),
        ]
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
    pub id: String,

    #[validate(length(
        max = "DEFAULT_MAX_LENGTH",
        min = "DEFAULT_MIN_LENGTH",
        message = "The length of name must be between 1 and 64."
    ))]
    pub name: String,

    #[oai(skip_serializing_if_is_none)]
    pub description: Option<String>,

    #[validate(regex(
        path = "JSON_REGEX",
        message = "The payload must be a valid json string."
    ))]
    pub payload: String, // json string, e.g. {"nodes": [], "edges": []}. how to validate json string?

    #[serde(skip_deserializing)]
    #[serde(with = "ts_seconds")]
    #[oai(read_only)]
    pub created_time: DateTime<Utc>,

    #[validate(length(
        min = 1,
        max = 36,
        message = "The owner length should be between 1 and 36"
    ))]
    pub owner: String,

    #[validate(length(
        min = 1,
        max = 36,
        message = "The version length should be between 1 and 36"
    ))]
    pub version: String,

    #[validate(length(
        min = 1,
        max = 36,
        message = "The db_version length should be between 1 and 36"
    ))]
    pub db_version: String,

    #[validate(regex(
        path = "SUBGRAPH_UUID_REGEX",
        message = "The parent must match the ^[a-f0-9]{8}-[a-f0-9]{4}-[a-f0-9]{4}-[a-f0-9]{4}-[a-f0-9]{12}$ pattern."
    ))]
    pub parent: Option<String>, // parent subgraph id, it is same as id if it is a root subgraph (no parent), otherwise it is the parent subgraph id
}

impl CheckData for Subgraph {
    fn check_csv_is_valid(filepath: &PathBuf) -> Vec<Box<dyn Error>> {
        Self::check_csv_is_valid_default::<Subgraph>(filepath)
    }

    fn unique_fields() -> Vec<String> {
        vec![
            "id".to_string(),
            "owner".to_string(),
            "version".to_string(),
            "db_version".to_string(),
            "parent".to_string(),
        ]
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

        let sql_str = "INSERT INTO biomedgps_subgraph (id, name, description, payload, owner, version, db_version, parent) VALUES ($1, $2, $3, $4, $5, $6, $7, $8) RETURNING *";
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
