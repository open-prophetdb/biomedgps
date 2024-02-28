//! SQL initialization strings for creating tables.

use crate::model::kge::{
    get_embedding_metadata, get_entity_emb_table_name, get_relation_emb_table_name,
    EmbeddingMetadata, DEFAULT_MODEL_NAME,
};
use crate::model::util::ValidationError;
use log::{debug, error};
use sqlx::PgPool;

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

    // TODO: We need to add more score functions here
    let score_function_name = if embedding_metadata.model_type == "TransE_l2" {
        "pgml.transe_l2_ndarray"
    } else if embedding_metadata.model_type == "TransE_l1" {
        "pgml.transe_l1_ndarray"
    } else if embedding_metadata.model_type == "DistMult" {
        "pgml.distmult_ndarray"
    } else if embedding_metadata.model_type == "ComplEx" {
        "pgml.complex_ndarray"
    } else {
        "pgml.transe_l2_ndarray"
    };

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

    // TODO: We need to add more score functions here
    let score_function_name = if embedding_metadata.model_type == "TransE_l2" {
        "pgml.transe_l2_ndarray"
    } else if embedding_metadata.model_type == "TransE_l1" {
        "pgml.transe_l1_ndarray"
    } else if embedding_metadata.model_type == "DistMult" {
        "pgml.distmult_ndarray"
    } else if embedding_metadata.model_type == "ComplEx" {
        "pgml.complex_ndarray"
    } else {
        "pgml.transe_l2_ndarray"
    };

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
    let init_sql = init_kg_score_sql(
        table_prefix,
        gamma,
        &embedding_metadata,
    );

    debug!("init_sql: {}", init_sql);
    let mut tx = pool.begin().await.unwrap();
    let delete_sql_str = format!(
        "DROP TABLE IF EXISTS {score_table};",
        score_table = get_kg_score_table_name(
            table_prefix.unwrap_or(DEFAULT_MODEL_NAME),
        )
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
