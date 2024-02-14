use super::core::{
    CheckData, ValidationError, DEFAULT_DATASET_NAME, DEFAULT_MAX_LENGTH, DEFAULT_MIN_LENGTH,
    ENTITY_ID_REGEX, ENTITY_LABEL_REGEX, ENTITY_NAME_MAX_LENGTH,
};
use super::util::{drop_table, get_delimiter, parse_csv_error, read_annotation_file};
use crate::pgvector::Vector;
use crate::query_builder::sql_builder::{ComposeQuery, QueryItem};
use anyhow::Ok as AnyOk;
use chrono::serde::ts_seconds;
use chrono::{DateTime, Utc};
use lazy_static::lazy_static;
use log::{debug, info, warn};
use poem_openapi::Object;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::collections::HashMap;
use std::error::Error;
use std::path::PathBuf;
use std::sync::Mutex;
use validator::Validate;

pub const DEFAULT_MODEL_NAME: &str = "biomedgps";

pub const DEFAULT_MODEL_TYPES: [&str; 8] = [
    "TransE_l2",
    "TransE_l1",
    "TransH",
    "TransR",
    "TransD",
    "RotatE",
    "DistMult",
    "ComplEx",
];

lazy_static! {
    static ref KGE_MODELS: Mutex<HashMap<String, EmbeddingMetadata>> = Mutex::new(HashMap::new());
}

async fn check_table_is_valid(
    pool: &sqlx::PgPool,
    table_names: &Vec<&str>,
) -> Result<(), ValidationError> {
    // Get all table names from database
    let sql_str = "SELECT table_name FROM information_schema.tables WHERE table_schema = 'public'";

    let records = sqlx::query_as::<_, (String,)>(sql_str)
        .fetch_all(pool)
        .await
        .unwrap();

    let all_table_names: Vec<String> = records.iter().map(|r| r.0.clone()).collect();

    for table_name in table_names {
        if !all_table_names.contains(&table_name.to_string()) {
            return Err(ValidationError::new(
                "The table name does not exist in the database.",
            ));
        }
    }

    Ok(())
}

fn get_entity_emb_table_name(table_name: &str) -> String {
    format!("{}_entity_embedding", table_name)
}

fn get_relation_emb_table_name(table_name: &str) -> String {
    format!("{}_relation_embedding", table_name)
}

pub async fn check_default_model_is_valid(pool: &sqlx::PgPool) -> Result<(), ValidationError> {
    let table_names = vec![
        get_entity_emb_table_name(DEFAULT_MODEL_NAME),
        get_relation_emb_table_name(DEFAULT_MODEL_NAME),
    ];

    let table_names = table_names
        .iter()
        .map(|s| s.as_str())
        .collect::<Vec<&str>>();

    match check_table_is_valid(pool, &table_names).await {
        Ok(_) => {
            let sql_str = "SELECT * FROM biomedgps_embedding_metadata WHERE table_name = $1 AND model_name = $2";

            let count = match sqlx::query_as::<_, (i64,)>(&sql_str)
                .bind(DEFAULT_MODEL_NAME)
                .bind(DEFAULT_MODEL_NAME)
                .fetch_one(pool)
                .await
            {
                Ok(count) => count,
                Err(e) => {
                    return Err(ValidationError::new(&format!(
                        "The default model does not exist in the database: {}",
                        e.to_string()
                    )));
                }
            };

            if count.0 == 0 {
                return Err(ValidationError::new(
                    "The default model does not exist in the database.",
                ));
            } else {
                return Ok(());
            }
        }
        Err(e) => {
            return Err(e);
        }
    }
}

pub async fn add_default_model(pool: &sqlx::PgPool) -> Result<(), anyhow::Error> {
    match check_default_model_is_valid(pool).await {
        Ok(_) => {
            info!("The default model of BiomedKG already exists.");
            return Ok(());
        }
        Err(_) => {
            warn!("WARNING: The default model of BiomedKG does not exist, we will create it automatically. You may forget to create the default model.");
            let mut kge_models = KGE_MODELS.lock().unwrap();
            let metadata = EmbeddingMetadata {
                id: 0,
                table_name: DEFAULT_MODEL_NAME.to_string(),
                model_name: DEFAULT_MODEL_NAME.to_string(),
                model_type: "TransE".to_string(),
                description: "The default model of BiomedKG".to_string(),
                datasets: vec![DEFAULT_DATASET_NAME.to_string()],
                created_at: Utc::now(),
                dimension: 400,
                metadata: None,
            };
            match &metadata.insert(pool).await {
                Ok(_) => {
                    info!("The default model of BiomedKG has been created successfully.");
                }
                Err(e) => {
                    warn!("WARNING: The default model of BiomedKG has not been created successfully: {}", e.to_string());
                    let msg = format!(
                        "The default model of BiomedKG has not been created successfully: {}",
                        e.to_string()
                    );
                    return Err(anyhow::Error::msg(msg));
                }
            };
            kge_models.insert(DEFAULT_MODEL_NAME.to_string(), metadata);
            return Ok(());
        }
    }
}

/// Initialize the kge models, it should be called when the server starts.
pub async fn init_kge_models(
    pool: &sqlx::PgPool,
) -> Result<HashMap<String, EmbeddingMetadata>, anyhow::Error> {
    add_default_model(pool).await;

    let mut kge_models = KGE_MODELS.lock().unwrap();

    if kge_models.is_empty() {
        let sql_str = "SELECT * FROM biomedgps_embedding_metadata";

        let records = sqlx::query_as::<_, EmbeddingMetadata>(sql_str)
            .fetch_all(pool)
            .await?;

        for record in records {
            let model_name = record.model_name.clone();
            let table_name = record.table_name.clone();
            let entity_emb_table_name = get_entity_emb_table_name(&table_name);
            let relation_emb_table_name = get_relation_emb_table_name(&table_name);
            match check_table_is_valid(
                pool,
                &vec![&entity_emb_table_name, &relation_emb_table_name],
            )
            .await
            {
                Ok(_) => {
                    // We can fetch the embedding metadata easily if we use the model name and table name as the key. We don't need to care about whether the table name is same as the model name.
                    kge_models.insert(table_name.clone(), record.clone());
                    kge_models.insert(model_name.clone(), record.clone());
                    info!(
                        "Load the embedding metadata of model {} successfully.",
                        model_name
                    );
                }
                Err(e) => {
                    return Err(anyhow::Error::new(e));
                }
            }
        }
    }

    Ok(kge_models.clone())
}

pub fn get_embedding_metadata(key: &str) -> Option<EmbeddingMetadata> {
    let kge_models = KGE_MODELS.lock().unwrap();

    match kge_models.get(key) {
        Some(metadata) => Some(metadata.clone()),
        None => None,
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

/// A struct for embedding metadata, it is used for recording the metadata of embedding.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, sqlx::FromRow, Object, Validate)]
pub struct EmbeddingMetadata {
    // Ignore this field when deserialize from json
    #[serde(skip_deserializing)]
    #[oai(read_only)]
    pub id: i64,

    #[validate(length(
        max = "DEFAULT_MAX_LENGTH",
        min = "DEFAULT_MIN_LENGTH",
        message = "The length of table_name should be between 1 and 64."
    ))]
    pub table_name: String,

    #[validate(length(
        max = "DEFAULT_MAX_LENGTH",
        min = "DEFAULT_MIN_LENGTH",
        message = "The length of model_name should be between 1 and 64, such as `transe_mecfs`"
    ))]
    pub model_name: String,

    #[validate(length(
        max = "DEFAULT_MAX_LENGTH",
        min = "DEFAULT_MIN_LENGTH",
        message = "The length of model_type should be between 1 and 64, such as TransE, TransH, TransR, TransD, RotatE, etc."
    ))]
    pub model_type: String,

    pub description: String,

    pub datasets: Vec<String>, // Dataset name, such as hsdn, drkg, ctdbase, etc.

    #[serde(skip_deserializing)]
    #[serde(with = "ts_seconds")]
    #[oai(read_only)]
    pub created_at: DateTime<Utc>,

    pub dimension: i32, // The dimension of embedding, such as 400, 768, 1024, etc.

    pub metadata: Option<String>, // The metadata of embedding, such as hyperparameters, etc.
}

impl EmbeddingMetadata {
    async fn create_entity_emb_table(
        tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
        table_name: &str,
        dimension: usize,
    ) -> Result<(), Box<dyn Error>> {
        let real_table_name = get_entity_emb_table_name(table_name);

        let sql_str = format!("
            CREATE TABLE
            IF NOT EXISTS {} (
                embedding_id BIGINT PRIMARY KEY, -- The embedding ID
                entity_id VARCHAR(64) NOT NULL, -- The entity ID
                entity_type VARCHAR(64) NOT NULL, -- The entity type, such as Anatomy, Disease, Gene, Compound, Biological Process, etc.
                entity_name VARCHAR(255) NOT NULL, -- The entity name
                embedding vector({}), -- The embedding array, the length of the embedding array is {}. It is related with the knowledge graph model, such as TransE, DistMult, etc.
                CONSTRAINT {}_uniq_key UNIQUE (entity_id, entity_type)
            );
        ", &real_table_name, dimension, dimension, &real_table_name);

        match sqlx::query(&sql_str).execute(tx).await {
            Ok(_) => {
                return Ok(());
            }
            Err(e) => {
                return Err(Box::new(e));
            }
        };
    }

    async fn create_relation_emb_table(
        tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
        table_name: &str,
        dimension: usize,
    ) -> Result<(), Box<dyn Error>> {
        let real_table_name = get_relation_emb_table_name(table_name);

        let sql_str = format!("
            CREATE TABLE
            IF NOT EXISTS {} (
                embedding_id BIGINT PRIMARY KEY, -- The embedding ID
                relation_type VARCHAR(64) NOT NULL, -- The relation type, such as ACTIVATOR::Gene:Compound, INHIBITOR::Gene:Compound, etc.
                formatted_relation_type VARCHAR(64) NOT NULL, -- The formatted relation type, such as ACTIVATOR_Gene_Compound, INHIBITOR_Gene_Compound, etc.
                embedding vector({}), -- The embedding array, the length of the embedding array is {}. It is related with the knowledge graph model, such as TransE, DistMult, etc.
                UNIQUE (
                    formatted_relation_type,
                    relation_type
                )
            );
        ", &real_table_name, dimension, dimension);

        match sqlx::query(&sql_str).execute(tx).await {
            Ok(_) => {
                return Ok(());
            }
            Err(e) => {
                return Err(Box::new(e));
            }
        };
    }

    /// Insert a record into the embedding metadata table. If the table name and model name already exists, it will return an error and rollback the transaction.
    pub async fn insert(&self, pool: &sqlx::PgPool) -> Result<EmbeddingMetadata, Box<dyn Error>> {
        return EmbeddingMetadata::init_embedding_table(
            pool,
            &self.table_name,
            &self.model_name,
            &self.model_type,
            &self.description,
            &self
                .datasets
                .iter()
                .map(|s| s.as_str())
                .collect::<Vec<&str>>(),
            self.dimension as usize,
            self.metadata.clone(),
        )
        .await;
    }

    /// Initialize the embedding tables, it will create the entity embedding table, relation embedding table and insert a record into the embedding metadata table. If the table name and model name already exists or it can not create the entity embedding table or relation embedding table, it will return an error and rollback the transaction.
    ///
    /// # Arguments
    /// * `pool` - The database connection pool.
    /// * `table_name` - The table name of embedding metadata.
    /// * `model_name` - The model name of embedding metadata.
    /// * `model_type` - The model type of embedding metadata.
    /// * `description` - The description of embedding metadata.
    /// * `datasets` - The datasets of embedding metadata.
    /// * `dimension` - The dimension of embedding metadata.
    /// * `metadata` - The metadata of embedding metadata.
    ///
    /// # Returns
    /// * `Result<EmbeddingMetadata, Box<dyn Error>>` - The embedding metadata.
    pub async fn init_embedding_table(
        pool: &sqlx::PgPool,
        table_name: &str,
        model_name: &str,
        model_type: &str,
        description: &str,
        datasets: &Vec<&str>,
        dimension: usize,
        metadata: Option<String>,
    ) -> Result<EmbeddingMetadata, Box<dyn Error>> {
        // Begin to transaction
        let mut tx = pool.begin().await?;

        let sql_str = "SELECT COUNT(*) FROM biomedgps_embedding_metadata WHERE table_name = $1 AND model_name = $2";

        let count = sqlx::query_as::<_, (i64,)>(&sql_str)
            .bind(table_name)
            .bind(model_name)
            .fetch_one(&mut tx)
            .await?;

        if count.0 > 0 {
            return Err(Box::new(ValidationError::new(&format!(
                "The table {} and model {} already exists.",
                table_name, model_name
            ))));
        }

        let m = match metadata {
            Some(m) => m,
            None => "".to_string(),
        };

        let sql_str = "INSERT INTO biomedgps_embedding_metadata (table_name, model_name, model_type, description, datasets, dimension, metadata) VALUES ($1, $2, $3, $4, $5, $6, $7) RETURNING id, created_at";

        let record = sqlx::query_as::<_, (i64, DateTime<Utc>)>(&sql_str)
            .bind(table_name)
            .bind(model_name)
            .bind(model_type)
            .bind(description)
            .bind(datasets)
            .bind(dimension as i32)
            .bind(&m)
            .fetch_one(&mut tx)
            .await?;

        // Check if the table exists
        let entity_emb_table_name = get_entity_emb_table_name(table_name);
        let relation_emb_table_name = get_relation_emb_table_name(table_name);
        info!(
            "Check if the entity embedding table ({}) and relation embedding table ({}) exist.",
            &entity_emb_table_name, &relation_emb_table_name
        );
        let err_msg = match check_table_is_valid(&pool, &vec![&entity_emb_table_name]).await {
            Ok(_) => "".to_string(),
            Err(e) => {
                // Create the entity embedding table and relation embedding table
                match Self::create_entity_emb_table(&mut tx, table_name, dimension).await {
                    Ok(_) => "".to_string(),
                    Err(e) => e.to_string(),
                }
            }
        };

        if !err_msg.is_empty() {
            tx.rollback().await?;
            return Err(Box::new(ValidationError::new(&format!(
                "Create the entity embedding table failed: {}",
                err_msg
            ))));
        } else {
            info!("The entity embedding table has been created successfully.");
        }

        let err_msg = match check_table_is_valid(&pool, &vec![&relation_emb_table_name]).await {
            Ok(_) => "".to_string(),
            Err(e) => {
                // Create the entity embedding table and relation embedding table
                match Self::create_relation_emb_table(&mut tx, table_name, dimension).await {
                    Ok(_) => "".to_string(),
                    Err(e) => e.to_string(),
                }
            }
        };

        if !err_msg.is_empty() {
            tx.rollback().await?;
            return Err(Box::new(ValidationError::new(&format!(
                "Create the relation embedding table failed: {}",
                err_msg
            ))));
        } else {
            info!("The relation embedding table has been created successfully.");
        }

        tx.commit().await?;

        Ok(EmbeddingMetadata {
            id: record.0,
            table_name: table_name.to_string(),
            model_name: model_name.to_string(),
            model_type: model_type.to_string(),
            description: description.to_string(),
            datasets: datasets
                .iter()
                .map(|s| s.to_string())
                .collect::<Vec<String>>(),
            created_at: record.1,
            dimension: dimension as i32,
            metadata: Some(m.clone()),
        })
    }

    pub async fn get_embedding_metadata_by_id(
        pool: &sqlx::PgPool,
        id: i64,
    ) -> Result<EmbeddingMetadata, Box<dyn Error>> {
        let sql_str = "SELECT * FROM biomedgps_embedding_metadata WHERE id = $1";

        let metadata = sqlx::query_as::<_, EmbeddingMetadata>(sql_str)
            .bind(id)
            .fetch_one(pool)
            .await?;

        Ok(metadata)
    }

    pub async fn get_embedding_metadata(
        pool: &sqlx::PgPool,
        query: &Option<ComposeQuery>,
        page: Option<u64>,
        page_size: Option<u64>,
        order_by: Option<&str>,
    ) -> Result<EmbeddingRecordResponse<EmbeddingMetadata>, anyhow::Error> {
        EmbeddingRecordResponse::<EmbeddingMetadata>::get_records(
            pool,
            "biomedgps_embedding_metadata",
            query,
            page,
            page_size,
            order_by,
        )
        .await
    }

    pub async fn import_embedding_metadata(
        pool: &sqlx::PgPool,
        filepath: &PathBuf,
        delimiter: u8,
        drop: bool,
    ) -> Result<(), Box<dyn Error>> {
        if drop {
            drop_table(&pool, "biomedgps_embedding_metadata").await;
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

        let mut line_number = 1;
        for result in reader.deserialize() {
            let mut record: EmbeddingMetadata = match result {
                Ok(r) => r,
                Err(e) => {
                    let error_msg = parse_csv_error(&e);
                    return Err(Box::new(ValidationError::new(&error_msg)));
                }
            };

            record.id = line_number;
            line_number += 1;

            let sql_str = "INSERT INTO biomedgps_embedding_metadata (id, table_name, model_name, model_type, description, datasets, dimension, metadata) VALUES ($1, $2, $3, $4, $5, $6, $7, $8)";

            let query = sqlx::query(&sql_str)
                .bind(record.id)
                .bind(record.table_name)
                .bind(record.model_name)
                .bind(record.model_type)
                .bind(record.description)
                .bind(record.datasets)
                .bind(record.dimension)
                .bind(record.metadata);

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

impl CheckData for EmbeddingMetadata {
    fn check_csv_is_valid(filepath: &PathBuf) -> Vec<Box<dyn Error>> {
        Self::check_csv_is_valid_default::<EmbeddingMetadata>(filepath)
    }

    fn unique_fields() -> Vec<String> {
        vec!["table_name".to_string(), "model_name".to_string()]
    }

    fn fields() -> Vec<String> {
        vec![
            // "id".to_string(),
            "table_name".to_string(),
            "model_name".to_string(),
            "model_type".to_string(),
            "description".to_string(),
            "datasets".to_string(),
            "dimension".to_string(),
            "metadata".to_string(),
        ]
    }
}

/// A struct for entity embedding, it is used for import entity embeddings into database from csv file.
/// Only for internal use, not for api.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, sqlx::FromRow, Object, Validate)]
pub struct EntityEmbedding {
    // Ignore this field when deserialize from json
    #[serde(skip_deserializing)]
    #[oai(read_only)]
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
        table_name: Option<&str>,
    ) -> Result<(), Box<dyn Error>> {
        let mut tx = pool.begin().await.unwrap();
        let real_table_name = match table_name {
            Some(t) => get_entity_emb_table_name(t),
            None => get_entity_emb_table_name(DEFAULT_MODEL_NAME),
        };

        if drop {
            drop_table(&pool, &real_table_name).await;
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

        let mut line_number = 1;
        for result in reader.deserialize() {
            let mut record: EntityEmbedding = match result {
                Ok(r) => r,
                Err(e) => {
                    let error_msg = parse_csv_error(&e);
                    return Err(Box::new(ValidationError::new(&error_msg)));
                }
            };

            record.embedding_id = line_number;
            line_number += 1;

            let sql_str = format!("INSERT INTO {} (embedding_id, entity_id, entity_type, entity_name, embedding) VALUES ($1, $2, $3, $4, $5)", real_table_name);

            // Execute the no more than 1000 queries in one transaction
            let query = sqlx::query(&sql_str)
                .bind(record.embedding_id)
                .bind(record.entity_id)
                .bind(record.entity_type)
                .bind(record.entity_name)
                .bind(record.embedding)
                .execute(&mut tx)
                .await;

            match query {
                Ok(_) => {}
                Err(e) => {
                    return Err(Box::new(e));
                }
            };
        }

        tx.commit().await.unwrap();

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
            // "embedding_id".to_string(),
            "entity_id".to_string(),
            "entity_type".to_string(),
            "entity_name".to_string(),
            "embedding".to_string(),
        ]
    }
}

/// A legacy struct for relation embedding, it is only for checking the relation embedding csv file.
#[derive(Debug, Clone, Deserialize, PartialEq, sqlx::FromRow, Object, Validate)]
pub struct LegacyRelationEmbedding {
    #[validate(length(
        max = "DEFAULT_MAX_LENGTH",
        min = "DEFAULT_MIN_LENGTH",
        message = "The length of relation_type should be between 1 and 64."
    ))]
    pub id: String,

    #[serde(deserialize_with = "text2vector")]
    pub embedding: Vector,
}

impl LegacyRelationEmbedding {
    pub async fn import_relation_embeddings(
        pool: &sqlx::PgPool,
        filepath: &PathBuf,
        annotated_relation_file: &Option<PathBuf>,
        drop: bool,
        table_name: Option<&str>,
        delimiter: u8,
    ) -> Result<(), Box<dyn Error>> {
        let real_table_name = match table_name {
            Some(t) => get_relation_emb_table_name(t),
            None => get_relation_emb_table_name(DEFAULT_MODEL_NAME),
        };

        if drop {
            drop_table(&pool, &real_table_name).await;
        };

        let relation_type_mappings = match annotated_relation_file {
            None => HashMap::new(),
            Some(annotated_relation_file) => {
                match read_annotation_file(annotated_relation_file) {
                    Ok(mappings) => mappings,
                    Err(e) => {
                        return Err(e)
                    }
                }
            }
        };

        let mut reader = match csv::ReaderBuilder::new()
            .delimiter(delimiter)
            .from_path(filepath)
        {
            Ok(r) => r,
            Err(e) => {
                return Err(Box::new(e));
            }
        };

        debug!(
            "The columns of the relation embedding csv file: {:?}",
            reader.headers().unwrap()
        );

        let mut line_num = 1;
        for record in reader.deserialize() {
            let record: LegacyRelationEmbedding = record.unwrap();
            let relation_type = record.id;
            let formatted_relation_type = match relation_type_mappings.get(&relation_type) {
                Some(t) => t.to_string(),
                None => relation_type.clone(),
            };
            let sql_str = format!("INSERT INTO {} (embedding_id, relation_type, formatted_relation_type, embedding) VALUES ($1, $2, $3, $4)", real_table_name);

            let query = sqlx::query(&sql_str)
                .bind(line_num)
                .bind(relation_type)
                .bind(formatted_relation_type)
                .bind(record.embedding);

            match query.execute(pool).await {
                Ok(_) => {}
                Err(e) => {
                    return Err(Box::new(e));
                }
            };

            line_num += 1;
        }

        Ok(())
    }
}

impl CheckData for LegacyRelationEmbedding {
    fn check_csv_is_valid(filepath: &PathBuf) -> Vec<Box<dyn Error>> {
        Self::check_csv_is_valid_default::<LegacyRelationEmbedding>(filepath)
    }

    fn unique_fields() -> Vec<String> {
        vec!["id".to_string()]
    }

    fn fields() -> Vec<String> {
        vec!["id".to_string(), "embedding".to_string()]
    }
}

/// A struct for relation embedding, it is used for import relation embeddings into database from csv file.
#[derive(Debug, Clone, Deserialize, PartialEq, sqlx::FromRow, Object, Validate)]
pub struct RelationEmbedding {
    // Ignore this field when deserialize from json
    #[serde(skip_deserializing)]
    #[oai(read_only)]
    pub embedding_id: i64,

    #[validate(length(
        max = "DEFAULT_MAX_LENGTH",
        min = "DEFAULT_MIN_LENGTH",
        message = "The length of relation_type should be between 1 and 64."
    ))]
    pub relation_type: String,

    #[validate(length(
        max = "DEFAULT_MAX_LENGTH",
        min = "DEFAULT_MIN_LENGTH",
        message = "The length of formatted_relation_type should be between 1 and 64."
    ))]
    pub formatted_relation_type: String,

    #[serde(deserialize_with = "text2vector")]
    pub embedding: Vector,
}

impl RelationEmbedding {
    pub async fn new(
        pool: &sqlx::PgPool,
        relation_type: &str,
        formatted_relation_type: &str,
        embedding: &Vec<f32>,
        table_name: Option<&str>,
    ) -> Result<RelationEmbedding, Box<dyn Error>> {
        let real_table_name = match table_name {
            Some(t) => get_relation_emb_table_name(t),
            None => get_relation_emb_table_name(DEFAULT_MODEL_NAME),
        };

        let sql_str = format!(
            "SELECT COUNT(*) FROM {} WHERE relation_type = $1 AND formatted_relation_type = $2",
            real_table_name
        );

        let count = sqlx::query_as::<_, (i64,)>(&sql_str)
            .bind(relation_type)
            .bind(formatted_relation_type)
            .fetch_one(pool)
            .await?;

        if count.0 > 0 {
            return Err(Box::new(ValidationError::new(
                "The relation type and formatted relation type already exists.",
            )));
        }

        let sql_str = format!("INSERT INTO {} (relation_type, formatted_relation_type, embedding) VALUES ($1, $2, $3) RETURNING embedding_id", real_table_name);

        let embedding_id = sqlx::query_as::<_, (i64,)>(&sql_str)
            .bind(relation_type)
            .bind(formatted_relation_type)
            .bind(embedding)
            .fetch_one(pool)
            .await?;

        Ok(RelationEmbedding {
            embedding_id: embedding_id.0,
            relation_type: relation_type.to_string(),
            formatted_relation_type: formatted_relation_type.to_string(),
            embedding: Vector::from(embedding.clone()),
        })
    }

    pub async fn import_relation_embeddings(
        pool: &sqlx::PgPool,
        filepath: &PathBuf,
        delimiter: u8,
        drop: bool,
        table_name: Option<&str>,
    ) -> Result<(), Box<dyn Error>> {
        let real_table_name = match table_name {
            Some(t) => get_relation_emb_table_name(t),
            None => get_relation_emb_table_name(DEFAULT_MODEL_NAME),
        };

        if drop {
            drop_table(&pool, &real_table_name).await;
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

        let mut line_number = 1;
        for result in reader.deserialize() {
            let mut record: RelationEmbedding = match result {
                Ok(r) => r,
                Err(e) => {
                    let error_msg = parse_csv_error(&e);
                    return Err(Box::new(ValidationError::new(&error_msg)));
                }
            };

            record.embedding_id = line_number;
            line_number += 1;

            let sql_str = format!("INSERT INTO {} (embedding_id, relation_type, formatted_relation_type, embedding) VALUES ($1, $2, $3, $4)", real_table_name);

            let query = sqlx::query(&sql_str)
                .bind(record.embedding_id)
                .bind(record.relation_type)
                .bind(record.formatted_relation_type)
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
            "formatted_relation_type".to_string(),
            "source_id".to_string(),
            "source_type".to_string(),
            "target_id".to_string(),
            "target_type".to_string(),
        ]
    }

    fn fields() -> Vec<String> {
        vec![
            // "embedding_id".to_string(),
            "relation_type".to_string(),
            "formatted_relation_type".to_string(),
            "source_id".to_string(),
            "source_type".to_string(),
            "target_id".to_string(),
            "target_type".to_string(),
            "embedding".to_string(),
        ]
    }
}
