//! SQL initialization strings for creating tables.

use crate::model::kge::{
    get_embedding_metadata, get_entity_emb_table_name, get_relation_emb_table_name,
    EmbeddingMetadata, DEFAULT_MODEL_NAME,
};
use crate::model::util::ValidationError;
use anyhow::anyhow;
use log::{debug, error, info, warn};
use neo4rs::{query, Graph, Node as NeoNode};
use sqlx::PgPool;
use std::sync::Arc;

/// Generate a table name for the score table of the triple entity.
///
/// # Arguments
/// * `table_prefix` - The prefix of the table name, such as "biomedgps".
/// * `first_entity_type` - The type of the first entity, such as "Compound".
/// * `second_entity_type` - The type of the second entity, such as "Disease".
/// * `third_entity_type` - The type of the third entity, such as "Symptom".
///
/// # Returns
/// `String` - The table name for the score table of the triple entity, such as "biomedgps_compound_disease_symptom_score".
///
/// # Example
/// ```
/// use biomedgps::model::init_db::get_triple_entity_score_table_name;
/// let table_name = get_triple_entity_score_table_name("biomedgps", "Compound", "Disease", "Symptom");
/// assert_eq!(table_name, "biomedgps_compound_disease_symptom_score");
/// ```
///
pub fn get_triple_entity_score_table_name(
    table_prefix: &str,
    first_entity_type: &str,  // Such as "Compound"
    second_entity_type: &str, // Such as "Disease"
    third_entity_type: &str,  // Such as "Symptom"
) -> String {
    // Such as "biomedgps_compound_disease_symptom_score", the table_prefix need to match with the table name related to the KGE model.
    format!(
        "{}_{}_{}_{}_score",
        table_prefix,
        first_entity_type.to_ascii_lowercase(),
        second_entity_type.to_ascii_lowercase(),
        third_entity_type.to_ascii_lowercase()
    )
}

/// Generate the SQL query for initializing the score table for a triple entity.
///
/// # Arguments
/// * `first_entity_type` - The type of the first entity, such as "Compound".
/// * `second_entity_type` - The type of the second entity, such as "Disease".
/// * `third_entity_type` - The type of the third entity, such as "Symptom".
/// * `first_second_relation_type` - The relation type between the first and second entities.
/// * `second_third_relation_type` - The relation type between the second and third entities.
/// * `table_prefix` - Optional prefix for the table name. If not provided, the default model name will be used.
/// * `gamma` - The gamma value used in the score calculation.
/// * `embedding_metadata` - The metadata of the embeddings used in the score calculation.
///
/// # Returns
/// `String` - The SQL query for initializing the score table.
///
/// # Example
/// ```
/// use biomedgps::model::init_db::init_score_sql;
/// let sql_query = init_score_sql(
///     "Compound",
///     "Disease",
///     "Symptom",
///     "has_compound",
///     "has_disease",
///     Some("biomedgps"),
///     12.0,
///     &embedding_metadata,
/// );
///
/// assert!(sql_query.contains("biomedgps_compound_disease_symptom_score"));
/// ```
///
pub fn init_score_sql(
    first_entity_type: &str,  // Such as "Compound"
    second_entity_type: &str, // Such as "Disease"
    third_entity_type: &str,  // Such as "Symptom"
    first_second_relation_type: &str,
    second_third_relation_type: &str,
    table_prefix: Option<&str>,
    gamma: f64,
    embedding_metadata: &EmbeddingMetadata,
) -> String {
    let table_prefix = table_prefix.unwrap_or(DEFAULT_MODEL_NAME);
    let score_table_name = get_triple_entity_score_table_name(
        table_prefix,
        first_entity_type,
        second_entity_type,
        third_entity_type,
    );

    let score_function_name = embedding_metadata.detect_score_fn();

    format!(
        r#"
            WITH first_second AS (
                SELECT
                    id,
                    source_id AS first_id,
                    target_id AS second_id,
                    relation_type AS first_second_relation_type
                FROM biomedgps_relation
                WHERE relation_type = '{first_second_relation_type}'
            ),
            second_third AS (
                SELECT
                    id,
                    source_id AS second_id,
                    target_id AS third_id,
                    relation_type AS second_third_relation_type
                FROM biomedgps_relation
                WHERE relation_type = '{second_third_relation_type}'
            ),
            combined AS (
                SELECT
                    cd.first_id,
                    cd.second_id,
                    ds.third_id,
                    cd.first_second_relation_type,
                    ds.second_third_relation_type
                FROM first_second cd
                JOIN second_third ds ON cd.second_id = ds.second_id
            ),
            embeddings AS (
                SELECT
                    c.*,
                    cd_emb.embedding AS first_second_embedding,
                    ds_emb.embedding AS second_third_embedding
                FROM combined c
                JOIN {realtion_emb_table} cd_emb ON c.first_second_relation_type = cd_emb.relation_type
                JOIN {realtion_emb_table} ds_emb ON c.second_third_relation_type = ds_emb.relation_type
            ),
            final_embeddings AS (
                SELECT
                    e.*,
                    ce.embedding AS first_embedding,
                    de.embedding AS second_embedding,
                    se.embedding AS third_embedding
                FROM embeddings e
                JOIN {entity_emb_table} ce ON e.first_id = ce.entity_id
                JOIN {entity_emb_table} de ON e.second_id = de.entity_id
                JOIN {entity_emb_table} se ON e.third_id = se.entity_id
            )
            SELECT
                first_id AS source_id,
                '{first_entity_type}' AS source_type,
                second_id AS {second_entity_type_lower_str}_id,
                third_id AS target_id,
                '{third_entity_type}' AS target_type,
                pgml.mean(ARRAY[
                    {score_function_name}(
                        vector_to_float4(tt.first_embedding, {dimension}, false),
                        vector_to_float4(tt.first_second_embedding, {dimension}, false),
                        vector_to_float4(tt.second_embedding, {dimension}, false),
                        {gamma},
                        true,
                        false
                    ),
                    {score_function_name}(
                        vector_to_float4(tt.second_embedding, {dimension}, false),
                        vector_to_float4(tt.second_third_embedding, {dimension}, false),
                        vector_to_float4(tt.third_embedding, {dimension}, false),
                        {gamma},
                        true,
                        false
                    )
                ]) AS score
            INTO TABLE {score_table}
            FROM final_embeddings tt;
        "#,
        score_table = score_table_name,
        first_second_relation_type = first_second_relation_type,
        second_third_relation_type = second_third_relation_type,
        first_entity_type = first_entity_type,
        second_entity_type_lower_str = second_entity_type.to_ascii_lowercase(),
        third_entity_type = third_entity_type,
        realtion_emb_table = get_relation_emb_table_name(table_prefix),
        entity_emb_table = get_entity_emb_table_name(table_prefix),
        score_function_name = score_function_name,
        dimension = embedding_metadata.dimension,
    )
}

/// Create the score table for a triple entity.
///
/// # Arguments
/// * `pool` - The database connection pool.
/// * `first_entity_type` - The type of the first entity, such as "Compound".
/// * `second_entity_type` - The type of the second entity, such as "Disease".
/// * `third_entity_type` - The type of the third entity, such as "Symptom".
/// * `first_second_relation_type` - The relation type between the first and second entities.
/// * `second_third_relation_type` - The relation type between the second and third entities.
/// * `table_prefix` - Optional prefix for the table name. If not provided, the default model name will be used.
///
/// # Returns
/// `Result<(), ValidationError>` - The result of creating the score table.
///
/// # Example
/// ```
/// use biomedgps::model::init_db::create_score_table;
/// use sqlx::PgPool;
/// use std::env;
///
/// #[tokio::main]
/// async fn main() {
///     let db_url = env::var("DATABASE_URL").expect("DATABASE_URL is not set in the environment variables");
///     let pool = PgPool::connect(&db_url).await.unwrap();
///     let first_entity_type = "Compound";
///     let second_entity_type = "Disease";
///     let third_entity_type = "Symptom";
///     let first_second_relation_type = "treats";
///     let second_third_relation_type = "causes";
///     let table_prefix = "biomedgps";
///     let result = create_score_table(
///         &pool,
///         first_entity_type,
///         second_entity_type,
///         third_entity_type,
///         first_second_relation_type,
///         second_third_relation_type,
///         Some(table_prefix),
///     ).await;
///     assert!(result.is_ok());
/// }
/// ```
///
pub async fn create_score_table(
    pool: &PgPool,
    first_entity_type: &str,  // Such as "Compound"
    second_entity_type: &str, // Such as "Disease"
    third_entity_type: &str,  // Such as "Symptom"
    first_second_relation_type: &str,
    second_third_relation_type: &str,
    table_prefix: Option<&str>,
) -> Result<(), ValidationError> {
    if !first_second_relation_type
        .contains(&format!("{}:{}", first_entity_type, second_entity_type))
        && second_third_relation_type
            .contains(&format!("{}:{}", second_entity_type, third_entity_type))
    {
        let error_msg = format!(
            "The relation type {} is not correct, because the order of the entity types is not matched with the entity types {} and {} you provided",
            first_second_relation_type, first_entity_type, second_entity_type
        );
        error!("{}", error_msg);
        return Err(ValidationError::new(&error_msg, vec![]));
    }

    if !second_third_relation_type
        .contains(&format!("{}:{}", second_entity_type, third_entity_type))
    {
        let error_msg = format!(
            "The relation type {} is not correct, because the order of the entity types is not matched with the entity types {} and {} you provided",
            second_third_relation_type, second_entity_type, third_entity_type
        );
        error!("{}", error_msg);
        return Err(ValidationError::new(&error_msg, vec![]));
    }

    // TODO: We need to allow the user to set the score function, gamma and exp_enabled or get them from the model.
    let embedding_metadata = match get_embedding_metadata(
        &table_prefix.unwrap_or(DEFAULT_MODEL_NAME),
    ) {
        Some(metadata) => metadata,
        None => {
            error!("Failed to get the embedding metadata from the database");
            return Err(ValidationError::new(
                "Failed to get the embedding metadata from the database, so we don't know how to calculate the similarity for the node. Please check the database or the model/table name you provided.",
                vec![],
            ));
        }
    };

    let gamma = 12.0;
    let init_sql = init_score_sql(
        first_entity_type,
        second_entity_type,
        third_entity_type,
        first_second_relation_type,
        second_third_relation_type,
        table_prefix,
        gamma,
        &embedding_metadata,
    );

    debug!("init_sql: {}", init_sql);
    let mut tx = pool.begin().await.unwrap();
    let delete_sql_str = format!(
        "DROP TABLE IF EXISTS {score_table};",
        score_table = get_triple_entity_score_table_name(
            table_prefix.unwrap_or(DEFAULT_MODEL_NAME),
            first_entity_type,
            second_entity_type,
            third_entity_type,
        )
    );
    match sqlx::query(&delete_sql_str).execute(&mut tx).await {
        Ok(_) => {
            debug!("The score table is deleted successfully");
        }
        Err(e) => {
            error!("Failed to delete the score table: {}", e);
            return Err(ValidationError::new(
                &format!("Failed to delete the score table: {}", e),
                vec![],
            ));
        }
    }

    match sqlx::query(&init_sql).execute(&mut tx).await {
        Ok(_) => {
            debug!("The score table is created successfully");
        }
        Err(e) => {
            error!("Failed to create the score table: {}", e);
            return Err(ValidationError::new(
                &format!("Failed to create the score table: {}", e),
                vec![],
            ));
        }
    };

    // Commit the transaction
    match tx.commit().await {
        Ok(_) => Ok(()),
        Err(e) => {
            error!("Failed to commit the transaction: {}", e);
            return Err(ValidationError::new(
                &format!("Failed to commit the transaction: {}", e),
                vec![],
            ));
        }
    }
}

/// Generate a table name for the score table of the knowledge graph.
///
/// # Arguments
/// * `table_prefix` - The prefix of the table name, such as "biomedgps".
///
/// # Returns
/// `String` - The table name for the score table of the knowledge graph, such as "biomedgps_relation_with_score".
///
/// # Example
/// ```
/// use biomedgps::model::init_db::get_kg_score_table_name;
/// let table_name = get_kg_score_table_name("biomedgps");
/// assert_eq!(table_name, "biomedgps_relation_with_score");
/// ```
///
pub fn get_kg_score_table_name(table_prefix: &str) -> String {
    format!("{}_relation_with_score", table_prefix)
}

pub fn init_kg_score_sql(
    table_prefix: Option<&str>,
    gamma: f64,
    embedding_metadata: &EmbeddingMetadata,
) -> String {
    let table_prefix = table_prefix.unwrap_or(DEFAULT_MODEL_NAME);
    let score_table_name = get_kg_score_table_name(table_prefix);
    let score_function_name = embedding_metadata.detect_score_fn();

    format!(
        r#"
            WITH kg_embeddings AS (
                SELECT
                    c.*,
                    cd_emb.embedding AS relation_type_embedding
                FROM biomedgps_relation c
                LEFT JOIN {realtion_emb_table} cd_emb ON c.relation_type = cd_emb.relation_type
            ),
            final_embeddings AS (
                SELECT
                    e.*,
                    ce.embedding AS source_embedding,
                    de.embedding AS target_embedding
                FROM kg_embeddings e
                LEFT JOIN {entity_emb_table} ce ON e.source_id = ce.entity_id AND e.source_type = ce.entity_type
                LEFT JOIN {entity_emb_table} de ON e.target_id = de.entity_id AND e.target_type = de.entity_type
            )
            SELECT
                id AS id,
                source_id AS source_id,
                source_type AS source_type,
                target_id AS target_id,
                target_type AS target_type,
                relation_type AS relation_type,
                formatted_relation_type AS formatted_relation_type,
                key_sentence AS key_sentence,
                resource AS resource,
                dataset AS dataset,
                pmids AS pmids,
                {score_function_name}(
                    vector_to_float4(tt.source_embedding, {dimension}, false),
                    vector_to_float4(tt.relation_type_embedding, {dimension}, false),
                    vector_to_float4(tt.target_embedding, {dimension}, false),
                    {gamma},
                    true,
                    false
                )::FLOAT8 AS score
            INTO TABLE {score_table}
            FROM final_embeddings tt;
        "#,
        score_table = score_table_name,
        realtion_emb_table = get_relation_emb_table_name(table_prefix),
        entity_emb_table = get_entity_emb_table_name(table_prefix),
        score_function_name = score_function_name,
        dimension = embedding_metadata.dimension,
        gamma = gamma
    )
}

/// Create the score table for the knowledge graph.
///
/// # Arguments
/// * `pool` - The database connection pool.
/// * `table_prefix` - Optional prefix for the table name. If not provided, the default model name will be used.
///
/// # Returns
/// `Result<(), ValidationError>` - The result of creating the score table.
///
/// # Example
/// ```
/// use biomedgps::model::init_db::create_kg_score_table;
/// use sqlx::PgPool;
/// use std::env;
///
/// #[tokio::main]
/// async fn main() {
///     let db_url = env::var("DATABASE_URL").expect("DATABASE_URL is not set in the environment variables");
///     let pool = PgPool::connect(&db_url).await.unwrap();
///     let table_prefix = "biomedgps";
///     let result = create_kg_score_table(&pool, Some(table_prefix)).await;
///     assert!(result.is_ok());
/// }
/// ```
///
pub async fn create_kg_score_table(
    pool: &PgPool,
    table_prefix: Option<&str>,
) -> Result<(), ValidationError> {
    // TODO: We need to allow the user to set the score function, gamma and exp_enabled or get them from the model.
    let embedding_metadata = match get_embedding_metadata(
        &table_prefix.unwrap_or(DEFAULT_MODEL_NAME),
    ) {
        Some(metadata) => metadata,
        None => {
            error!("Failed to get the embedding metadata from the database");
            return Err(ValidationError::new(
                "Failed to get the embedding metadata from the database, so we don't know how to calculate the similarity for the node. Please check the database or the model/table name you provided.",
                vec![],
            ));
        }
    };

    let gamma = 12.0;
    let init_sql = init_kg_score_sql(table_prefix, gamma, &embedding_metadata);

    debug!("init_sql: {}", init_sql);
    let mut tx = pool.begin().await.unwrap();
    let delete_sql_str = format!(
        "DROP TABLE IF EXISTS {score_table};",
        score_table = get_kg_score_table_name(table_prefix.unwrap_or(DEFAULT_MODEL_NAME),)
    );
    match sqlx::query(&delete_sql_str).execute(&mut tx).await {
        Ok(_) => {
            debug!("The kg score table is deleted successfully");
        }
        Err(e) => {
            error!("Failed to delete the score table: {}", e);
            return Err(ValidationError::new(
                &format!("Failed to delete the score table: {}", e),
                vec![],
            ));
        }
    }

    match sqlx::query(&init_sql).execute(&mut tx).await {
        Ok(_) => {
            debug!("The kg score table is created successfully");
        }
        Err(e) => {
            error!("Failed to create the score table: {}", e);
            return Err(ValidationError::new(
                &format!("Failed to create the score table: {}", e),
                vec![],
            ));
        }
    };

    // Commit the transaction
    match tx.commit().await {
        Ok(_) => Ok(()),
        Err(e) => {
            error!("Failed to commit the transaction: {}", e);
            return Err(ValidationError::new(
                &format!("Failed to commit the transaction: {}", e),
                vec![],
            ));
        }
    }
}

/// Generate the attribute name for the score of the relation in the graph database.
///
/// # Arguments
/// * `table_prefix` - The prefix of the table name, such as "biomedgps".
///
/// # Returns
/// `String` - The attribute name for the score of the relation in the graph database, such as "biomedgps_score".
///
/// # Example
/// ```
/// use biomedgps::model::init_db::get_score_attr_name;
/// let attr_name = get_score_attr_name("biomedgps");
/// assert_eq!(attr_name, "biomedgps_score");
/// ```
///
fn get_score_attr_name(table_prefix: &str) -> String {
    format!("{}_score", table_prefix)
}

/// Convert the score table of the triple entity to the graph database.
///
/// # Arguments
/// * `jdbc_url` - The JDBC URL of the database.
/// * `graphdb` - The graph database.
/// * `table_prefix` - Optional prefix for the table name. If not provided, the default model name will be used.
/// * `total` - The total number of the records in the score table.
/// * `batch_size` - The batch size for the iteration.
/// * `only_score` - If true, only the score will be set for the relation, otherwise the resource, dataset, pmids and key_sentence will also be set. If you want to update all the attributes, you need to set it to false. Otherwise, we assume that you have an idx attribute for the relation.
pub async fn kg_score_table2graphdb(
    database_url: &str,
    graphdb: &Arc<Graph>,
    table_prefix: Option<&str>,
    total: usize,
    batch_size: usize,
    only_score: bool,
) -> Result<(), anyhow::Error> {
    let jdbc_url = database_url.replace("postgres://", "jdbc:postgresql://");
    let table_prefix = table_prefix.unwrap_or(DEFAULT_MODEL_NAME);
    let score_table_name = get_kg_score_table_name(table_prefix);
    let score_attr_name = get_score_attr_name(table_prefix);
    info!("jdbc_url: {}", jdbc_url);

    info!("Need to convert {} records to the graph database", total);
    let batch = total as usize / batch_size;

    for i in 0..batch {
        info!(
            "Run the batch: {}/{}, each batch has {} records",
            i, batch, batch_size
        );
        let offset = i * batch_size;
        // https://github.com/pgjdbc/pgjdbc
        let query_str = if !only_score {
            format!(
                r#"
                    CALL apoc.periodic.iterate(
                        'CALL apoc.load.jdbc(
                            "{jdbc_url}",
                            "SELECT id, source_id, source_type, target_id, target_type, relation_type, 
                                    formatted_relation_type, key_sentence, resource, dataset, pmids, score, 
                                    COALESCE(source_type, \'\') || \'::\' || COALESCE(source_id, \'\') AS source_node_id, 
                                    COALESCE(target_type, \'\') || \'::\' || COALESCE(target_id, \'\') AS target_node_id,
                                    COALESCE(source_id, \'\') || \'-\' || COALESCE(relation_type, \'\') || \'-\' || COALESCE(target_id, \'\') AS idx
                             FROM {score_table} LIMIT {limit} OFFSET {offset}") YIELD row RETURN row',
                        'WITH row
                         CALL apoc.merge.node([row.source_type], {{ idx: row.source_node_id }}) YIELD node as source
                         CALL apoc.merge.node([row.target_type], {{ idx: row.target_node_id }}) YIELD node as target
                         CALL apoc.merge.relationship(source, row.relation_type, {{}}, {{}}, target, {{}}) YIELD rel
                         SET rel.{score_attr_name} = row.score,
                             rel.idx = row.idx,
                             rel.resource = row.resource,
                             rel.dataset = row.dataset,
                             rel.pmids = row.pmids,
                             rel.key_sentence = row.key_sentence',
                        {{batchSize: {batch_size}, parallel: true, iterateList: true, retries: 0}}
                    )
                    YIELD batches, total, errorMessages, failedOperations
                    RETURN batches, total, errorMessages, failedOperations
                "#,
                limit = batch_size * 5,
                offset = offset,
                jdbc_url = jdbc_url,
                score_table = score_table_name,
                score_attr_name = score_attr_name,
                batch_size = batch_size,
            )
        } else {
            format!(
                r#"
                    CALL apoc.periodic.iterate(
                        'CALL apoc.load.jdbc(
                            "{jdbc_url}",
                            "SELECT id, source_id, source_type, target_id, target_type, relation_type, formatted_relation_type, 
                                    key_sentence, resource, dataset, pmids, score, 
                                    COALESCE(source_id, \'\') || \'-\' || COALESCE(relation_type, \'\') || \'-\' || COALESCE(target_id, \'\') AS idx 
                            FROM {score_table} LIMIT {limit} OFFSET {offset}")
                         YIELD row RETURN row',
                        'WITH row
                         MATCH ()-[r]->() WHERE r.idx = row.idx 
                         SET r.{score_attr_name} = row.score
                        {{batchSize: {batch_size}, parallel: true, iterateList: true, retries: 0}}
                    )
                    YIELD batches, total, errorMessages, failedOperations
                    RETURN batches, total, errorMessages, failedOperations
                "#,
                limit = batch_size * 5,
                offset = offset,
                jdbc_url = jdbc_url,
                score_table = score_table_name,
                score_attr_name = score_attr_name,
                batch_size = batch_size,
            )
        };

        info!("query_str: {}", query_str);

        let err_msg = "if you encounter a connection error and you are using the docker container, please try to set the --db-host to the hostname:port of your database docker container.";

        match graphdb.execute(query(&query_str)).await {
            Ok(mut result) => {
                // Extract the batches, total and errorMessages from the result
                while let Some(row) = result.next().await? {
                    info!("row: {:?}", row);
                    let batches = match row.get("batches") {
                        Some(b) => b,
                        None => 0,
                    };
                    let total = match row.get("total") {
                        Some(t) => t,
                        None => 0,
                    };
                    let failed_operations = match row.get::<i64>("failedOperations") {
                        Some(f) => f,
                        None => continue,
                    };
                    let msg = match row.get::<NeoNode>("errorMessages") {
                        Some(e) => e,
                        None => continue,
                    };

                    if batches == 0 && total == 0 {
                        warn!("The score table is empty.");
                        return Ok(());
                    }

                    if failed_operations > 0 {
                        error!(
                            "The score table is not empty, but the number of failed operations is {} out of {}. The error message: {:?}",
                            failed_operations, total, msg
                        );

                        return Err(
                            anyhow!(
                                "The score table is not empty, but the number of failed operations is {} out of {}. The error message: {:?}",
                                failed_operations, total, msg
                            )
                        );
                    }
                }
                info!(
                    "The score table {} is converted to the graph database successfully",
                    score_table_name
                );
            }
            Err(e) => {
                error!(
                    "Failed to set the score for the relation: {}, {}",
                    e, err_msg
                );
                return Err(anyhow::Error::new(e));
            }
        }
    }

    info!("The score table is converted to the graph database successfully");

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::setup_test_db;
    use chrono::{DateTime, NaiveDateTime, Utc};

    #[test]
    fn test_get_triple_entity_score_table_name() {
        let table_name =
            get_triple_entity_score_table_name("biomedgps", "Compound", "Disease", "Symptom");
        assert_eq!(table_name, "biomedgps_compound_disease_symptom_score");
    }

    #[test]
    fn test_init_score_sql() {
        let table_prefix = "biomedgps";
        let first_entity_type = "Compound";
        let second_entity_type = "Disease";
        let third_entity_type = "Symptom";
        let first_second_relation_type = "treats";
        let second_third_relation_type = "causes";
        let gamma = 12.0;
        let embedding_metadata = EmbeddingMetadata {
            id: 1,
            metadata: None,
            model_name: "biomedgps_transe_l2".to_string(),
            model_type: "TransE_l2".to_string(),
            dimension: 400,
            table_name: "biomedgps".to_string(),
            created_at: DateTime::from_utc(NaiveDateTime::from_timestamp(0, 0), Utc),
            datasets: vec!["STRING".to_string()],
            description: "The entity embedding trained by the TransE_l2 model".to_string(),
        };

        let sql = init_score_sql(
            first_entity_type,
            second_entity_type,
            third_entity_type,
            first_second_relation_type,
            second_third_relation_type,
            Some(table_prefix),
            gamma,
            &embedding_metadata,
        );
        println!("sql: {}", sql);
        assert!(sql.contains("biomedgps_compound_disease_symptom_score"));
    }

    #[tokio::test]
    async fn test_create_score_table() {
        let pool = setup_test_db().await;
        let first_entity_type = "Compound";
        let second_entity_type = "Disease";
        let third_entity_type = "Symptom";
        let first_second_relation_type = "treats";
        let second_third_relation_type = "causes";
        let table_prefix = "biomedgps";
        let result = create_score_table(
            &pool,
            first_entity_type,
            second_entity_type,
            third_entity_type,
            first_second_relation_type,
            second_third_relation_type,
            Some(table_prefix),
        )
        .await;
        assert!(result.is_ok());
    }

    #[test]
    fn test_get_kg_score_table_name() {
        let table_name = get_kg_score_table_name("biomedgps");
        assert_eq!(table_name, "biomedgps_relation_with_score");
    }

    #[test]
    fn test_init_kg_score_sql() {
        let table_prefix = "biomedgps";
        let gamma = 12.0;
        let embedding_metadata = EmbeddingMetadata {
            id: 1,
            metadata: None,
            model_name: "biomedgps_transe_l2".to_string(),
            model_type: "TransE_l2".to_string(),
            dimension: 400,
            table_name: "biomedgps".to_string(),
            created_at: DateTime::from_utc(NaiveDateTime::from_timestamp(0, 0), Utc),
            datasets: vec!["STRING".to_string()],
            description: "The entity embedding trained by the TransE_l2 model".to_string(),
        };
        let sql = init_kg_score_sql(Some(table_prefix), gamma, &embedding_metadata);
        println!("sql: {}", sql);
        assert!(sql.contains("biomedgps_relation_with_score"));
    }

    #[tokio::test]
    async fn test_create_kg_score_table() {
        let pool = setup_test_db().await;
        let table_prefix = "biomedgps";
        let result = create_kg_score_table(&pool, Some(table_prefix)).await;
        assert!(result.is_ok());
    }
}
