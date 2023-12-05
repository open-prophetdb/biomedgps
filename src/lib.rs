#![doc = include_str!("../README.md")]

//! BioMedGPS library for knowledge graph construction and analysis.
pub mod algorithm;
pub mod api;
pub mod model;
pub mod pgvector;
pub mod query_builder;

use log::{debug, error, info, warn, LevelFilter};
use log4rs;
use log4rs::append::console::ConsoleAppender;
use log4rs::config::{Appender, Config, Logger, Root};
use log4rs::encode::pattern::PatternEncoder;
use model::core::EntityAttribute;
use neo4rs::{ConfigBuilder, Graph, Query};
use polars::prelude::{col, lit, CsvReader, IntoLazy, SerReader};
use std::error::Error;
use std::fs::Permissions;
use std::os::unix::fs::PermissionsExt;
use std::vec;

use crate::model::core::{
    CheckData, Entity, Entity2D, EntityEmbedding, KnowledgeCuration, Relation, RelationAttribute,
    RelationEmbedding, Subgraph,
};
use crate::model::util::{
    drop_table, get_delimiter, import_file_in_loop, show_errors, update_entity_metadata,
    update_relation_metadata,
};

use serde_json::Value;
use sqlx::migrate::Migrator;
use std::collections::HashMap;
use std::fs::File;
use std::io::Write;
use std::path::Path;
use std::path::PathBuf;
use tempfile::tempdir;
use url::form_urlencoded;

const MIGRATIONS: include_dir::Dir = include_dir::include_dir!("migrations");

pub async fn run_migrations(database_url: &str) -> sqlx::Result<()> {
    info!("Running migrations.");
    // Create a temporary directory.
    let dir = tempdir()?;

    for file in MIGRATIONS.files() {
        // Create each file in the temporary directory.
        let file_path = dir.path().join(file.path());
        let mut temp_file = File::create(&file_path)?;
        // Write the contents of the included file to the temporary file.
        temp_file.write_all(file.contents())?;
    }

    // Now we can create a Migrator from the temporary directory.
    info!("Importing migrations from {:?}", dir.path());
    // List all files in the temporary directory.
    for file in dir.path().read_dir()? {
        match file {
            Ok(file) => info!("Found file: {:?}", file.path()),
            Err(e) => warn!("Error: {:?}", e),
        }
    }
    let migrator = Migrator::new(Path::new(dir.path())).await?;

    let pool = sqlx::postgres::PgPoolOptions::new()
        .connect(database_url)
        .await?;

    migrator.run(&pool).await?;

    // Don't forget to cleanup the temporary directory.
    dir.close()?;
    info!("Migrations finished.");

    Ok(())
}

pub async fn check_curated_knowledges(pool: &sqlx::PgPool, file: &PathBuf, delimiter: u8) {
    // Get all source_id and source_type pairs from the biomedgps_knowledge_curation table and keep them in a HashMap. The key is the source_id and source_type pair, the value is a list of numbers which are the row numbers that have the same source_id and source_type.
    let mut curated_knowledges: HashMap<(String, String), Vec<i64>> = HashMap::new();
    let records = KnowledgeCuration::get_records(pool).await.unwrap();
    log::info!(
        "The number of records in the biomedgps_knowledge_curation table is {}.",
        records.len()
    );

    if records.len() == 0 {
        return;
    }

    for record in records {
        let key1 = (record.source_id, record.source_type);
        let key2 = (record.target_id, record.target_type);
        let value = record.id;

        for key in vec![key1, key2] {
            let v = curated_knowledges.get(&key).map_or_else(
                || vec![value],
                |v| {
                    let cloned_v = v.clone();
                    let mut cloned_v = cloned_v.to_vec();
                    cloned_v.push(value);
                    cloned_v
                },
            );
            curated_knowledges.insert(key, v);
        }
    }

    // Load the data file into a DataFrame.
    log::info!(
        "Loading the data file ({}) into a DataFrame.",
        file.display()
    );
    // How to set truncate_ragged_lines=true?

    let df = CsvReader::from_path(file)
        .unwrap()
        .with_delimiter(delimiter)
        .has_header(true)
        .finish()
        .unwrap();

    let mut errors = vec![];
    // Check the id-type pairs where are from the curated_knowledges hashmap whether all are in the data file. The data file contains id and label columns.
    for id_type_pair in curated_knowledges.keys() {
        let id = id_type_pair.0.clone();
        if id == "" || id == "Unknown:Unknown" {
            continue;
        }

        let type_ = id_type_pair.1.clone();
        let id_type_pair_ = format!("{}-{}", id, type_);

        let predicate = col("id").eq(lit(id)).and(col("label").eq(lit(type_)));
        let filtered_df = df.clone().lazy().filter(predicate).collect().unwrap();

        if filtered_df.height() == 0 {
            errors.push((
                id_type_pair_,
                curated_knowledges
                    .get(id_type_pair)
                    .unwrap()
                    .iter()
                    .map(|v| v.to_string())
                    .collect::<Vec<String>>()
                    .join(","),
            ));
        }
    }

    if errors.len() > 0 {
        error!("The following id-type pairs are not in the data file:");
        for error in errors {
            error!(
                "The id-type pair is {}, and the related ids are {}",
                error.0, error.1
            );
        }
        std::process::exit(1);
    }
}

async fn prepare_entity_queries(records: Vec<Entity>) -> Result<Vec<Query>, Box<dyn Error>> {
    let mut queries = Vec::new();

    for record in records {
        let label = record.label;
        let description = match record.description {
            Some(d) => d,
            None => "".to_string(),
        };
        let taxid = match record.taxid {
            Some(t) => t,
            None => "".to_string(),
        };
        let synonyms = match record.synonyms {
            Some(s) => s,
            None => "".to_string(),
        };
        let xrefs = match record.xrefs {
            Some(x) => x,
            None => "".to_string(),
        };

        let query_string = format!("MERGE (n:{} {{id: $id, name: $name, resource: $resource, description: $description, taxid: $taxid, synonyms: $synonyms, xrefs: $xrefs}}) ON CREATE SET n.id = $id", label);
        let query = Query::new(query_string)
            .param("id", record.id)
            .param("name", record.name)
            .param("resource", record.resource)
            .param("description", description)
            .param("taxid", taxid)
            .param("synonyms", synonyms)
            .param("xrefs", xrefs);
        queries.push(query);
    }

    Ok(queries)
}

pub async fn prepare_relation_queries(
    records: Vec<Relation>,
) -> Result<Vec<Query>, Box<dyn Error>> {
    let mut queries = Vec::new();

    for record in records {
        let label = record.relation_type;
        let key_sentence = match record.key_sentence {
            Some(d) => d,
            None => "".to_string(),
        };
        let pmids = match record.pmids {
            Some(t) => t,
            None => "".to_string(),
        };
        let query_string = format!(
            "MATCH (e1:{} {{id: $source_id}})
             MATCH (e2:{} {{id: $target_id}})
             MERGE (e1)-[r:{} {{resource: $resource, key_sentence: $key_sentence, pmids: $pmids}}]->(e2)",
            record.source_type, record.target_type, label
        );
        let query = Query::new(query_string)
            .param("source_id", record.source_id)
            .param("target_id", record.target_id)
            .param("pmids", pmids)
            .param("resource", record.resource)
            .param("key_sentence", key_sentence);
        queries.push(query);
    }

    Ok(queries)
}

pub async fn prepare_entity_attr_queries(
    records: Vec<EntityAttribute>,
) -> Result<Vec<Query>, Box<dyn Error>> {
    let mut queries = Vec::new();

    for record in records {
        let label = record.entity_type;
        let id = record.entity_id;
        let query_string = format!(
            "
            MATCH (e:{} {{id: $entity_id}})
            SET e.external_db_name = $external_db_name
            SET e.external_id = $external_id
            SET e.external_url = $external_url
            SET e.description = $description
            ",
            label // Directly use the label here
        );
        let query = Query::new(query_string)
            .param("entity_id", id)
            .param("external_db_name", record.external_db_name)
            .param("external_id", record.external_id)
            .param("external_url", record.external_url)
            .param("description", record.description);

        queries.push(query);
    }

    Ok(queries)
}

pub async fn prepare_relation_attr_queries(
    records: Vec<RelationAttribute>,
) -> Result<Vec<Query>, Box<dyn Error>> {
    let mut queries = Vec::new();

    let total = records.len();
    for record in records {
        let relation_id = record.relation_id; // The relation_id is like "<RELATION_TYPE>_<SOURCE_ID>_<TARGET_ID>".
        let relation_id = relation_id.split("_").collect::<Vec<&str>>();
        let label = relation_id[0];
        // The relation_type is like "<DB>::<RELATION>::<SOURCE_TYPE>:<TARGET_TYPE>". We need to split it to get the source_type and target_type.
        let types = label.split("::").collect::<Vec<&str>>();
        let source_type = types[2].split(":").collect::<Vec<&str>>()[0];
        let target_type = types[2].split(":").collect::<Vec<&str>>()[1];
        let source_id = relation_id[1];
        let target_id = relation_id[2];
        let moa_ids = record.moa_ids.split("|").collect::<Vec<&str>>();

        let query_string = format!(
            "
            MATCH (e1:{} {{id: $source_id}})-[r:{}]->(e2:{} {{id: $target_id}})
            SET r.attention_score = $attention_score
            SET r.moa_ids = $moa_ids
            ",
            source_type, label, target_type
        );
        let query = Query::new(query_string)
            .param("source_id", source_id)
            .param("target_id", target_id)
            .param("label", label)
            .param("attention_score", record.attention_score)
            .param("moa_ids", moa_ids);

        queries.push(query);
    }

    Ok(queries)
}

pub async fn batch_insert(
    queries: Vec<Query>,
    graphdb_host: &str,
    username: &str,
    password: &str,
    batch_size: usize,
) -> Result<(), Box<dyn Error>> {
    let graph = Graph::connect(
        ConfigBuilder::default()
            .uri(graphdb_host)
            .user(username)
            .password(password)
            .build()
            .unwrap(),
    )
    .await
    .unwrap();

    let total = queries.len();
    let mut imported = 0;
    for chunk in queries.chunks(batch_size) {
        let tx = graph.start_txn().await?;
        for query in chunk {
            tx.run(query.to_owned()).await?;
        }
        tx.commit().await?;
        imported += chunk.len();
        debug!("Imported {}/{} records.", imported, total);
    }

    Ok(())
}

pub fn create_temp_file(dir: &PathBuf, extension: Option<&str>) -> PathBuf {
    // Create a temporary file with the extension.
    debug!(
        "Creating a temporary file with extension {:?} in {:?}.",
        extension, dir
    );
    let extension = match extension {
        Some(e) => e,
        None => "",
    };
    let temp_file = tempfile::Builder::new()
        .suffix(format!(".{:?}", extension).as_str()) // Replace '.ext' with the desired extension
        .tempfile_in(dir)
        .unwrap();
    let temp_filepath = PathBuf::from(temp_file.path().to_str().unwrap());
    let permissions = Permissions::from_mode(0o755); // 0o755 is the octal representation of 755
    log::info!("Setting permissions to 755 for the temp file.");
    File::open(&temp_filepath)
        .expect("Failed to open the file")
        .set_permissions(permissions)
        .expect("Failed to set file permissions");

    return temp_filepath;
}

pub async fn import_graph_data(
    graphdb_host: &str,
    username: &str,
    password: &str,
    filepath: &Option<String>,
    filetype: &str,
    skip_check: bool,
    show_all_errors: bool,
    batch_size: usize,
) {
    let filepath = match filepath {
        Some(f) => f,
        None => {
            error!("Please specify the file path.");
            return;
        }
    };

    let mut files = vec![];
    if std::path::Path::new(&filepath).is_dir() {
        let paths = std::fs::read_dir(&filepath).unwrap();
        for path in paths {
            let path = path.unwrap().path();
            match get_delimiter(&path) {
                Ok(_d) => {
                    if path.is_file() {
                        files.push(path);
                    }
                }
                Err(_) => continue,
            };
        }
    } else {
        files.push(std::path::PathBuf::from(&filepath));
    }

    if files.is_empty() {
        error!("No valid files found. Only tsv/csv/txt files are supported.");
        std::process::exit(1);
    }

    for file in files {
        let filename = file.to_str().unwrap();
        info!("Importing {} into neo4j...", file.display());
        warn!("Please make sure that you have upload the data file into the importer directory of the neo4j database.");

        if !skip_check {
            let validation_errors = if filetype == "entity" {
                Entity::check_csv_is_valid(&file)
            } else if filetype == "relation" {
                Relation::check_csv_is_valid(&file)
            } else if filetype == "entity_attribute" {
                RelationAttribute::check_csv_is_valid(&file)
            } else if filetype == "relation_attribute" {
                RelationAttribute::check_csv_is_valid(&file)
            } else {
                error!("Invalid file type: {}", filetype);
                // Stop the program if the file type is invalid.
                std::process::exit(1);
            };

            if validation_errors.len() > 0 {
                error!("Invalid file: {}", filename);
                show_errors(&validation_errors, show_all_errors);
                warn!("Skipping {}...\n\n", filename);
                continue;
            } else {
                info!("{} is valid.", filename);
            }
        }

        let file = match file.file_name() {
            Some(f) => PathBuf::from(f.to_str().unwrap()),
            None => {
                error!("Invalid file: {}", filename);
                continue;
            }
        };

        let queries = if filetype == "entity" {
            let records = Entity::get_records(&file).unwrap();
            prepare_entity_queries(records).await.unwrap()
        } else if filetype == "relation" {
            let records = Relation::get_records(&file).unwrap();
            prepare_relation_queries(records).await.unwrap()
        } else if filetype == "entity_attribute" {
            let records = EntityAttribute::get_records(&file).unwrap();
            prepare_entity_attr_queries(records).await.unwrap()
        } else if filetype == "relation_attribute" {
            let records = RelationAttribute::get_records(&file).unwrap();
            prepare_relation_attr_queries(records).await.unwrap()
        } else {
            error!("Invalid file type: {}", filetype);
            // Stop the program if the file type is invalid.
            std::process::exit(1);
        };

        if queries.len() == 0 {
            error!("No queries generated.");
            continue;
        } else {
            match batch_insert(queries, graphdb_host, username, password, batch_size).await {
                Ok(_) => {
                    info!("Import {} into neo4j successfully.", filename);
                    return;
                }
                Err(e) => {
                    error!("Failed to import {} into neo4j: ({})", filename, e);
                    return;
                }
            }
        }
    }
}

pub async fn import_data(
    database_url: &str,
    filepath: &Option<String>,
    table: &str,
    drop: bool,
    skip_check: bool,
    show_all_errors: bool,
) {
    let pool = sqlx::postgres::PgPoolOptions::new()
        .connect(&database_url)
        .await
        .unwrap();

    let filepath = match filepath {
        Some(f) => f,
        None => {
            error!("Please specify the file path.");
            return;
        }
    };

    if table == "relation_metadata" {
        update_relation_metadata(&pool, &PathBuf::from(filepath), true)
            .await
            .unwrap();
        return;
    } else if table == "entity_metadata" {
        update_entity_metadata(&pool, true).await.unwrap();
        return;
    }

    if table == "entity_embedding" || table == "relation_embedding" {
        let file = PathBuf::from(filepath);

        if file.is_dir() {
            error!("Please specify the file path, not a directory.");
            return;
        };

        let delimiter = match get_delimiter(&file) {
            Ok(d) => d,
            Err(_) => {
                error!("Invalid filename: {}, no extension found.", file.display());
                return;
            }
        };

        match if table == "entity_embedding" {
            let errors = EntityEmbedding::check_csv_is_valid(&file);
            if errors.len() > 0 {
                show_errors(&errors, show_all_errors);
                return;
            } else {
                info!("The data file {} is valid.", file.display());
            }

            EntityEmbedding::import_entity_embeddings(&pool, &file, delimiter, drop).await
        } else {
            let errors = RelationEmbedding::check_csv_is_valid(&file);
            if errors.len() > 0 {
                show_errors(&errors, show_all_errors);
                return;
            };

            RelationEmbedding::import_relation_embeddings(&pool, &file, delimiter, drop).await
        } {
            Ok(_) => {
                info!("Import embeddings into {} table successfully.", table);
                return;
            }
            Err(e) => {
                error!("Failed to parse CSV: ({})", e);
                return;
            }
        }
    } else {
        let mut files = vec![];
        if std::path::Path::new(&filepath).is_dir() {
            let paths = std::fs::read_dir(&filepath).unwrap();
            for path in paths {
                let path = path.unwrap().path();
                match get_delimiter(&path) {
                    Ok(_d) => {
                        if path.is_file() {
                            files.push(path);
                        }
                    }
                    Err(_) => continue,
                };
            }
        } else {
            files.push(std::path::PathBuf::from(&filepath));
        }

        if files.is_empty() {
            error!("No valid files found. Only tsv/csv/txt files are supported.");
            std::process::exit(1);
        }

        for file in files {
            let filename = file.to_str().unwrap();
            info!("Importing {} into {}...", filename, table);

            let validation_errors = if table == "entity" {
                Entity::check_csv_is_valid(&file)
            } else if table == "entity2d" {
                Entity2D::check_csv_is_valid(&file)
            } else if table == "relation" {
                Relation::check_csv_is_valid(&file)
            } else if table == "knowledge_curation" {
                KnowledgeCuration::check_csv_is_valid(&file)
            } else if table == "subgraph" {
                Subgraph::check_csv_is_valid(&file)
            } else {
                error!("Invalid table name: {}", table);
                vec![]
            };

            if validation_errors.len() > 0 {
                error!("Invalid file: {}", filename);
                show_errors(&validation_errors, show_all_errors);
                warn!("Skipping {}...\n\n", filename);
                continue;
            } else {
                info!("{} is valid.", filename);
            }

            let delimiter = match get_delimiter(&file) {
                Ok(d) => d,
                Err(_) => {
                    error!("Invalid filename: {}, no extension found.", filename);
                    continue;
                }
            };

            let expected_columns = if table == "entity" {
                Entity::get_column_names(&file)
            } else if table == "entity2d" {
                Entity2D::get_column_names(&file)
            } else if table == "relation" {
                Relation::get_column_names(&file)
            } else if table == "knowledge_curation" {
                KnowledgeCuration::get_column_names(&file)
            } else if table == "subgraph" {
                Subgraph::get_column_names(&file)
            } else {
                error!("Invalid table name: {}", table);
                Ok(vec![])
            };

            let expected_columns = match expected_columns {
                Ok(v) => v,
                Err(e) => {
                    error!(
                        "Fn: get_column_names, Invalid file: {}, reason: {}",
                        filename, e
                    );
                    continue;
                }
            };

            debug!(
                "Expected columns which will be imported: {:?}",
                expected_columns
            );

            // Selecting process must be done after getting expected columns. because the temporary table is created based on the expected columns and it don't have extension. The get_column_names will fail if the file don't have extension.
            let pardir = file.parent().unwrap().to_path_buf();
            let extension = file.extension().unwrap().to_str();
            let temp_filepath = create_temp_file(&pardir, extension);
            debug!("Data file: {:?}, Temp file: {:?}", file, temp_filepath);

            let file = if table == "entity" {
                let results: Result<Vec<Entity>, Box<dyn Error>> =
                    Entity::select_expected_columns(&file, &temp_filepath);
                match results {
                    Ok(_) => temp_filepath,
                    Err(e) => {
                        error!(
                            "Fn: select_expected_columns, Invalid file: {}, reason: {}",
                            filename, e
                        );
                        continue;
                    }
                }
            } else if table == "entity2d" {
                let results: Result<Vec<Entity2D>, Box<dyn Error>> =
                    Entity2D::select_expected_columns(&file, &temp_filepath);
                match results {
                    Ok(_) => temp_filepath,
                    Err(e) => {
                        error!(
                            "Fn: select_expected_columns, Invalid file: {}, reason: {}",
                            filename, e
                        );
                        continue;
                    }
                }
            } else if table == "relation" {
                let results: Result<Vec<Relation>, Box<dyn Error>> =
                    Relation::select_expected_columns(&file, &temp_filepath);
                match results {
                    Ok(_) => temp_filepath,
                    Err(e) => {
                        error!(
                            "Fn: select_expected_columns, Invalid file: {}, reason: {}",
                            filename, e
                        );
                        continue;
                    }
                }
            } else if table == "knowledge_curation" {
                let results: Result<Vec<KnowledgeCuration>, Box<dyn Error>> =
                    KnowledgeCuration::select_expected_columns(&file, &temp_filepath);
                match results {
                    Ok(_) => temp_filepath,
                    Err(e) => {
                        error!(
                            "Fn: select_expected_columns, Invalid file: {}, reason: {}",
                            filename, e
                        );
                        continue;
                    }
                }
            } else if table == "subgraph" {
                let results: Result<Vec<Subgraph>, Box<dyn Error>> =
                    Subgraph::select_expected_columns(&file, &temp_filepath);
                match results {
                    Ok(_) => temp_filepath,
                    Err(e) => {
                        error!(
                            "Fn: select_expected_columns, Invalid file: {}, reason: {}",
                            filename, e
                        );
                        continue;
                    }
                }
            } else {
                error!("Invalid table name: {}", table);
                continue;
            };

            match table {
                "entity" => {
                    if !skip_check {
                        if file.exists() {
                            // To ensure ids in the biomedgps_knowledge_curation table are in the data file, elsewise we cannot use the biomedgps_knowledge_curation table correctly.
                            check_curated_knowledges(&pool, &file, delimiter).await;
                        } else {
                            error!("The file {} doesn't exist.", file.display());
                            return;
                        }
                    }

                    let table_name = "biomedgps_entity";
                    if drop {
                        drop_table(&pool, table_name).await;
                    };

                    import_file_in_loop(
                        &pool,
                        &file,
                        table_name,
                        &expected_columns,
                        &Entity::unique_fields(),
                        delimiter,
                    )
                    .await
                    .expect("Failed to import data into the biomedgps_entity table.");
                }
                "relation" => {
                    let table_name = "biomedgps_relation";
                    if drop {
                        drop_table(&pool, table_name).await;
                    };

                    import_file_in_loop(
                        &pool,
                        &file,
                        table_name,
                        &expected_columns,
                        &Relation::unique_fields(),
                        delimiter,
                    )
                    .await
                    .expect("Failed to import data into the biomedgps_relation table.");
                }
                "entity2d" => {
                    let table_name = "biomedgps_entity2d";
                    if drop {
                        drop_table(&pool, table_name).await;
                    };

                    import_file_in_loop(
                        &pool,
                        &file,
                        table_name,
                        &expected_columns,
                        &Entity2D::unique_fields(),
                        delimiter,
                    )
                    .await
                    .expect("Failed to import data into the biomedgps_entity2d table.");
                }
                "knowledge_curation" => {
                    let table_name = "biomedgps_knowledge_curation";
                    if drop {
                        drop_table(&pool, table_name).await;
                    };

                    import_file_in_loop(
                        &pool,
                        &file,
                        table_name,
                        &expected_columns,
                        &KnowledgeCuration::unique_fields(),
                        delimiter,
                    )
                    .await
                    .expect("Failed to import data into the biomedgps_knowledge_curation table.");
                }
                "subgraph" => {
                    let table_name = "biomedgps_subgraph";
                    if drop {
                        drop_table(&pool, table_name).await;
                    };

                    import_file_in_loop(
                        &pool,
                        &file,
                        table_name,
                        &expected_columns,
                        &Subgraph::unique_fields(),
                        delimiter,
                    )
                    .await
                    .expect("Failed to import data into the biomedgps_subgraph table.");
                }
                _ => {
                    error!("Unsupported table name: {}", table);
                    return;
                }
            };

            info!("{} imported.\n\n", filename);
        }
    }
}

// Setup the test database
pub async fn setup_test_db() -> sqlx::PgPool {
    // Get the database url from the environment variable
    let database_url = match std::env::var("DATABASE_URL") {
        Ok(v) => v,
        Err(_) => {
            println!("{}", "DATABASE_URL is not set.");
            std::process::exit(1);
        }
    };
    let pool = sqlx::PgPool::connect(&database_url).await.unwrap();

    return pool;
}

pub fn jsonstr2urlstr(json_str: &str) -> String {
    // // This is your JSON string.
    // let json_str = r#"{
    //     "key1": "value1",
    //     "key2": "value2"
    // }"#;

    // Parse the JSON string into a serde_json::Value.
    let v: Value = serde_json::from_str(json_str).expect("Failed to parse JSON");

    // Convert the Value into a HashMap.
    let map: HashMap<String, String> = v
        .as_object()
        .expect("Expected JSON to be an Object")
        .iter()
        .map(|(k, v)| {
            (
                k.clone(),
                v.as_str()
                    .expect("Expected value to be a String")
                    .to_string(),
            )
        })
        .collect();

    // Convert the HashMap into a URL-encoded string.
    let encoded: String = form_urlencoded::Serializer::new(String::new())
        .extend_pairs(map)
        .finish();

    return encoded;
}

pub fn kv2urlstr(key: &str, value: &str) -> String {
    let encoded: String = form_urlencoded::Serializer::new(String::new())
        .append_pair(key, value)
        .finish();

    return encoded;
}

pub fn init_logger(tag_name: &str, level: LevelFilter) -> Result<log4rs::Handle, String> {
    let stdout = ConsoleAppender::builder()
        .encoder(Box::new(PatternEncoder::new(
            &(format!("[{}]", tag_name) + " {d} - {h({l} - {t} - {m}{n})}"),
        )))
        .build();

    let config = Config::builder()
        .appender(Appender::builder().build("stdout", Box::new(stdout)))
        .logger(
            Logger::builder()
                .appender("stdout")
                .additive(false)
                .build("stdout", level),
        )
        .build(Root::builder().appender("stdout").build(level))
        .unwrap();

    log4rs::init_config(config).map_err(|e| {
        format!(
            "couldn't initialize log configuration. Reason: {}",
            e.description()
        )
    })
}

pub fn parse_db_url(db_url: &str) -> (String, String, String, String) {
    let url = url::Url::parse(db_url).unwrap();
    let host = url.host_str().unwrap().to_string();
    let port = url.port().unwrap().to_string();
    let username = url.username().to_string();
    let password = url.password().unwrap().to_string();

    return (host, port, username, password);
}
