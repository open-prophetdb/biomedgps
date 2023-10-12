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
use polars::prelude::{col, lit, CsvReader, IntoLazy, SerReader};
use std::error::Error;
use std::vec;

use crate::model::core::{
    CheckData, Entity, Entity2D, EntityEmbedding, KnowledgeCuration, Relation, RelationEmbedding,
    Subgraph,
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

pub async fn import_data(
    database_url: &str,
    filepath: &Option<String>,
    table: &str,
    drop: bool,
    show_all_errors: bool,
) {
    let pool = sqlx::postgres::PgPoolOptions::new()
        .connect(&database_url)
        .await
        .unwrap();

    if table == "relation_metadata" {
        update_relation_metadata(&pool, true).await.unwrap();
        return;
    } else if table == "entity_metadata" {
        update_entity_metadata(&pool, true).await.unwrap();
        return;
    }

    let filepath = match filepath {
        Some(f) => f,
        None => {
            error!("Please specify the file path.");
            return;
        }
    };
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
            let pardir = file.parent().unwrap();
            let temp_file = tempfile::NamedTempFile::new_in(pardir).unwrap();
            let temp_filepath = PathBuf::from(temp_file.path().to_str().unwrap());
            debug!("Data file: {:?}, Temp file: {:?}", file, temp_filepath);
            let results = if table == "entity" {
                Entity::select_expected_columns(&file, &temp_filepath)
            } else if table == "entity2d" {
                Entity2D::select_expected_columns(&file, &temp_filepath)
            } else if table == "relation" {
                Relation::select_expected_columns(&file, &temp_filepath)
            } else if table == "knowledge_curation" {
                KnowledgeCuration::select_expected_columns(&file, &temp_filepath)
            } else if table == "subgraph" {
                Subgraph::select_expected_columns(&file, &temp_filepath)
            } else {
                error!("Invalid table name: {}", table);
                continue;
            };

            let file = match results {
                Ok(_) => temp_filepath,
                Err(e) => {
                    error!(
                        "Fn: select_expected_columns, Invalid file: {}, reason: {}",
                        filename, e
                    );
                    continue;
                }
            };

            match table {
                "entity" => {
                    if file.exists() {
                        // To ensure ids in the biomedgps_knowledge_curation table are in the data file, elsewise we cannot use the biomedgps_knowledge_curation table correctly.
                        check_curated_knowledges(&pool, &file, delimiter).await;
                    } else {
                        error!("The file {} doesn't exist.", file.display());
                        return;
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
