#![doc = include_str!("../README.md")]
//! BioMedGPS library for knowledge graph construction and analysis.

// You must change the DB_VERSION to match the version of the database the library is compatible with.
const DB_VERSION: &str = "2.8.3";

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
use model::core::{EntityAttribute, DEFAULT_DATASET_NAME};
use model::kge::{EmbeddingMetadata, DEFAULT_MODEL_TYPES};
use neo4rs::{query, ConfigBuilder, Graph, Query};
use polars::prelude::{
    col, lit, CsvReader, CsvWriter, IntoLazy, NamedFrom, SerReader, SerWriter, Series,
};
use regex::Regex;
use sqlx::postgres::PgPoolOptions;
use sqlx::Row;
use std::error::Error;
use std::fs::Permissions;
use std::os::unix::fs::PermissionsExt;
use std::vec;

use crate::model::core::{
    CheckData, Entity, Entity2D, KnowledgeCuration, Relation, RelationMetadata, Subgraph,
};
use crate::model::graph::{Edge, Node};
use crate::model::kge::{EntityEmbedding, LegacyRelationEmbedding, RelationEmbedding};
use crate::model::util::{
    drop_records, drop_table, get_delimiter, import_file_in_loop, show_errors,
    update_entity_metadata, update_relation_metadata,
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

/// Connect to the database and run the migrations.
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

    let pool = connect_db(database_url, 1).await;

    migrator.run(&pool).await?;

    // Don't forget to cleanup the temporary directory.
    dir.close()?;
    info!("Migrations finished.");

    Ok(())
}

/// Before updating the entity table, we need to check whether the curated knowledge ids are in the entity table. otherwise, we cannot use the curated knowledge table correctly.
///
/// # Arguments
/// - `pool`: The database connection pool.
/// - `entity_file`: The entity file.
/// - `delimiter`: The delimiter of the entity file.
pub async fn check_curated_knowledges(pool: &sqlx::PgPool, entity_file: &PathBuf, delimiter: u8) {
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

    // Load the entity file into a DataFrame.
    log::info!(
        "Loading the entity file ({}) into a DataFrame.",
        entity_file.display()
    );
    // How to set truncate_ragged_lines=true?

    let df = CsvReader::from_path(entity_file)
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

/// Render all entities into a set of queries for importing into a neo4j database.
///
/// # Arguments
/// - `records`: A vector of Entity.
/// - `check_exist`: Whether to check whether the entity exists in the database before importing.
///
/// # Returns
/// A vector of Query or an error.
async fn prepare_entity_queries(
    records: Vec<Entity>,
    check_exist: bool,
) -> Result<Vec<Query>, Box<dyn Error>> {
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

        let query_string = if check_exist {
            format!("MERGE (n:{} {{idx: $idx, id: $id, name: $name, resource: $resource, description: $description, taxid: $taxid, synonyms: $synonyms, xrefs: $xrefs}}) ON CREATE SET n.id = $id", label)
        } else {
            format!("CREATE (n:{} {{idx: $idx, id: $id, name: $name, resource: $resource, description: $description, taxid: $taxid, synonyms: $synonyms, xrefs: $xrefs}})", label)
        };

        let query = Query::new(query_string)
            // Such as Gene::ENTREZ:01
            .param("idx", Node::format_id(&label, &record.id))
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

/// Render all relations into a set of queries for importing into a neo4j database.
///
/// # Arguments
/// - `records`: A vector of Relation.
/// - `check_exist`: Whether to check whether the relation exists in the database before importing.
///
/// # Returns
/// A vector of Query or an error.
pub async fn prepare_relation_queries(
    records: Vec<Relation>,
    check_exist: bool,
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

        let dataset = match record.dataset {
            Some(t) => t,
            None => DEFAULT_DATASET_NAME.to_string(),
        };

        let query_string = if check_exist {
            format!(
                "MATCH (e1:{} {{idx: $source_idx}})
                 MATCH (e2:{} {{idx: $target_idx}})
                 MERGE (e1)-[r:`{}` {{resource: $resource, key_sentence: $key_sentence, pmids: $pmids, dataset: $dataset, idx: $relation_idx}}]->(e2)",
                record.source_type, record.target_type, label
            )
        } else {
            format!(
                "MATCH (e1:{} {{idx: $source_idx}})
                 MATCH (e2:{} {{idx: $target_idx}})
                 CREATE (e1)-[r:`{}` {{resource: $resource, key_sentence: $key_sentence, pmids: $pmids, dataset: $dataset, idx: $relation_idx}}]->(e2)",
                record.source_type, record.target_type, label
            )
        };

        let query = Query::new(query_string)
            .param(
                "source_idx",
                Node::format_id(&record.source_type, &record.source_id),
            )
            .param(
                "target_idx",
                Node::format_id(&record.target_type, &record.target_id),
            )
            .param("pmids", pmids)
            .param("resource", record.resource)
            .param("key_sentence", key_sentence)
            .param("dataset", dataset)
            .param(
                "relation_idx",
                Edge::format_id(&record.source_id, &label, &record.target_id),
            );

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
            MATCH (e:{} {{idx: $entity_idx}})
            SET e.external_db_name = $external_db_name
            SET e.external_id = $external_id
            SET e.external_url = $external_url
            SET e.description = $description
            ",
            label // Directly use the label here
        );
        let query = Query::new(query_string)
            .param("entity_idx", Node::format_id(&label, &id))
            .param("external_db_name", record.external_db_name)
            .param("external_id", record.external_id)
            .param("external_url", record.external_url)
            .param("description", record.description);

        queries.push(query);
    }

    Ok(queries)
}

pub async fn batch_insert(
    graph: &Graph,
    queries: Vec<Query>,
    batch_size: usize,
) -> Result<(), Box<dyn Error>> {
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
        .suffix(format!(".{}", extension).as_str()) // Replace '.ext' with the desired extension
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

pub async fn build_index_by_jdbc(database_url: &str, graphdb: &Graph) {
    let jdbc_url = database_url.replace("postgres://", "jdbc:postgresql://");
    info!("jdbc_url: {}", jdbc_url);

    let query_str = format!(
        r#"
            CALL apoc.load.jdbc(
                "{jdbc_url}",
                "SELECT label FROM biomedgps_entity GROUP BY label",
            ) YIELD row RETURN row
            CALL apoc.cypher.doIt('CREATE INDEX IF NOT EXISTS FOR (n:'+row.label+') ON (n.idx)', {{}}) YIELD value
            RETURN count(*)
        "#,
        jdbc_url = jdbc_url
    );

    match graphdb.execute(query(&query_str)).await {
        Ok(_) => {
            info!("Build indexes successfully.");
            return;
        }
        Err(e) => {
            error!("Failed to build indexes: ({})", e);
            return;
        }
    }
}

pub async fn build_index(
    graph: &Graph,
    filepath: &Option<String>,
    skip_check: bool,
    show_all_errors: bool,
) {
    let filepath = match filepath {
        Some(f) => f,
        None => {
            error!("Please specify the file path.");
            return;
        }
    };

    if std::path::Path::new(&filepath).is_file() {
        let file = PathBuf::from(filepath);

        if !skip_check {
            let validation_errors = Entity::check_csv_is_valid(&file);

            if validation_errors.len() > 0 {
                error!("Invalid file: {}", file.display());
                show_errors(&validation_errors, show_all_errors);
                warn!("Skipping {}...\n\n", file.display());
                return;
            } else {
                info!("{} is valid.", file.display());
            }
        }

        let records: Vec<Entity> = Entity::get_records(&file).unwrap();
        let entity_types = records
            .iter()
            .map(|r| r.label.clone())
            .collect::<Vec<String>>();
        let uniq_entity_types = entity_types
            .iter()
            .cloned()
            .collect::<std::collections::HashSet<String>>()
            .into_iter()
            .collect::<Vec<String>>();
        let mut queries = vec![];
        for entity_type in uniq_entity_types {
            let query_string = format!(
                "CREATE INDEX {} IF NOT EXISTS FOR (n:{}) ON (n.idx)",
                format!("biomedgps_{}_idx", entity_type.to_lowercase()),
                entity_type // Directly use the label here
            );
            let query = Query::new(query_string);
            queries.push(query);
        }

        match batch_insert(graph, queries, 1000).await {
            Ok(_) => {
                info!("Build indexes successfully.");
                return;
            }
            Err(e) => {
                error!("Failed to build indexes: ({})", e);
                return;
            }
        }
    } else {
        error!("Please specify the file path, not a directory.");
        return;
    }
}

pub async fn import_graph_data(
    graph: &Graph,
    filepath: &Option<String>,
    filetype: &str,
    skip_check: bool,
    check_exist: bool,
    show_all_errors: bool,
    batch_size: usize,
    dataset: &Option<String>,
) {
    if dataset.is_none() && filetype == "relation" {
        error!("Please specify the dataset name.");
        return;
    }

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
                EntityAttribute::check_csv_is_valid(&file)
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

        let queries = if filetype == "entity" {
            let records = Entity::get_records(&file).unwrap();
            prepare_entity_queries(records, check_exist).await.unwrap()
        } else if filetype == "relation" {
            let records = Relation::get_records(&file).unwrap();
            let records = records
                .iter()
                .map(|r: &Relation| {
                    let mut r = r.clone();
                    if let Some(d) = dataset {
                        r.dataset = Some(d.to_string());
                    }
                    r
                })
                .collect::<Vec<Relation>>();
            prepare_relation_queries(records, check_exist)
                .await
                .unwrap()
        } else if filetype == "entity_attribute" {
            let records = EntityAttribute::get_records(&file).unwrap();
            prepare_entity_attr_queries(records).await.unwrap()
        } else {
            error!("Invalid file type: {}", filetype);
            // Stop the program if the file type is invalid.
            std::process::exit(1);
        };

        if queries.len() == 0 {
            error!("No queries generated.");
            continue;
        } else {
            match batch_insert(graph, queries, batch_size).await {
                Ok(_) => {
                    info!("Import {} into neo4j successfully.", filename);

                    if filetype == "entity" {
                        // Build indexes for the entity nodes.
                        info!("Building indexes for the entity nodes.");
                        build_index(
                            graph,
                            &Some(filename.to_string()),
                            skip_check,
                            show_all_errors,
                        )
                        .await;
                    }

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

fn add_new_column(
    filepath: &str,
    column_name: &str,
    column_value: &Vec<&str>,
    delimiter: u8,
) -> Result<(), Box<dyn Error>> {
    // Add a new column named dataset into the temp file by using the polar crate.
    let mut df = CsvReader::from_path(&filepath)
        .unwrap()
        .with_delimiter(delimiter)
        .has_header(true)
        .finish()
        .unwrap();

    let columns = df.get_column_names();

    if columns.contains(&column_name) {
        warn!("The column {} already exists, skip adding it.", column_name);
        return Ok(());
    };

    let column_value = if column_value.len() == 1 {
        let column_value = column_value[0];
        vec![column_value; df.height()]
    } else {
        column_value.to_vec()
    };

    if column_value.len() != df.height() {
        let err_msg =
            "The length of the column value must be equal to the height of your data file.";
        return Err(err_msg.into());
    } else {
        let datasets = Series::new(column_name, column_value);
        df.with_column(datasets).unwrap();
    };

    let writer = File::create(&filepath).unwrap();
    CsvWriter::new(writer)
        .has_header(true)
        .with_delimiter(delimiter)
        .finish(&mut df)
        .unwrap();

    return Ok(());
}

pub async fn import_data(
    database_url: &str,
    filepath: &Option<String>,
    table: &str,
    dataset: &Option<String>,
    relation_type_mappings: &Option<HashMap<String, String>>,
    drop: bool,
    skip_check: bool,
    show_all_errors: bool,
) {
    let pool = connect_db(database_url, 10).await;

    // Don't need a file path for updating the entity_metadata table.
    if table == "entity_metadata" {
        update_entity_metadata(&pool, true).await.unwrap();
        return;
    }

    if dataset.is_none() && table == "relation" {
        error!("Please specify the dataset name. It is required for the relation table.");
        return;
    }

    let filepath = match filepath {
        Some(f) => f,
        None => {
            error!("Please specify the file path.");
            return;
        }
    };

    if table == "relation_metadata" {
        match update_relation_metadata(&pool, &PathBuf::from(filepath), true).await {
            Ok(_) => {
                info!("Relation metadata updated successfully.");
            }
            Err(e) => {
                error!("Failed to update relation metadata: ({})", e);
            }
        }
        return;
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

            let mut expected_columns = match expected_columns {
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
                    Ok(records) => {
                        // It must be done before importing the data into the database.
                        // The dataset must not be None here, because the dataset is required for the relation table and it is checked before.
                        if let Some(d) = dataset {
                            match add_new_column(
                                &temp_filepath.to_str().unwrap(),
                                "dataset",
                                &vec![d],
                                delimiter,
                            ) {
                                Ok(_) => {}
                                Err(e) => {
                                    error!(
                                        "Fn: add_new_column, NewColumn: dataset, Invalid file: {}, reason: {}",
                                        filename, e
                                    );
                                    continue;
                                }
                            }

                            if !expected_columns.contains(&"dataset".to_string()) {
                                expected_columns.push("dataset".to_string());
                            }
                        }

                        if relation_type_mappings.is_some() {
                            // Add the formatted_relation_type column if it doesn't exist. It must be done before importing the data into the database.
                            let relation_types = records
                                .iter()
                                .map(|r| r.relation_type.clone())
                                .collect::<Vec<String>>();

                            let formatted_relation_types = relation_types
                                .iter()
                                .map(|r| {
                                    let mut r = r.clone();
                                    if let Some(mappings) = relation_type_mappings {
                                        if mappings.contains_key(&r) {
                                            r = mappings.get(&r).unwrap().to_string();
                                        } else {
                                            warn!("The relation type {} is not in the relation_type_mappings, skip formatting it and use it directly.", r);
                                        }
                                    }
                                    r
                                })
                                .collect::<Vec<String>>();

                            debug!(
                                "The length of the relation types is {}.",
                                relation_types.len()
                            );
                            debug!(
                                "The length of the formatted relation types is {}.",
                                formatted_relation_types.len()
                            );

                            match add_new_column(
                                &temp_filepath.to_str().unwrap(),
                                "formatted_relation_type",
                                &formatted_relation_types
                                    .iter()
                                    .map(|v| v.as_str())
                                    .collect::<Vec<&str>>(),
                                delimiter,
                            ) {
                                Ok(_) => {}
                                Err(e) => {
                                    error!(
                                        "Fn: add_new_column, NewColumn: formatted_relation_type, Invalid file: {}, reason: {}",
                                        filename, e
                                    );
                                    continue;
                                }
                            }

                            if !expected_columns.contains(&"formatted_relation_type".to_string()) {
                                expected_columns.push("formatted_relation_type".to_string());
                            }

                            // TODO: The order of the source_type and target_type values might not be correct. Such as the source_type is "Gene" and the target_type is "Disease", but the formatted_relation_type is "Disease:Gene". We need to fix the order of the source_type and target_type values. Or warn that the order of the source_type and target_type values in the formatted_relation_type column is not correct.
                        }

                        temp_filepath
                    }
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
                        if dataset.is_none() {
                            drop_table(&pool, table_name).await;
                        } else {
                            // Only drop the relation table with the specified dataset.
                            let dataset = dataset.as_ref().unwrap();
                            drop_records(&pool, table_name, "dataset", dataset).await;
                        }
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
    let pool = connect_db(&database_url, 1).await;

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

pub fn is_db_url_valid(db_url: &str) -> bool {
    // check whether db url is valid. the db_url format is <postgres|neo4j>://<username>:<password>@<host>:<port>/database
    let regex_str = r"^(postgres|neo4j)://((.+):(.+)@)?(.+):(\d+)(/.+)?$";
    let is_valid = match Regex::new(regex_str) {
        Ok(r) => r.is_match(db_url),
        Err(_) => false,
    };

    return is_valid;
}

pub fn parse_db_url(db_url: &str) -> (String, String, String, String, String) {
    // Get host, username and password from db_url. the db_url format is postgres://<username>:<password>@<host>:<port>/database
    let url = url::Url::parse(db_url).unwrap();
    let host = match url.host_str() {
        Some(h) => h.to_string(),
        None => "".to_string(),
    };
    let port = match url.port() {
        Some(p) => p.to_string(),
        None => "".to_string(),
    };
    let username = url.username().to_string();
    let password = match url.password() {
        Some(p) => p.to_string(),
        None => "".to_string(),
    };
    let database = url.path().to_string().replace("/", "");

    return (host, port, username, password, database);
}

pub async fn connect_graph_db(neo4j_url: &str) -> Graph {
    if is_db_url_valid(neo4j_url) {
        debug!("Valid neo4j_url: {}", neo4j_url);
    } else {
        error!(
            "Invalid neo4j_url: {}, the format is neo4j://<username>:<password>@<host>:<port>",
            neo4j_url
        );
        std::process::exit(1);
    };

    // Get host, username and password from neo4j_url. the neo4j_url format is neo4j://<username>:<password>@<host>:<port>
    let mut host = "".to_string();
    let mut username = "".to_string();
    let mut password = "".to_string();
    let mut default_db_name = "neo4j".to_string(); // default db name is "neo4j
    if neo4j_url.starts_with("neo4j://") {
        let (hostname, port, user, pass, db_name) = parse_db_url(&neo4j_url);
        host = format!("{}:{}", hostname, port);
        username = user;
        password = pass;

        if !db_name.is_empty() {
            default_db_name = db_name;
        }
    } else {
        error!("Invalid neo4j_url: {}", neo4j_url);
        std::process::exit(1);
    };

    if host.is_empty() || username.is_empty() {
        debug!("Invalid neo4j_url: {}", neo4j_url);
        std::process::exit(1);
    };

    let graph = Graph::connect(
        ConfigBuilder::default()
            .uri(host)
            .user(username)
            .password(password)
            .db(default_db_name)
            .build()
            .unwrap(),
    )
    .await
    .unwrap();

    return graph;
}

pub async fn connect_db(database_url: &str, max_connections: u32) -> sqlx::PgPool {
    match is_db_url_valid(database_url) {
        true => (),
        false => {
            error!("Invalid database_url: {}, the format is postgres://<username>:<password>@<host>:<port>/<database>", database_url);
            std::process::exit(1);
        }
    };

    let pool = PgPoolOptions::new()
        .max_connections(max_connections)
        .idle_timeout(std::time::Duration::from_secs(600)) // 10 min
        .acquire_timeout(std::time::Duration::from_secs(30)) // 30 seconds
        .max_lifetime(std::time::Duration::from_secs(1800)) // 30 min
        .connect(&database_url)
        .await;

    match pool {
        Ok(p) => p,
        Err(e) => {
            error!("Failed to connect to the database: {}", e);
            std::process::exit(1);
        }
    }
}

pub async fn import_kge(
    database_url: &str,
    table_name: &str,
    model_name: &str,
    model_type: &str,
    datasets: &Vec<&str>,
    description: Option<&str>,
    entity_file: &PathBuf,
    relation_file: &PathBuf,
    metadata_file: &PathBuf,
    drop: bool,
    skip_check: bool,
    show_all_errors: bool,
    annotation_file: &Option<PathBuf>,
) {
    let pool = connect_db(database_url, 10).await;
    let default_datasets = match RelationMetadata::get_relation_metadata(&pool).await {
        Ok(r) => {
            let mut datasets = vec![];
            for record in r {
                datasets.push(record.dataset);
            }
            datasets
        }
        Err(e) => {
            error!("Failed to get the relation metadata: {}", e);
            std::process::exit(1);
        }
    };

    for dataset in datasets {
        if default_datasets.contains(&dataset.to_string()) {
            debug!("Valid dataset: {}", dataset);
        } else {
            error!(
                "Invalid dataset: {}, the valid datasets are {:?}. You can add the dataset into the relation_metadata table by using the importdb command. It means that you need to import at least one entity and one relation into the database before importing the KGE model if the valid datasets are empty. And then update the entity_metadata and relation_metadata tables by using the importdb command.",
                dataset, default_datasets
            );
            std::process::exit(1);
        };
    }

    if DEFAULT_MODEL_TYPES.contains(&model_type) {
        debug!("Valid model_type: {}", model_type);
    } else {
        error!(
            "Invalid model_type: {}, the valid model types are {:?}",
            model_type, DEFAULT_MODEL_TYPES
        );
        std::process::exit(1);
    };

    // Read the metadata file as a json string.
    let metadata = match std::fs::read_to_string(metadata_file) {
        Ok(f) => f,
        Err(e) => {
            error!("Failed to read the metadata file: {}", e);
            std::process::exit(1);
        }
    };

    // Detect the dimension of the entity embeddings.
    if skip_check {
        info!("Skip checking the entity file.");
    } else {
        let errors = EntityEmbedding::check_csv_is_valid(entity_file);
        if errors.len() > 0 {
            show_errors(&errors, show_all_errors);
            std::process::exit(1);
        } else {
            info!("{} is valid.", entity_file.display());
        }
    };

    let mut fileformat = "relation_embedding";
    if skip_check {
        info!("Skip checking the relation file.");
    } else {
        let errors = RelationEmbedding::check_csv_is_valid(relation_file);

        if errors.len() == 0 {
            // We prefer to use the new format of relation embedding. If the new format is valid, we will ignore the old format.
        } else {
            let warning_msg = "Your relation embedding file is not valid, the file should contain the following fields: relation_type, formatted_relation_type, embedding. We will try to use the old format of relation embedding. If the old format is valid, we will return the old format. Otherwise, the relation embedding file is invalid.";
            warn!("{}", warning_msg);
        }

        let final_errors = LegacyRelationEmbedding::check_csv_is_valid(relation_file);

        if final_errors.len() == 0 {
            // If the old format is valid, we will return the old format.
            fileformat = "legacy_relation_embedding";
            info!("{} is valid.", relation_file.display());
        } else {
            show_errors(&errors, show_all_errors);
            std::process::exit(1);
        };
    };

    info!("Detecting the dimension of the entity embeddings.");
    let records: Vec<EntityEmbedding> = match EntityEmbedding::get_records(entity_file) {
        Ok(r) => r,
        Err(e) => {
            error!("Failed to get records from the entity file: {}", e);
            std::process::exit(1);
        }
    };
    let dimension = records[0].embedding.to_vec().len();

    // Init the embedding tables.
    let description = match description {
        Some(d) => d.to_string(),
        None => format!(
            "The model is trained with the {} dataset and the model type is {}.",
            datasets.join(" + "),
            model_type
        ),
    };

    match EmbeddingMetadata::init_embedding_table(
        &pool,
        table_name,
        model_name,
        model_type,
        &description,
        datasets,
        dimension,
        Some(metadata),
    )
    .await
    {
        Ok(_) => {
            info!("Init the embedding tables successfully.");
            true
        }
        Err(e) => {
            if drop {
                info!("The embedding tables already exist, drop their records and reimport the embeddings.");
                true
            } else {
                error!("Failed to init the embedding tables: {}", e);
                std::process::exit(1);
            }
        }
    };

    let delimiter = match get_delimiter(relation_file) {
        Ok(d) => d,
        Err(e) => {
            error!(
                "Failed to get the delimiter of the {}: {}",
                relation_file.display(),
                e
            );
            std::process::exit(1);
        }
    };

    if fileformat == "relation_embedding" {
        // Import the relation embeddings.
        match RelationEmbedding::import_relation_embeddings(
            &pool,
            relation_file,
            delimiter,
            drop,
            Some(table_name),
        )
        .await
        {
            Ok(_) => {
                info!("Import the relation embeddings successfully.");
            }
            Err(e) => {
                error!("Failed to import the relation embeddings: {}", e);
                std::process::exit(1);
            }
        }
    } else {
        match LegacyRelationEmbedding::import_relation_embeddings(
            &pool,
            relation_file,
            annotation_file,
            drop,
            Some(table_name),
            delimiter,
        )
        .await
        {
            Ok(_) => {
                info!(
                    "Import the relation embeddings successfully. The relation embeddings are in the old format.",
                );
            }
            Err(e) => {
                error!("Failed to import the relation embeddings: {}", e);
                std::process::exit(1);
            }
        };
    }

    let delimiter = match get_delimiter(entity_file) {
        Ok(d) => d,
        Err(e) => {
            error!(
                "Failed to get the delimiter of the {}: {}",
                entity_file.display(),
                e
            );
            std::process::exit(1);
        }
    };
    // Import the entity embeddings.
    match EntityEmbedding::import_entity_embeddings(
        &pool,
        entity_file,
        delimiter,
        drop,
        Some(table_name),
    )
    .await
    {
        Ok(_) => {
            info!("Import the entity embeddings successfully.");
        }
        Err(e) => {
            error!("Failed to import the entity embeddings: {}", e);
            std::process::exit(1);
        }
    }
}

pub async fn check_db_version(pool: &sqlx::PgPool) -> Result<(), Box<dyn Error>> {
    // Check whether the pgml.version function exists.
    let sql_str = "
        SELECT
        CASE
            WHEN EXISTS(SELECT 1 FROM pg_extension WHERE extname = 'pgml') THEN
                -- If the pgml extension is enabled, return the version of the pgml.
                pgml.version()
            ELSE
                -- If the pgml extension is not enabled, return the error message.
                'Unknown'
        END AS version;
    ";

    let version = match sqlx::query(sql_str).fetch_one(pool).await {
        Ok(v) => v,
        Err(e) => {
            return Err(format!("Failed to get the database version: {}", e).into());
        }
    };

    let version: String = version.get("version");
    let version_num_str = version.split(" ").collect::<Vec<&str>>()[0].to_string();
    info!("The database version is: {}", version_num_str);

    if DB_VERSION >= &version_num_str[..] && version_num_str != "Unknown" {
        info!("The database version is compatible with the current version of the pgml.");
        return Ok(());
    } else {
        error!(
            "The database version is not compatible with the current version of the pgml. The database version is {}, but the current version of the pgml requires the database version to be {} or higher. If the database version is Unknown, it means that the pgml extension is not enabled or not installed.",
            version_num_str, DB_VERSION
        );
        std::process::exit(1);
    }
}
