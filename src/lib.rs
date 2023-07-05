//! BioMedGPS library for knowledge graph construction and analysis.

pub mod algorithm;
pub mod api;
pub mod model;
pub mod query_builder;
pub mod pgvector;

use crate::model::core::{
    CheckData, Entity, Entity2D, EntityEmbedding, KnowledgeCuration, Relation, RelationEmbedding,
    Subgraph,
};

use crate::model::util::{
    drop_table, get_delimiter, import_file_in_loop, show_errors, update_entity_metadata,
    update_relation_metadata,
};

use log::{error, info, warn, debug};
use sqlx::migrate::Migrator;
use std::fs::File;
use std::io::Write;
use std::path::Path;
use std::path::PathBuf;
use tempfile::tempdir;

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

pub async fn import_data(
    database_url: &str,
    filepath: &str,
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
    } else if table == "entity_embedding" || table == "relation_embedding" {
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
                    error!("Invalid file: {}, reason: {}", filename, e);
                    continue;
                }
            };

            // Selecting process must be done after getting expected columns. because the temporary table is created based on the expected columns and it don't have extension. The get_column_names will fail if the file don't have extension.
            let temp_file = tempfile::NamedTempFile::new().unwrap();
            let temp_filepath = PathBuf::from(temp_file.path().to_str().unwrap());
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
                    error!("Invalid file: {}, reason: {}", filename, e);
                    continue;
                }
            };

            match table {
                "entity" => {
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

// Init log for each test
pub fn init_log() {
    let _ = stderrlog::new()
        .module(module_path!())
        .module("biomedgps")
        .verbosity(5)
        .timestamp(stderrlog::Timestamp::Second)
        .init();
}
