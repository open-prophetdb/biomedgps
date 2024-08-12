extern crate log;

use biomedgps::model::entity::compound::CompoundAttr;
use biomedgps::model::init_db::create_kg_score_table;
use biomedgps::model::kge::{init_kge_models, DEFAULT_MODEL_NAME};
use biomedgps::model::{
    init_db::{
        create_score_table, get_kg_score_table_name, kg_entity_table2graphdb,
        kg_score_table2graphdb,
    },
    util::read_annotation_file,
};
use biomedgps::{
    build_index, change_emb_dimension, connect_graph_db, import_data, import_kge, init_logger,
    run_migrations,
};
use log::*;
use regex::Regex;
use sqlx::Row;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use structopt::StructOpt;

/// NOTE: In the first time, you need to follow the order to run the commands: initdb -> importdb (entity + entity_metadata + relation + relation_metadata etc.) -> importkge (embeddings) -> cachetable (compound-disease-symptom, gene-disease-symptom, knowledge-score). In the current stage, we don't have a mechanism to check the format of entity ids and relation_types and keep the consistent of the data, such as whether all entities in the relation table exist in the entity table. But we provide a script for this purpose, you can follow this link to check the data consistency: https://github.com/open-prophetdb/biomedgps-data/blob/main/graph_data/scripts/correct_graph_data.py
///
#[derive(StructOpt, Debug)]
#[structopt(setting=structopt::clap::AppSettings::ColoredHelp, name = "A cli for biomedgps service.", author="Jingcheng Yang <yjcyxky@163.com>;")]
struct Opt {
    /// Activate debug mode
    /// short and long flags (--debug) will be deduced from the field's name
    #[structopt(name = "debug", long = "debug")]
    debug: bool,

    #[structopt(subcommand)]
    cmd: SubCommands,
}

#[derive(Debug, PartialEq, StructOpt)]
enum SubCommands {
    #[structopt(name = "initdb")]
    InitDB(InitDbArguments),
    #[structopt(name = "importdb")]
    ImportDB(ImportDBArguments),
    #[structopt(name = "importkge")]
    ImportKGE(ImportKGEArguments),
    #[structopt(name = "cachetable")]
    CacheTable(CacheTableArguments),
    #[structopt(name = "cleandb")]
    CleanDB(CleanDBArguments),
    #[structopt(name = "statdb")]
    StatDB(StatDBArguments),
}

/// Initialize the database, only for the postgres database. In common, we don't need to initialize the graph database, such as neo4j. We can clean the graph database by the cleandb command simply before we import the data. We might need to run the initdb command when we want to upgrade the database schema or the first time we run the application.
#[derive(StructOpt, PartialEq, Debug)]
#[structopt(setting=structopt::clap::AppSettings::ColoredHelp, name="BioMedGPS - initdb", author="Jingcheng Yang <yjcyxky@163.com>")]
pub struct InitDbArguments {
    /// Database url, such as postgres://postgres:postgres@localhost:5432/rnmpdb, if not set, use the value of environment variable DATABASE_URL.
    #[structopt(name = "database_url", short = "d", long = "database-url")]
    database_url: Option<String>,
}

/// Output the statistics of the database, such as the number of entities, relations, metadata etc.
/// The statistics include the number of entities, relations, metadata, subgraph, knowledge_curation, entity2d, compound-disease-symptom, gene-disease-symptom, knowledge-score, embedding, graph etc.
#[derive(StructOpt, PartialEq, Debug)]
#[structopt(setting=structopt::clap::AppSettings::ColoredHelp, name="BioMedGPS - statdb", author="Jingcheng Yang <yjcyxky@163.com>")]
pub struct StatDBArguments {
    /// Database url, such as postgres://postgres:postgres@localhost:5432/rnmpdb, if not set, use the value of environment variable DATABASE_URL.
    #[structopt(name = "database_url", short = "d", long = "database-url")]
    database_url: Option<String>,
}

/// Clean the database, if you want to clean any table in the database, you can use this command.
#[derive(StructOpt, PartialEq, Debug)]
#[structopt(setting=structopt::clap::AppSettings::ColoredHelp, name="BioMedGPS - cleandb", author="Jingcheng Yang <yjcyxky@163.com>")]
pub struct CleanDBArguments {
    /// Database url, such as postgres://postgres:postgres@localhost:5432/rnmpdb or neo4j://<username>:<password>@localhost:7687, if not set, use the value of environment variable DATABASE_URL or NEO4J_URL.
    #[structopt(name = "database_url", short = "d", long = "database-url")]
    database_url: Option<String>,

    /// [Required] The table name to clean. e.g We will empty all entity-related tables if you use the entity table name. such as entity, entity_metadata, entity2d.
    #[structopt(name = "table", short = "t", long = "table", possible_values = &["entity", "relation", "embedding", "subgraph", "curation", "score", "message", "metadata"], multiple = true)]
    table: Vec<String>,
}

/// Import data files into database, such as entity, relation, entity_metadata, relation_metadata, knowledge_curation, subgraph, entity2d etc. When you import the entity data, we will also sync the entity data to the graph database. But the relation data will be synced to the graph database in the cachetable command, because we need to compute the score for the relation data first. The entity_metadata and relation_metadata are generated by the importdb command automatically, actually, you don't need to prepare the entity_metadata and relation_metadata files. But you must use the importdb command manually to upgrade the entity_metadata and relation_metadata tables after the entity and relation tables are upgraded or the first time you run the application. In the most cases, you don't need to import knowledge_curation and subgraph data, we might import them at the migration stage. The entity_2d table is used to store the 2D embedding data, you need to prepare the 2D embedding data manually. If you have multiple models, you might need to choose one model to compute the 2D embedding data. The 2D embedding data is used to visualize the entity data in the 2D space.
#[derive(StructOpt, PartialEq, Debug)]
#[structopt(setting=structopt::clap::AppSettings::ColoredHelp, name="BioMedGPS - importdb", author="Jingcheng Yang <yjcyxky@163.com>")]
pub struct ImportDBArguments {
    /// [Required] Database url, such as postgres://postgres:postgres@localhost:5432/rnmpdb, if not set, use the value of environment variable DATABASE_URL.
    #[structopt(name = "database_url", short = "d", long = "database-url")]
    database_url: Option<String>,

    /// [Optional] Database url, such as neo4j://<username>:<password>@localhost:7687, if not set, use the value of environment variable NEO4J_URL. If you don't want to sync the data to the graph database, you can skip this option.
    #[structopt(name = "neo4j_url", short = "n", long = "neo4j-url")]
    neo4j_url: Option<String>,

    /// [Optional] Database host, such as postgres-ml:5432. Only needed when you run your application in a docker container and the database is in another container.
    #[structopt(name = "db_host", short = "D", long = "db-host")]
    db_host: Option<String>,

    /// [Required] The file path of the data file to import. It may be a file or a directory. If you have multiple files to import, you can use the --filepath option with a directory path. We will import all files in the directory. But you need to disable the --drop option, otherwise, only the last file will be imported successfully.
    ///
    /// In the case of entity, the file should be a csv/tsv file which contains the id, name, label etc. More details about the format can be found in the github.com/yjcyxky/biomedgps-data.
    ///
    /// In the case of relation, the file should be a csv/tsv file which contains the source_id, source_type, relation_type, target_id, target_type etc. More details about the format can be found in the github.com/yjcyxky/biomedgps-data.
    ///
    /// In the case of entity_metadata, the file is not required.
    ///
    /// In the case of relation_metadata, the file should be a csv/tsv file which contains the relation_type, description.
    ///
    /// In the case of knowledge_curation, the file should be a csv/tsv file which contains the source_id, source_type, relation_type, target_id, target_type, description etc.
    ///
    /// In the case of subgraph, the file should be a json file which contains the subgraph data.
    #[structopt(name = "filepath", short = "f", long = "filepath")]
    filepath: Option<String>,

    /// [Optional] Annotation file path. This option is only required for relation table. It is used to annotate relation_type or other attributes. In current version, it is only used for relation_type.
    ///
    /// The annotation file is a csv/tsv file which contains two columns: relation_type and formatted_relation_type. e.g. relation_type,formatted_relation_type. If you don't want to format the relation_type, you can use the same value for the two columns.
    ///
    /// NOTE: You must ensure that the relation_type in the annotation file is consistent with the relation_type in the relation and relation_embedding files. If not, the import might fail or the relation_type will not be annotated.
    #[structopt(name = "annotation_file", short = "a", long = "annotation-file")]
    annotation_file: Option<String>,

    /// [Required] The table name to import data into. supports entity, entity2d, relation, relation_metadata, entity_metadata, knowledge_curation, subgraph, compound_metadata. Please note that we don't check whether the entities in other tables, such as entity2d, relation, knowledge etc. exist in the entity table. So you need to make sure that.
    ///
    /// In addition, if you upgrade the entity and relation tables, you need to ensure that the entity2d, relation_metadata, entity_metadata, knowledge_curation, subgraph tables are also upgraded. For the entity_metadata and relation_metadata, you can use the importdb command to upgrade after the entity and relation tables are upgraded.
    ///
    /// The order of the tables to import is: entity, relation, entity_metadata, relation_metadata, knowledge_curation [Optional], subgraph [Optional], entity2d [Optional], compound_metadata[Optional].
    #[structopt(name = "table", short = "t", long = "table", possible_values = &["entity", "entity2d", "relation", "relation_metadata", "entity_metadata", "knowledge_curation", "subgraph", "compound_metadata"])]
    table: String,

    /// [Optional] Drop the table before import data. If you have multiple files to import, don't use this option. If you use this option, only the last file will be imported successfully.
    #[structopt(name = "drop", long = "drop")]
    drop: bool,

    /// [Optional] Don't check other related tables in the database. Such as knowledge_curation which might be related to entity.
    #[structopt(name = "skip_check", short = "s", long = "skip-check")]
    skip_check: bool,

    /// [Optional] Which dataset is the data from. We assume that you have split the data into different datasets. If not, you can treat all data as one dataset. e.g. biomedgps. This feature is used to distinguish different dataset combinations matched with your model. It is only required for relation table.
    #[structopt(name = "dataset", long = "dataset")]
    dataset: Option<String>,

    /// [Optional] Show the first 3 errors when import data.
    #[structopt(name = "show_all_errors", short = "e", long = "show-all-errors")]
    show_all_errors: bool,

    /// [Optional] The batch size for syncing data to the graph database.
    #[structopt(
        name = "batch_size",
        short = "b",
        long = "batch-size",
        default_value = "10000"
    )]
    batch_size: usize,
}

/// Cache tables for performance. You must run this command after the importdb command.
#[derive(StructOpt, PartialEq, Debug)]
#[structopt(setting=structopt::clap::AppSettings::ColoredHelp, name="BioMedGPS - cachetable", author="Jingcheng Yang <yjcyxky@163.com>")]
pub struct CacheTableArguments {
    /// [Required] Database url, such as postgres://postgres:postgres@localhost:5432/rnmpdb, if not set, use the value of environment variable DATABASE_URL.
    #[structopt(name = "database_url", short = "d", long = "database-url")]
    database_url: Option<String>,

    /// [Optional] Database url, such as neo4j://<username>:<password>@localhost:7687, if not set, use the value of environment variable NEO4J_URL.
    #[structopt(name = "neo4j_url", short = "n", long = "neo4j-url")]
    neo4j_url: Option<String>,

    /// [Optional] Database host, such as postgres-ml:5432. Only needed when you run your application in a docker container and the database is in another container.
    #[structopt(name = "db_host", short = "D", long = "db-host")]
    db_host: Option<String>,

    /// [Required] The table name to init. supports compound-disease-symptom, gene-disease-symptom, knowledge-score etc.
    #[structopt(name = "table", short = "t", long = "table")]
    table: String,

    /// [Optional] Relation types for compound-disease-symptom table. Separated by comma. e.g. DRUGBANK::treats::Compound:Disease,HSDN::has_symptom::Disease:Symptom. The number and order of relation types should be consistent with the pairs of table name. e.g. compound-disease-symptom table should have two relation types for compound-disease and disease-symptom.
    #[structopt(name = "relation_types", short = "r", long = "relation-types")]
    relation_types: Option<String>,

    /// [Optional] The table_prefix which is used to distinguish different models. e.g. biomedgps, mecfs, etc. This feature is used to distinguish different dataset combinations matched with your model. If you want to generate the score table for the KGE model, you need to specify the table_prefix which is consistent with the table name related to the KGE model.
    #[structopt(
        name = "table_prefix",
        short = "T",
        long = "table_prefix",
        default_value = DEFAULT_MODEL_NAME
    )]
    table_prefix: String,

    /// [Optional] The batch size for caching table.
    #[structopt(
        name = "batch_size",
        short = "b",
        long = "batch-size",
        default_value = "10000"
    )]
    batch_size: usize,
}

/// Import embedding files into a database. The embedding files are generated by KGE models. If you have multiple models for different cases or datasets, you need to import them all and with different parameters, such as table_name, model_type, dataset, description etc. More details about these parameters can be found in their descriptions.
#[derive(StructOpt, PartialEq, Debug)]
#[structopt(setting=structopt::clap::AppSettings::ColoredHelp, name="BioMedGPS - importkge", author="Jingcheng Yang <yjcyxky@163.com>")]
pub struct ImportKGEArguments {
    /// [Optional] Database url, such as postgres://postgres:postgres@localhost:5432/rnmpdb, if not set, use the value of environment variable DATABASE_URL.
    #[structopt(name = "database_url", short = "d", long = "database-url")]
    database_url: Option<String>,

    /// [Required] The file path of the data file to import. It must be a csv/tsv file which contains the embedding_id, entity_id, entity_type, entity_name, embedding columns. And the embedding column is a string which contains the embedding values separated by pipe and the dimension of the embedding should be 400 or other values. e.g. Gene::ENTREZ:6747,ENTREZ:6747,Gene,SSR3,0.1|0.2|0.3|0.4|0.5
    #[structopt(
        name = "entity_embedding_file",
        short = "e",
        long = "entity-embedding-file"
    )]
    entity_embedding_file: String,

    /// [Required] The file path of the data file to import. It must be a csv/tsv file which contains the embedding_id, relation_type, formatted_relation_type, embedding columns. And the embedding column is a string which contains the embedding values separated by pipe and the dimension of the embedding should be 400 or other values. e.g. 1,STRING::BINDING::Gene:Gene,STRING::BINDING::Gene:Gene,0.1|0.2|0.3|0.4|0.5
    #[structopt(
        name = "relation_embedding_file",
        short = "r",
        long = "relation-embedding-file"
    )]
    relation_embedding_file: String,

    /// [Required] The file path of the metadata file to import. It must be a json file which contains the metadata of the model. Such as the model name, model type, description, learning rate, batch size, epochs, etc. If you don't have the metadata file, you can use a empty json file. e.g. {}
    #[structopt(name = "metadata_file", short = "f", long = "metadata-file")]
    metadata_file: String,

    /// [Optional] The dataset name you want to name. e.g. biomedgps, mecfs, etc. This feature is used to distinguish different dataset combinations matched with your model. If not set, we will use the biomedgps as default. But in this case, the dimension of the embedding should be 400.
    #[structopt(
        name = "dataset_name",
        short = "t",
        long = "dataset-name",
        default_value = DEFAULT_MODEL_NAME
    )]
    table_name: String,

    /// [Required] The model type of generated embedding files. e.g. TransE_l1, TransE_l2, DistMult, ComplEx, etc. This feature is used to distinguish different models. Users can choose the model for their own purpose.
    #[structopt(name = "model_type", short = "M", long = "model-type", default_value = "TransE_l2", possible_values = &["TransE_l1", "TransE_l2", "TransH", "TransR", "TransD", "RotatE", "DistMult", "ComplEx"])]
    model_type: String,

    /// [Optional] The dimension of the embedding. The default value is 400. The dimension of the embedding should be 400 or other values, like 768, 1024 etc.
    #[structopt(name = "dimension", long = "dimension", default_value = "400")]
    dimension: usize,

    /// [Required] Which dataset is the data from. We assume that you have split the data into different datasets. If not, you can treat all data as one dataset. e.g. biomedgps. This feature is used to distinguish different dataset combinations matched with your model.
    ///
    /// If you have multiple datasets, you can use the --dataset option with multiple values. e.g. --dataset biomedgps --dataset mecfs
    ///
    /// Each dataset must be registered in the relation table by the importdb command. If not, the import might fail.
    #[structopt(name = "dataset", long = "dataset", multiple = true)]
    dataset: Vec<String>,

    /// [Optional] Description of the model.
    #[structopt(name = "description", long = "description")]
    description: Option<String>,

    /// [Optional] Drop the table before import data. If you have multiple files to import, don't use this option. If you use this option, only the last file will be imported successfully.
    #[structopt(name = "drop", short = "D", long = "drop")]
    drop: bool,

    /// [Optional] Don't check the validity of the data files.
    #[structopt(name = "skip_check", short = "s", long = "skip-check")]
    skip_check: bool,

    /// [Optional] Force to import the embedding files.
    #[structopt(name = "force", short = "F", long = "force")]
    force: bool,

    /// [Optional] Show the first 3 errors when import data.
    #[structopt(name = "show_all_errors", short = "E", long = "show-all-errors")]
    show_all_errors: bool,

    /// [Optional] Annotation file path. This option is only required for legacy relation_embedding file which only contains id, embedding columns.
    ///
    /// The annotation file is a csv/tsv file which contains two columns: relation_type and formatted_relation_type. e.g. relation_type,formatted_relation_type. If you don't want to format the relation_type, you can use the same value for the two columns.
    ///
    /// NOTE: You must ensure that the relation_type in the annotation file is consistent with the relation_type in the relation_embedding files. If not, the import might fail or the relation_type will not be annotated.
    #[structopt(name = "annotation_file", short = "a", long = "annotation-file")]
    annotation_file: Option<String>,
}

#[tokio::main]
async fn main() {
    let opt = Opt::from_args();

    let _ = if opt.debug {
        init_logger("biomedgps-cli", LevelFilter::Debug)
    } else {
        init_logger("biomedgps-cli", LevelFilter::Info)
    };

    match opt.cmd {
        SubCommands::InitDB(arguments) => {
            let database_url = arguments.database_url;

            let database_url = if database_url.is_none() {
                match std::env::var("DATABASE_URL") {
                    Ok(v) => v,
                    Err(_) => {
                        error!("{}", "DATABASE_URL is not set.");
                        std::process::exit(1);
                    }
                }
            } else {
                database_url.unwrap()
            };

            match run_migrations(&database_url).await {
                Ok(_) => info!("Init database successfully."),
                Err(e) => error!("Init database failed: {}", e),
            }
        }
        SubCommands::CacheTable(arguments) => {
            let database_url = arguments.database_url;

            let database_url = if database_url.is_none() {
                match std::env::var("DATABASE_URL") {
                    Ok(v) => v,
                    Err(_) => {
                        error!("{}", "DATABASE_URL is not set.");
                        std::process::exit(1);
                    }
                }
            } else {
                database_url.unwrap()
            };

            let pool = match sqlx::PgPool::connect(&database_url).await {
                Ok(v) => v,
                Err(e) => {
                    error!("Connect to database failed: {}", e);
                    std::process::exit(1);
                }
            };

            // Initialize KGE models.
            let _ = match init_kge_models(&pool).await {
                Ok(_) => {
                    debug!("Initialize KGE models successfully.");
                    Some(())
                }
                Err(err) => {
                    error!("Initialize KGE models failed, {}", err);
                    None
                }
            };

            match arguments.table.as_str() {
                "compound-disease-symptom" => {
                    let default_relation_types =
                        "DRUGBANK::treats::Compound:Disease,HSDN::has_symptom::Disease:Symptom";
                    let relation_types = arguments.relation_types.unwrap_or(
                        // TODO: the HSDN::has_symptom::Disease:Symptom is non-standard relation type. We need to change it to the standard format.
                        default_relation_types.to_string(),
                    );
                    let relation_types = relation_types.split(",").collect::<Vec<&str>>();

                    if relation_types.len() != 2 {
                        error!("The number of relation types should be 2 and the order should be consistent with the pairs of table name. e.g. compound-disease-symptom table should have two relation types for compound-disease and disease-symptom.");
                        std::process::exit(1);
                    }

                    let compound_disease_relation_type = relation_types.get(0).unwrap();
                    let disease_symptom_relation_type = relation_types.get(1).unwrap();

                    if !compound_disease_relation_type.contains("Compound:Disease") {
                        error!("The first relation type should be for compound-disease. e.g. DRUGBANK::treats::Compound:Disease");
                        std::process::exit(1);
                    }

                    if !disease_symptom_relation_type.contains("Disease:Symptom") {
                        error!("The second relation type should be for disease-symptom. e.g. HSDN::has_symptom::Disease:Symptom");
                        std::process::exit(1);
                    }

                    match create_score_table(
                        &pool,
                        "Compound",
                        "Disease",
                        "Symptom",
                        compound_disease_relation_type,
                        disease_symptom_relation_type,
                        Some(&arguments.table_prefix),
                    )
                    .await
                    {
                        Ok(_) => info!("Init compound-disease-symptom table successfully."),
                        Err(e) => error!("Init compound-disease-symptom table failed: {}", e),
                    }
                }
                "gene-disease-symptom" => {
                    let default_relation_types =
                        "GNBR::J::Gene:Disease,HSDN::has_symptom::Disease:Symptom";
                    let relation_types = arguments.relation_types.unwrap_or(
                        // TODO: the HSDN::has_symptom::Disease:Symptom is non-standard relation type. We need to change it to the standard format.
                        default_relation_types.to_string(),
                    );
                    let relation_types = relation_types.split(",").collect::<Vec<&str>>();

                    if relation_types.len() != 2 {
                        error!("The number of relation types should be 2 and the order should be consistent with the pairs of table name. e.g. gene-disease-symptom table should have two relation types for gene-disease and disease-symptom.");
                        std::process::exit(1);
                    }

                    let compound_disease_relation_type = relation_types.get(0).unwrap();
                    let disease_symptom_relation_type = relation_types.get(1).unwrap();

                    if !compound_disease_relation_type.contains("Gene:Disease") {
                        error!("The first relation type should be for compound-disease. e.g. GNBR::J::Gene:Disease.");
                        std::process::exit(1);
                    }

                    if !disease_symptom_relation_type.contains("Disease:Symptom") {
                        error!("The second relation type should be for disease-symptom. e.g. HSDN::has_symptom::Disease:Symptom");
                        std::process::exit(1);
                    }

                    match create_score_table(
                        &pool,
                        "Gene",
                        "Disease",
                        "Symptom",
                        compound_disease_relation_type,
                        disease_symptom_relation_type,
                        Some(&arguments.table_prefix),
                    )
                    .await
                    {
                        Ok(_) => info!("Init gene-disease-symptom table successfully."),
                        Err(e) => error!("Init gene-disease-symptom table failed: {}", e),
                    }
                }
                "knowledge-score" => {
                    let neo4j_url = if arguments.neo4j_url.is_none() {
                        match std::env::var("NEO4J_URL") {
                            Ok(v) => v,
                            Err(_) => {
                                error!("{}", "NEO4J_URL is not set.");
                                "".to_owned()
                            }
                        }
                    } else {
                        arguments.neo4j_url.unwrap()
                    };

                    match create_kg_score_table(&pool, Some(&arguments.table_prefix)).await {
                        Ok(_) => info!("Init kg score table successfully."),
                        Err(e) => error!("Init kg score table failed: {}", e),
                    }

                    if neo4j_url == "" {
                        error!("{}", "NEO4J_URL is not set, skip to import kg score table to graph database.");
                        std::process::exit(0);
                    } else {
                        let table_prefix = &arguments.table_prefix;
                        let table_name = get_kg_score_table_name(table_prefix);
                        let total = match sqlx::query(&format!(
                            "SELECT count(*) FROM {}",
                            table_name
                        ))
                        .fetch_one(&pool)
                        .await
                        {
                            Ok(row) => row.get::<i64, _>("count"),
                            Err(e) => {
                                error!(
                                    "Failed to get the total number of the records in the score table: {}",
                                    e
                                );
                                std::process::exit(1);
                            }
                        };
                        // Use the regex to replace the database host and port.
                        let re = Regex::new(r"(.*//.*?@)[^/]*(/.*)").unwrap();
                        let database_url = if arguments.db_host.is_none() {
                            database_url
                        } else {
                            let caps = re.captures(&database_url).unwrap();
                            let db_host = arguments.db_host.unwrap();
                            format!("{}{}{}", &caps[1], db_host, &caps[2])
                        };
                        let graph = Arc::new(connect_graph_db(&neo4j_url).await);
                        match kg_score_table2graphdb(
                            &database_url,
                            &graph,
                            Some(table_prefix),
                            total as usize,
                            arguments.batch_size,
                            false,
                        )
                        .await
                        {
                            Ok(_) => {
                                info!("Import kg score table to graph database successfully.")
                            }
                            Err(e) => {
                                error!("Import kg score table to graph database failed: {}", e)
                            }
                        }
                    }
                }
                _ => {
                    error!("The table name is not supported.");
                    std::process::exit(1);
                }
            }
        }
        SubCommands::ImportDB(arguments) => {
            let database_url = if arguments.database_url.is_none() {
                match std::env::var("DATABASE_URL") {
                    Ok(v) => v,
                    Err(_) => {
                        error!("{}", "DATABASE_URL is not set.");
                        std::process::exit(1);
                    }
                }
            } else {
                arguments.database_url.unwrap()
            };

            if arguments.table.is_empty() {
                error!("Please specify the table name.");
                return;
            };

            // The annotation file is essential for relation table. We need the formatted_relation_type to annotate the relation_type.
            let relation_type_mappings = if arguments.table == "relation" {
                if arguments.annotation_file.is_none() {
                    error!("Please specify the annotation file for annotating the relation_type. We expect the annotation file has two columns: relation_type and formatted_relation_type. If you don't want to format the relation_type, you can use the same value for the two columns.");
                    std::process::exit(1);
                } else {
                    let annotation_file = PathBuf::from(arguments.annotation_file.unwrap());
                    if !annotation_file.exists() {
                        error!("{} does not exist.", annotation_file.display());
                        std::process::exit(1);
                    };

                    // Read the annotation file into a hashmap.
                    let relation_type_mappings = match read_annotation_file(&annotation_file) {
                        Ok(v) => v,
                        Err(e) => {
                            error!("Read annotation file failed: {}", e);
                            std::process::exit(1);
                        }
                    };
                    Some(relation_type_mappings)
                }
            } else {
                None
            };

            if arguments.table == "compound_metadata" {
                let pool = match sqlx::PgPool::connect(&database_url).await {
                    Ok(v) => v,
                    Err(e) => {
                        error!("Connect to database failed: {}", e);
                        std::process::exit(1);
                    }
                };
                let filepath = PathBuf::from(arguments.filepath.as_ref().unwrap());
                match CompoundAttr::sync2db(&pool, &filepath, arguments.drop).await {
                    Ok(_) => info!("Import compound metadata successfully."),
                    Err(e) => error!("Import compound metadata failed: {}", e),
                }
            } else {
                import_data(
                    &database_url,
                    &arguments.filepath,
                    &arguments.table,
                    &arguments.dataset,
                    &relation_type_mappings,
                    arguments.drop,
                    arguments.skip_check,
                    arguments.show_all_errors,
                )
                .await;
            }

            // Sync the data to the graph database, we will sync the relations to the graph database in the cachetable command, because we need to compute the score for the relation data first.
            if arguments.table == "entity" {
                // Sync the entity data to the graph database. The relation data will be synced to the graph database in the cachetable command, because we need to compute the score for the relation data.
                let neo4j_url = if arguments.neo4j_url.is_none() {
                    match std::env::var("NEO4J_URL") {
                        Ok(v) => v,
                        Err(_) => {
                            error!(
                                "{}",
                                "NEO4J_URL is not set, so skip to sync the data to the graph database."
                            );
                            return ();
                        }
                    }
                } else {
                    arguments.neo4j_url.unwrap()
                };

                let pool = match sqlx::PgPool::connect(&database_url).await {
                    Ok(v) => v,
                    Err(e) => {
                        error!("Connect to database failed: {}", e);
                        std::process::exit(1);
                    }
                };
                let total = match sqlx::query("SELECT count(*) FROM biomedgps_entity")
                    .fetch_one(&pool)
                    .await
                {
                    Ok(row) => row.get::<i64, _>("count"),
                    Err(e) => {
                        error!(
                            "Failed to get the total number of the records in the score table: {}",
                            e
                        );
                        std::process::exit(1);
                    }
                };

                // Use the regex to replace the database host and port.
                let re = Regex::new(r"(.*//.*?@)[^/]*(/.*)").unwrap();
                let database_url = if arguments.db_host.is_none() {
                    database_url
                } else {
                    let caps = re.captures(&database_url).unwrap();
                    let db_host = arguments.db_host.unwrap();
                    format!("{}{}{}", &caps[1], db_host, &caps[2])
                };

                let graph = connect_graph_db(&neo4j_url).await;
                let graph = Arc::new(graph);

                info!("Build the index for the entity table before we sync the entity data to the graph database to improve the performance.");
                // We need to build the index for the entity table before we sync the entity data to the graph database to improve the performance.
                build_index(
                    &graph,
                    &arguments.filepath,
                    arguments.skip_check,
                    arguments.show_all_errors,
                )
                .await;

                match kg_entity_table2graphdb(
                    &database_url,
                    &graph,
                    total as usize,
                    arguments.batch_size,
                )
                .await
                {
                    Ok(_) => {
                        info!("Import kg score table to graph database successfully.")
                    }
                    Err(e) => {
                        error!("Import kg score table to graph database failed: {}", e)
                    }
                }

                info!("We have synced the entity data to the graph database, but the relation data will be synced to the graph database in the cachetable command, because we need to compute the score for the relation data. Before you run the cachetable command, you need to ensure that the entity, relation, and the embedding data has been imported into the database.");
            }
        }
        SubCommands::ImportKGE(arguments) => {
            let database_url = if arguments.database_url.is_none() {
                match std::env::var("DATABASE_URL") {
                    Ok(v) => v,
                    Err(_) => {
                        error!("{}", "DATABASE_URL is not set.");
                        std::process::exit(1);
                    }
                }
            } else {
                arguments.database_url.unwrap()
            };

            let entity_embedding_file = PathBuf::from(arguments.entity_embedding_file);
            let relation_embedding_file = PathBuf::from(arguments.relation_embedding_file);
            let metadata_file = PathBuf::from(arguments.metadata_file);
            let annotation_file = arguments.annotation_file.as_deref().map(PathBuf::from);

            for file in vec![
                &entity_embedding_file,
                &relation_embedding_file,
                &metadata_file,
            ] {
                if !file.exists() {
                    error!("{} does not exist.", file.display());
                    std::process::exit(1);
                }
            }

            let table_name = arguments.table_name;
            let model_type = arguments.model_type;
            let description = arguments.description;
            let datasets: Vec<&str> = arguments.dataset.iter().map(|s| s.as_str()).collect();
            let drop = arguments.drop;
            let skip_check = arguments.skip_check;
            let show_all_errors = arguments.show_all_errors;

            if table_name == "biomedgps" && arguments.dimension != 400 {
                if arguments.force {
                    warn!("The dimension of the embedding is not 400, but the table name is biomedgps. We will change the dimension of the embedding as you specified.");
                    match change_emb_dimension(
                        &database_url,
                        table_name.as_str(),
                        arguments.dimension,
                    )
                    .await
                    {
                        Ok(_) => {
                            info!("Change the dimension of the embedding successfully.");
                        }
                        Err(e) => {
                            error!("Change the dimension of the embedding failed: {}", e);
                            std::process::exit(1);
                        }
                    }
                } else {
                    warn!("The dimension of the embedding is not 400, but the table name is biomedgps. If you believe that the dimension is correct, you can run the command again with --force option. Then we will change the dimension of the embedding as you specified. If you do that, the table will be dropped and re-imported.");
                    std::process::exit(1);
                }
            }

            import_kge(
                &database_url,
                &table_name,
                &model_type,
                &datasets,
                description.as_deref(),
                &entity_embedding_file,
                &relation_embedding_file,
                &metadata_file,
                drop,
                skip_check,
                show_all_errors,
                &annotation_file,
            )
            .await
        }
        SubCommands::CleanDB(arguments) => {
            let database_url = if arguments.database_url.is_none() {
                match std::env::var("DATABASE_URL") {
                    Ok(v) => v,
                    Err(_) => {
                        error!("{}", "DATABASE_URL is not set.");
                        std::process::exit(1);
                    }
                }
            } else {
                arguments.database_url.unwrap()
            };

            let pool = match sqlx::PgPool::connect(&database_url).await {
                Ok(v) => v,
                Err(e) => {
                    error!("Connect to database failed: {}", e);
                    std::process::exit(1);
                }
            };

            let mut table_names_map = HashMap::<&str, Vec<&str>>::new();
            let pairs = vec![
                ("message", vec!["biomedgps_ai_message"]),
                ("score", vec!["biomedgps_compound_disease_symptom_score", "biomedgps_gene_disease_symptom_score", "biomedgps_relation_with_score"]),
                ("metadata", vec!["biomedgps_compound_metadata", "biomedgps_journal_metadata"]),
                ("entity", vec!["biomedgps_entity", "biomedgps_entity2d", "biomedgps_entity_metadata"]),
                ("relation", vec!["biomedgps_relation", "biomedgps_relation_metadata"]),
                ("embedding", vec!["biomedgps_entity_embedding", "biomedgps_relation_embedding", "biomedgps_embedding_metadata"]),
                ("subgraph", vec!["biomedgps_subgraph"]),
                ("curation", vec!["biomedgps_knowledge_curation"])
            ];

            for pair in pairs {
                table_names_map.insert(pair.0, pair.1);
            }


            let tables = arguments.table;
            for table in tables {
                let table_names = table_names_map.get(table.as_str());
                if table_names.is_none() {
                    error!("The table name is not supported.");
                    std::process::exit(1);
                }

                let table_names = table_names.unwrap();
                for table_name in table_names {
                    let sql = format!("TRUNCATE TABLE {}", table_name);
                    match sqlx::query(&sql).execute(&pool).await {
                        Ok(_) => info!("Clean the {} table successfully.", table_name),
                        Err(e) => error!("Clean the {} table failed: {}", table_name, e),
                    }
                }
            }
            
        }
        SubCommands::StatDB(arguments) => {
            let database_url = if arguments.database_url.is_none() {
                match std::env::var("DATABASE_URL") {
                    Ok(v) => v,
                    Err(_) => {
                        error!("{}", "DATABASE_URL is not set.");
                        std::process::exit(1);
                    }
                }
            } else {
                arguments.database_url.unwrap()
            };

            let pool = match sqlx::PgPool::connect(&database_url).await {
                Ok(v) => v,
                Err(e) => {
                    error!("Connect to database failed: {}", e);
                    std::process::exit(1);
                }
            };

            let mut all_tables = vec![];
            let sql =
                "SELECT table_name FROM information_schema.tables WHERE table_schema = 'public'";
            let rows = match sqlx::query(sql).fetch_all(&pool).await {
                Ok(v) => v,
                Err(e) => {
                    error!("Query tables failed: {}", e);
                    std::process::exit(1);
                }
            };

            for row in rows {
                let table_name: String = row.get("table_name");
                all_tables.push(table_name);
            }

            info!("All tables in the database: {:?}", all_tables);

            let mut counts = HashMap::new();

            for table in all_tables {
                let total = match sqlx::query(&format!("SELECT count(*) FROM {}", table))
                    .fetch_one(&pool)
                    .await
                {
                    Ok(row) => row.get::<i64, _>("count"),
                    Err(e) => {
                        error!(
                            "Failed to get the total number of the records in the {} table: {}",
                            table, e
                        );
                        std::process::exit(1);
                    }
                };

                counts.insert(table, total);
            }

            info!("The statistics of the database:");
            for (table, total) in counts {
                println!("\t{}: {}", table, total);
            }

            // TODO: We must change the following codes to match the updates, such as adding a new cache table.
            let table_name = "biomedgps_compound_disease_symptom_score";
            let total = match sqlx::query(&format!("SELECT count(*) FROM {}", table_name))
                .fetch_one(&pool)
                .await
            {
                Ok(row) => row.get::<i64, _>("count"),
                Err(e) => {
                    error!(
                        "Failed to get the total number of the records in the {} table: {}",
                        table_name, e
                    );
                    std::process::exit(1);
                }
            };

            if total > 0 {
                info!(
                    "The number of records in the {} table: {}",
                    table_name, total
                );
            } else {
                error!("The {} table is empty, but we need to cache the table for symptom-compound prediction.", table_name);
            }
        }
    }
}
