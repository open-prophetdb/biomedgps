extern crate log;

use biomedgps::model::init_sql::create_kg_score_table;
use biomedgps::model::kge::{init_kge_models, DEFAULT_MODEL_NAME};
use biomedgps::model::{init_sql::create_score_table, util::read_annotation_file};
use biomedgps::{
    build_index, connect_graph_db, import_data, import_graph_data, import_kge, init_logger,
    run_migrations,
};
use log::*;
use std::path::PathBuf;
use structopt::StructOpt;

/// A cli for biomedgps service.
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
    #[structopt(name = "importgraph")]
    ImportGraph(ImportGraphArguments),
    #[structopt(name = "importkge")]
    ImportKGE(ImportKGEArguments),
    #[structopt(name = "inittable")]
    InitTable(InitTableArguments),
}

/// Init database.
#[derive(StructOpt, PartialEq, Debug)]
#[structopt(setting=structopt::clap::AppSettings::ColoredHelp, name="BioMedGPS - initdb", author="Jingcheng Yang <yjcyxky@163.com>")]
pub struct InitDbArguments {
    /// Database url, such as postgres://postgres:postgres@localhost:5432/rnmpdb, if not set, use the value of environment variable DATABASE_URL.
    #[structopt(name = "database_url", short = "d", long = "database-url")]
    database_url: Option<String>,
}

/// Import data files into database.
#[derive(StructOpt, PartialEq, Debug)]
#[structopt(setting=structopt::clap::AppSettings::ColoredHelp, name="BioMedGPS - importdb", author="Jingcheng Yang <yjcyxky@163.com>")]
pub struct ImportDBArguments {
    /// [Required] Database url, such as postgres://postgres:postgres@localhost:5432/rnmpdb, if not set, use the value of environment variable DATABASE_URL.
    #[structopt(name = "database_url", short = "d", long = "database-url")]
    database_url: Option<String>,

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

    /// [Required] The table name to import data into. supports entity, entity2d, relation, relation_metadata, entity_metadata, knowledge_curation, subgraph. Please note that we don't check whether the entities in other tables, such as entity2d, relation, knowledge etc. exist in the entity table. So you need to make sure that.
    ///
    /// In addition, if you upgrade the entity and relation tables, you need to ensure that the entity2d, relation_metadata, entity_metadata, knowledge_curation, subgraph tables are also upgraded. For the entity_metadata and relation_metadata, you can use the importdb command to upgrade after the entity and relation tables are upgraded.
    ///
    /// The order of the tables to import is: entity, relation, entity_metadata, relation_metadata, knowledge_curation [Optional], subgraph [Optional], entity2d [Optional].
    #[structopt(name = "table", short = "t", long = "table")]
    table: String,

    /// [Optional] Drop the table before import data. If you have multiple files to import, don't use this option. If you use this option, only the last file will be imported successfully.
    #[structopt(name = "drop", short = "D", long = "drop")]
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
}

/// Init tables for performance.
#[derive(StructOpt, PartialEq, Debug)]
#[structopt(setting=structopt::clap::AppSettings::ColoredHelp, name="BioMedGPS - inittable", author="Jingcheng Yang <yjcyxky@163.com>")]
pub struct InitTableArguments {
    /// [Required] Database url, such as postgres://postgres:postgres@localhost:5432/rnmpdb, if not set, use the value of environment variable DATABASE_URL.
    #[structopt(name = "database_url", short = "d", long = "database-url")]
    database_url: Option<String>,

    /// [Required] The table name to init. supports compound-disease-symptom, knowledge-score etc.
    #[structopt(name = "table", short = "t", long = "table")]
    table: String,

    /// [Optional] Relation types for compound-disease-symptom table. Separated by comma. e.g. STRING::BINDING::Gene:Gene,STRING::BINDING::Gene:Gene. The number and order of relation types should be consistent with the pairs of table name. e.g. compound-disease-symptom table should have two relation types for compound-disease and disease-symptom.
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
}

/// Import data files into a graph database.
#[derive(StructOpt, PartialEq, Debug)]
#[structopt(setting=structopt::clap::AppSettings::ColoredHelp, name="BioMedGPS - importgraph", author="Jingcheng Yang <yjcyxky@163.com>")]
pub struct ImportGraphArguments {
    /// [Required] Database url, such as neo4j://<username>:<password>@localhost:7687, if not set, use the value of environment variable NEO4J_URL.
    #[structopt(name = "neo4j_url", short = "n", long = "neo4j-url")]
    neo4j_url: Option<String>,

    /// [Required] The file path of the data file to import. It may be a file or a directory.
    #[structopt(name = "filepath", short = "f", long = "filepath")]
    filepath: Option<String>,

    /// [Required] The file type of the data file to import. It may be entity, relation, entity_attribute and relation_attribute.
    #[structopt(name = "filetype", short = "t", long = "filetype")]
    filetype: Option<String>,

    /// [Optional] Batch size for import data. Default is 1000.
    #[structopt(name = "batch_size", short = "b", long = "batch-size")]
    batch_size: Option<usize>,

    /// [Optional] Don't check other related tables in the database. Such as knowledge_curation which might be related to entity.
    #[structopt(name = "skip_check", short = "s", long = "skip-check")]
    skip_check: bool,

    /// [Optional] Check if the data exists in the database before import data.
    #[structopt(name = "check_exist", short = "c", long = "check-exist")]
    check_exist: bool,

    /// [Optional] Show the first 3 errors when import data.
    #[structopt(name = "show_all_errors", short = "e", long = "show-all-errors")]
    show_all_errors: bool,

    /// [Optional] Which dataset is the data from. We assume that you have split the data into different datasets. If not, you can treat all data as one dataset. e.g. biomedgps. This feature is used to distinguish different dataset combinations matched with your model.
    #[structopt(name = "dataset", short = "d", long = "dataset")]
    dataset: Option<String>,
}

/// Import embedding files into a database
/// The embedding files are generated by KGE models.
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
    #[structopt(name = "metadata_file", short = "m", long = "metadata-file")]
    metadata_file: String,

    /// [Optional] The table name you want to name. e.g. biomedgps, mecfs, etc. This feature is used to distinguish different dataset combinations matched with your model. If not set, we will use the biomedgps as default. But in this case, the dimension of the embedding should be 400.
    #[structopt(
        name = "table_name",
        short = "t",
        long = "table-name",
        default_value = DEFAULT_MODEL_NAME
    )]
    table_name: String,

    /// [Optional] The model name you want to name. e.g. mecfs_transe, mecfs_distmult, etc. You need to specify the model name when you import the embedding files. This feature is used to distinguish different models. Users can choose the model for their own purpose.
    #[structopt(
        name = "model_name", 
        short = "m", 
        long = "model-name", 
        default_value = DEFAULT_MODEL_NAME
    )]
    model_name: String,

    /// [Required] The model type of generated embedding files. e.g. TransE_l1, TransE_l2, DistMult, ComplEx, etc. This feature is used to distinguish different models. Users can choose the model for their own purpose.
    #[structopt(name = "model_type", short = "M", long = "model-type")]
    model_type: String,

    /// [Required] Which dataset is the data from. We assume that you have split the data into different datasets. If not, you can treat all data as one dataset. e.g. biomedgps. This feature is used to distinguish different dataset combinations matched with your model.
    ///
    /// If you have multiple datasets, you can use the --dataset option with multiple values. e.g. --dataset biomedgps --dataset mecfs
    ///
    /// Each dataset must be registered in the relation table by the importdb command. If not, the import might fail.
    #[structopt(name = "dataset", long = "dataset", multiple = true)]
    dataset: Vec<String>,

    /// [Optional] Description of the model.
    #[structopt(name = "description", short = "D", long = "description")]
    description: Option<String>,

    /// [Optional] Drop the table before import data. If you have multiple files to import, don't use this option. If you use this option, only the last file will be imported successfully.
    #[structopt(name = "drop", short = "D", long = "drop")]
    drop: bool,

    /// [Optional] Don't check the validity of the data files.
    #[structopt(name = "skip_check", short = "s", long = "skip-check")]
    skip_check: bool,

    /// [Optional] Show the first 3 errors when import data.
    #[structopt(name = "show_all_errors", short = "e", long = "show-all-errors")]
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
        SubCommands::InitTable(arguments) => {
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
                        "DRUGBANK::treats::Compound:Disease,HSDN::has_symptom:Disease:Symptom";
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
                "knowledge-score" => {
                    match create_kg_score_table(
                        &pool,
                        Some(&arguments.table_prefix),
                    )
                    .await
                    {
                        Ok(_) => info!("Init kg score table successfully."),
                        Err(e) => error!("Init kg score table failed: {}", e),
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
            .await
        }
        SubCommands::ImportGraph(arguments) => {
            let neo4j_url = if arguments.neo4j_url.is_none() {
                match std::env::var("NEO4J_URL") {
                    Ok(v) => v,
                    Err(_) => {
                        error!("{}", "NEO4J_URL is not set.");
                        std::process::exit(1);
                    }
                }
            } else {
                arguments.neo4j_url.unwrap()
            };

            let graph = connect_graph_db(&neo4j_url).await;

            let filetype = if arguments.filetype.is_none() {
                error!("Please specify the file type.");
                std::process::exit(1);
            } else {
                arguments.filetype.unwrap()
            };

            let batch_size = if arguments.batch_size.is_none() {
                1000
            } else {
                arguments.batch_size.unwrap()
            };

            if filetype == "entity"
                || filetype == "relation"
                || filetype == "entity_attribute"
                || filetype == "relation_attribute"
            {
                import_graph_data(
                    &graph,
                    &arguments.filepath,
                    &filetype,
                    arguments.skip_check,
                    arguments.check_exist,
                    arguments.show_all_errors,
                    batch_size,
                    &arguments.dataset,
                )
                .await
            }

            if filetype == "entity_index" {
                build_index(
                    &graph,
                    &arguments.filepath,
                    arguments.skip_check,
                    arguments.show_all_errors,
                )
                .await
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
            let model_name = arguments.model_name;
            let model_type = arguments.model_type;
            let description = arguments.description;
            let datasets: Vec<&str> = arguments.dataset.iter().map(|s| s.as_str()).collect();
            let drop = arguments.drop;
            let skip_check = arguments.skip_check;
            let show_all_errors = arguments.show_all_errors;

            import_kge(
                &database_url,
                &table_name,
                &model_name,
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
    }
}
