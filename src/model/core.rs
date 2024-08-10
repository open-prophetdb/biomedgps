//! The database schema for the application. These are the models that will be used to interact with the database.

use super::graph::COMPOSED_ENTITY_DELIMITER;
use super::kge::get_entity_emb_table_name;
use super::util::{get_delimiter, parse_csv_error, ValidationError};
use std::collections::HashMap;
// use crate::model::util::match_color;
use crate::query_builder::sql_builder::ComposeQuery;
use anyhow::Ok as AnyOk;
use chrono::serde::ts_seconds;
use chrono::{DateTime, Utc};
use lazy_static::lazy_static;
use log::{debug, info};
use poem_openapi::Object;
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::{error::Error, option::Option, path::PathBuf};
use validator::Validate;

pub const DEFAULT_DATASET_NAME: &str = "biomedgps";
pub const ENTITY_NAME_MAX_LENGTH: u64 = 255;
pub const RELATION_ID_MAX_LENGTH: u64 = 255;
pub const DEFAULT_MAX_LENGTH: u64 = 64;
pub const DEFAULT_MIN_LENGTH: u64 = 1;

lazy_static! {
    // The relation_id is like "<RELATION_TYPE>|<SOURCE_ID>|<TARGET_ID>", e.g. "STRING::ACTIVATOR::Gene:Compound|Gene::ENTREZ:1017|Compound::DrugBank:2083"
    pub static ref RELATION_ID_REGEX: Regex = Regex::new(r"^[a-zA-Z0-9_\-]+::[a-zA-Z0-9 _\-]+::[a-zA-Z]+:[a-zA-Z]+|[a-zA-Z]+::[A-Za-z0-9\-]+:[a-z0-9A-Z\.\-_]+|[a-zA-Z]+::[A-Za-z0-9\-]+:[a-z0-9A-Z\.\-_]+$").unwrap();
    pub static ref ENTITY_LABEL_REGEX: Regex = Regex::new(r"^[A-Za-z]+$").unwrap();
    pub static ref ENTITY_ID_REGEX: Regex = Regex::new(r"^[A-Za-z0-9\-]+:[a-z0-9A-Z\.\-_]+$").unwrap();
    // 1.23|-4.56|7.89
    pub static ref EMBEDDING_REGEX: Regex = Regex::new(r"^(?:-?\d+(?:\.\d+)?\|)*-?\d+(?:\.\d+)?$").unwrap();
    pub static ref SUBGRAPH_UUID_REGEX: Regex = Regex::new(r"^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$").unwrap();
    pub static ref JSON_REGEX: Regex = Regex::new(r"^(\{.*\}|\[.*\])$").expect("Failed to compile regex");
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
                validation_errors.push(Box::new(ValidationError::new(
                    &format!("Failed to get delimiter: ({})", e),
                    vec![],
                )));
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
                validation_errors.push(Box::new(ValidationError::new(
                    &format!("Failed to read CSV: ({})", e),
                    vec![],
                )));
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
                        validation_errors.push(Box::new(ValidationError::new(
                            &format!(
                                "Failed to validate the data, line: {}, details: ({})",
                                line_number, e
                            ),
                            vec![],
                        )));
                        continue;
                    }
                },
                Err(e) => {
                    let error_msg = parse_csv_error(&e);

                    validation_errors.push(Box::new(ValidationError::new(&error_msg, vec![])));

                    continue;
                }
            };
        }

        validation_errors
    }

    fn fields() -> Vec<String>;

    fn unique_fields() -> Vec<String>;

    fn get_error_msg<S: for<'de> serde::Deserialize<'de> + Validate + std::fmt::Debug>(
        r: Result<Vec<S>, Box<dyn Error>>,
    ) -> String {
        match r {
            Ok(_) => "".to_string(),
            Err(e) => {
                return e.to_string();
            }
        }
    }

    /// Select the columns to keep
    /// Return the path of the output file which is a temporary file
    fn select_expected_columns<S: for<'de> serde::Deserialize<'de> + Validate + std::fmt::Debug>(
        in_filepath: &PathBuf,
        out_filepath: &PathBuf,
    ) -> Result<Vec<S>, Box<dyn Error>> {
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

        // TODO: Poor performance, need to optimize?
        Ok(Self::get_records(out_filepath)?) // Return the records of the output file
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

    fn get_records<S: for<'de> serde::Deserialize<'de> + Validate + std::fmt::Debug>(
        filepath: &PathBuf,
    ) -> Result<Vec<S>, Box<dyn Error>> {
        debug!("Start to get records from the csv file: {:?}", filepath);
        let delimiter = get_delimiter(filepath)?;
        let mut reader = csv::ReaderBuilder::new()
            .delimiter(delimiter)
            .from_path(filepath)?;

        let mut records = Vec::new();
        for result in reader.deserialize::<S>() {
            let record: S = result?;
            records.push(record);
        }

        debug!("Get {} records successfully.", records.len());

        Ok(records)
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
        owner: Option<&str>,
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

        let which_owner = if owner.is_some() {
            format!("AND owner = '{}'", owner.unwrap())
        } else {
            "".to_string()
        };

        let query_str = format!("{} {}", query_str, which_owner);

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
    #[oai(skip)]
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

impl Entity {
    /// Get the valid records in both entity and entity embedding tables
    ///
    /// # Arguments
    /// * `pool` - The database connection pool
    /// * `table_prefix` - The prefix of the entity embedding table name, such as "biomedgps"
    /// * `query` - The query condition
    /// * `page` - The page index
    /// * `page_size` - The page size
    /// * `order_by` - The order by condition
    ///
    /// # Returns
    /// * `Result<RecordResponse<Entity>, anyhow::Error>` - The valid records or an error
    pub async fn get_valid_records(
        pool: &sqlx::PgPool,
        table_prefix: &str,
        query: &Option<ComposeQuery>,
        page: Option<u64>,
        page_size: Option<u64>,
        order_by: Option<&str>,
    ) -> Result<RecordResponse<Entity>, anyhow::Error> {
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

        let joined_table_name = get_entity_emb_table_name(table_prefix);
        let sql_str = format!(
            "SELECT * FROM biomedgps_entity RIGHT JOIN {joined_table_name} ON biomedgps_entity.id = {joined_table_name}.entity_id AND biomedgps_entity.label = {joined_table_name}.entity_type WHERE {query_str} {order_by_str} {pagination_str}",
            joined_table_name = joined_table_name, 
            query_str = query_str, 
            order_by_str = order_by_str, 
            pagination_str = pagination_str
        );

        let records = sqlx::query_as::<_, Entity>(sql_str.as_str())
            .fetch_all(pool)
            .await?;

        let sql_str = format!(
            "SELECT COUNT(*) FROM biomedgps_entity RIGHT JOIN {joined_table_name} ON biomedgps_entity.id = {joined_table_name}.entity_id AND biomedgps_entity.label = {joined_table_name}.entity_type WHERE {query_str}",
            joined_table_name = joined_table_name, query_str = query_str
        );

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

// For importing attributes of entities into the graph database.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Object, sqlx::FromRow, Validate)]
pub struct EntityAttribute {
    // Ignore this field when deserialize from json
    #[serde(skip_deserializing)]
    #[oai(read_only)]
    pub idx: i64,

    #[validate(length(
        max = "DEFAULT_MAX_LENGTH",
        min = "DEFAULT_MIN_LENGTH",
        message = "The length of entity_id should be between 1 and 64."
    ))]
    #[validate(regex(
        path = "ENTITY_ID_REGEX",
        message = "The entity id is invalid. It should match ^[A-Za-z0-9\\-]+:[a-z0-9A-Z\\.\\-_]+$. Such as 'MESH:D000001'."
    ))]
    pub entity_id: String,

    #[validate(length(
        max = "DEFAULT_MAX_LENGTH",
        min = "DEFAULT_MIN_LENGTH",
        message = "The length of entity_type should be between 1 and 64."
    ))]
    #[validate(regex = "ENTITY_LABEL_REGEX")]
    pub entity_type: String,

    // A human-readable summary of the entity in an external database.
    pub description: String,

    // The name of an external database. such as MESH, OMIM, etc. Also, we can develop a integrated database for collecting information for each entity. It could be a external service. We can call it as 'biomedgps-metadata-service'. So, we can extract all related information into the external service, and then we can get the information from the external service by the entity_id and entity_type.
    #[validate(length(
        max = "DEFAULT_MAX_LENGTH",
        min = "DEFAULT_MIN_LENGTH",
        message = "The length of external_db_name should be between 1 and 64."
    ))]
    pub external_db_name: String,

    // The link to the entity in an external database. It should be a valid URL, like https://www.ncbi.nlm.nih.gov/mesh/68000001.
    pub external_url: String,

    // The id of the entity in an external database.
    #[validate(length(
        max = "DEFAULT_MAX_LENGTH",
        min = "DEFAULT_MIN_LENGTH",
        message = "The length of external_id should be between 1 and 64."
    ))]
    pub external_id: String,
}

impl CheckData for EntityAttribute {
    fn check_csv_is_valid(filepath: &PathBuf) -> Vec<Box<dyn Error>> {
        Self::check_csv_is_valid_default::<EntityAttribute>(filepath)
    }

    fn unique_fields() -> Vec<String> {
        vec![
            "entity_id".to_string(),
            "entity_type".to_string(),
            "external_db_name".to_string(),
            "external_id".to_string(),
        ]
    }

    fn fields() -> Vec<String> {
        vec![
            "entity_id".to_string(),
            "entity_type".to_string(),
            "description".to_string(),
            "external_db_name".to_string(),
            "external_url".to_string(),
            "external_id".to_string(),
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

    // The resource of the relation. such as STRING, BIOMEDGPS, etc.
    #[validate(length(
        max = "DEFAULT_MAX_LENGTH",
        min = "DEFAULT_MIN_LENGTH",
        message = "The length of resource should be between 1 and 64."
    ))]
    pub resource: String,

    // The dataset of the relation for labeling different datasets. such DRKG, HSDN, CTD, CuratedFindings, etc. Users can choose different dataset combinations to build a knowledge graph and train models.
    #[validate(length(
        max = "DEFAULT_MAX_LENGTH",
        min = "DEFAULT_MIN_LENGTH",
        message = "The length of dataset should be between 1 and 64."
    ))]
    pub dataset: String,

    // The relation type, such as STRING::ACTIVATOR::Gene:Compound, STRING::INHIBITOR::Gene:Compound, etc.
    #[validate(length(
        max = "DEFAULT_MAX_LENGTH",
        min = "DEFAULT_MIN_LENGTH",
        message = "The length of relation_type should be between 1 and 64."
    ))]
    pub relation_type: String,

    // The formatted relation type, such as BIOMEDGPS::ACTIVATOR::Gene::Compound, BIOMEDGPS::TREATMENT::Compound::Disease, etc.
    #[validate(length(
        max = "DEFAULT_MAX_LENGTH",
        min = "DEFAULT_MIN_LENGTH",
        message = "The length of formatted_relation_type should be between 1 and 64."
    ))]
    pub formatted_relation_type: String,

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

    // To describe the relation type with a human-readable sentence.
    #[oai(skip_serializing_if_is_none)]
    pub description: Option<String>,

    // Prompt Template
    #[oai(skip_serializing_if_is_none)]
    pub prompt_template: Option<String>,
}

impl CheckData for RelationMetadata {
    fn check_csv_is_valid(filepath: &PathBuf) -> Vec<Box<dyn Error>> {
        Self::check_csv_is_valid_default::<RelationMetadata>(filepath)
    }

    fn unique_fields() -> Vec<String> {
        vec![
            "dataset".to_string(),
            "resource".to_string(),
            "formatted_relation_type".to_string(),
            "relation_type".to_string(),
            "start_entity_type".to_string(),
            "end_entity_type".to_string(),
        ]
    }

    fn fields() -> Vec<String> {
        vec![
            "dataset".to_string(),
            "resource".to_string(),
            "relation_type".to_string(),
            "formatted_relation_type".to_string(),
            "relation_count".to_string(),
            "start_entity_type".to_string(),
            "end_entity_type".to_string(),
            "description".to_string(),
            "prompt_template".to_string(),
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
            formatted_relation_type: Some(self.relation_type.clone()),
            source_type: self.source_type.clone(),
            source_id: self.source_id.clone(),
            target_type: self.target_type.clone(),
            target_id: self.target_id.clone(),
            key_sentence: Some(self.key_sentence.clone()),
            resource: self.curator.clone(),
            dataset: Some(DEFAULT_DATASET_NAME.to_string()),
            pmids: Some(format!("{}", self.pmid)),
            score: None,
        }
    }

    pub async fn get_records(pool: &sqlx::PgPool) -> Result<Vec<KnowledgeCuration>, anyhow::Error> {
        let columns = <KnowledgeCuration as CheckData>::fields().join(",");
        let sql_str =
            format!("SELECT id,created_at,payload,{columns} FROM biomedgps_knowledge_curation");
        let records = sqlx::query_as::<_, KnowledgeCuration>(sql_str.as_str())
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

        let where_str = format!(
            "{} AND {} AND {}",
            curator_qstr, project_id_qstr, organization_id_qstr
        );

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

        let sql_str = format!(
            "SELECT COUNT(*) FROM biomedgps_knowledge_curation WHERE {}",
            where_str
        );

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
    #[oai(skip)]
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
        message = "The length of formatted_relation_type must be between 1 and 64."
    ))]
    pub formatted_relation_type: Option<String>,

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
    pub dataset: Option<String>,

    #[oai(skip_serializing_if_is_none)]
    pub pmids: Option<String>,
}

impl Relation {
    pub fn gen_composed_key(first_node_id: &str, second_node_id: &str) -> String {
        if first_node_id < second_node_id {
            format!(
                "{}{}{}",
                first_node_id, COMPOSED_ENTITY_DELIMITER, second_node_id
            )
        } else {
            format!(
                "{}{}{}",
                second_node_id, COMPOSED_ENTITY_DELIMITER, first_node_id
            )
        }
    }

    pub async fn exist_records(
        pool: &sqlx::PgPool,
        node_id: &str,              // for example, "Gene::ENTREZ:123"
        other_node_ids: &Vec<&str>, // for example, ["Gene::ENTREZ:123", "Compound::DrugBank:DB00001"]
        relation_type: Option<&str>,
        ignore_direction: bool,
    ) -> Result<HashMap<String, Relation>, anyhow::Error> {
        let node_id = node_id.split(",").collect::<Vec<&str>>().join("', '");
        let other_node_ids_str = other_node_ids
            .iter()
            .map(|x| format!("'{}'", x))
            .collect::<Vec<String>>()
            .join(",");
        let where_clauses = if ignore_direction {
            format!(
                "
                SELECT *
                FROM biomedgps_relation
                WHERE (
                    (
                        CONCAT(source_type, '{delimiter}', source_id) in ('{node_id}') AND 
                        CONCAT(target_type, '{delimiter}', target_id) in ({other_node_ids_str})
                    ) OR 
                    (
                        CONCAT(source_type, '{delimiter}', source_id) in ({other_node_ids_str}) AND 
                        CONCAT(target_type, '{delimiter}', target_id) in ('{node_id}')
                    )
                )
            ",
                delimiter = COMPOSED_ENTITY_DELIMITER,
                node_id = node_id,
                other_node_ids_str = other_node_ids_str
            )
        } else {
            format!(
                "
                SELECT *
                FROM biomedgps_relation
                WHERE (
                    CONCAT(source_type, '{delimiter}', source_id) in ({node_id}) AND 
                    CONCAT(target_type, '{delimiter}', target_id) in ({other_node_ids_str})
                )
            ",
                delimiter = COMPOSED_ENTITY_DELIMITER,
                node_id = node_id,
                other_node_ids_str = other_node_ids_str
            )
        };

        let sql_str = match relation_type {
            Some(relation_type) => {
                format!("{} AND relation_type = '{}'", where_clauses, relation_type)
            }
            None => where_clauses,
        };

        let records = sqlx::query_as::<_, Relation>(sql_str.as_str())
            .fetch_all(pool)
            .await?;

        let mut relation_map = HashMap::new();
        for record in records {
            let source_node_id = format!(
                "{}{}{}",
                record.source_type, COMPOSED_ENTITY_DELIMITER, record.source_id
            );
            let target_node_id = format!(
                "{}{}{}",
                record.target_type, COMPOSED_ENTITY_DELIMITER, record.target_id
            );
            let ordered_key_str = Self::gen_composed_key(&source_node_id, &target_node_id);
            relation_map.insert(ordered_key_str, record);
        }

        AnyOk(relation_map)
    }
}

impl CheckData for Relation {
    fn check_csv_is_valid(filepath: &PathBuf) -> Vec<Box<dyn Error>> {
        Self::check_csv_is_valid_default::<Relation>(filepath)
    }

    fn unique_fields() -> Vec<String> {
        vec![
            "resource".to_string(),
            "dataset".to_string(),
            "formatted_relation_type".to_string(),
            "relation_type".to_string(),
            "source_id".to_string(),
            "source_type".to_string(),
            "target_id".to_string(),
            "target_type".to_string(),
        ]
    }

    fn fields() -> Vec<String> {
        vec![
            "formatted_relation_type".to_string(),
            "relation_type".to_string(),
            "source_id".to_string(),
            "source_type".to_string(),
            "target_id".to_string(),
            "target_type".to_string(),
            "score".to_string(),
            "key_sentence".to_string(),
            "resource".to_string(),
            "dataset".to_string(),
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

    // It should be a valid json string. e.g. {"data": {"nodes": [], "edges": []}, "layout": {}, "defaultLayout": {}, llm": [{"prompt": "", "response": ""}]}. It might contain the data, layout, defaultLayout, and llm fields. The llm field is used to store the prompt and response which are generated by the chatgpt for explaining the subgraph.
    // TODO: how to validate json string?
    #[validate(regex(
        path = "JSON_REGEX",
        message = "The payload must be a valid json string."
    ))]
    pub payload: String,

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
    pub fn update_owner(&mut self, username: String) -> &Self {
        self.owner = username;
        return self;
    }

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
