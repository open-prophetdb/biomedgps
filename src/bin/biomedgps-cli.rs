extern crate log;

use biomedgps::model::kge::DEFAULT_MODEL_NAME;
use biomedgps::model::util::read_annotation_file;
use biomedgps::{
    build_index, connect_graph_db, import_data, import_graph_data, import_kge, init_logger,
    run_migrations,
};
use log::*;
use std::path::PathBuf;
use structopt::StructOpt;

/// A cli for rnmpdb.
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
    /// Database url, such as postgres://postgres:postgres@localhost:5432/rnmpdb, if not set, use the value of environment variable DATABASE_URL.
    #[structopt(name = "database_url", short = "d", long = "database-url")]
    database_url: Option<String>,

    /// The file path of the data file to import. It may be a file or a directory. If you have multiple files to import, you can use the --filepath option with a directory path. We will import all files in the directory. But you need to disable the --drop option, otherwise, only the last file will be imported successfully.
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

    /// Annotation file path. This option is only required for relation table. It is used to annotate relation_type or other attributes. In current version, it is only used for relation_type.
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

    /// Drop the table before import data. If you have multiple files to import, don't use this option. If you use this option, only the last file will be imported successfully.
    #[structopt(name = "drop", short = "D", long = "drop")]
    drop: bool,

    /// Don't check other related tables in the database. Such as knowledge_curation which might be related to entity.
    #[structopt(name = "skip_check", short = "s", long = "skip-check")]
    skip_check: bool,

    /// Which dataset is the data from. We assume that you have split the data into different datasets. If not, you can treat all data as one dataset. e.g. biomedgps. This feature is used to distinguish different dataset combinations matched with your model. It is only required for relation table.
    #[structopt(name = "dataset", long = "dataset")]
    dataset: Option<String>,

    /// Show the first 3 errors when import data.
    #[structopt(name = "show_all_errors", short = "e", long = "show-all-errors")]
    show_all_errors: bool,
}

/// Import data files into a graph database.
#[derive(StructOpt, PartialEq, Debug)]
#[structopt(setting=structopt::clap::AppSettings::ColoredHelp, name="BioMedGPS - importgraph", author="Jingcheng Yang <yjcyxky@163.com>")]
pub struct ImportGraphArguments {
    /// Database url, such as neo4j://<username>:<password>@localhost:7687, if not set, use the value of environment variable NEO4J_URL.
    #[structopt(name = "neo4j_url", short = "n", long = "neo4j-url")]
    neo4j_url: Option<String>,

    /// The file path of the data file to import. It may be a file or a directory.
    #[structopt(name = "filepath", short = "f", long = "filepath")]
    filepath: Option<String>,

    /// The file type of the data file to import. It may be entity, relation, entity_attribute and relation_attribute.
    #[structopt(name = "filetype", short = "t", long = "filetype")]
    filetype: Option<String>,

    /// Batch size for import data. Default is 1000.
    #[structopt(name = "batch_size", short = "b", long = "batch-size")]
    batch_size: Option<usize>,

    /// Don't check other related tables in the database. Such as knowledge_curation which might be related to entity.
    #[structopt(name = "skip_check", short = "s", long = "skip-check")]
    skip_check: bool,

    /// Check if the data exists in the database before import data.
    #[structopt(name = "check_exist", short = "c", long = "check-exist")]
    check_exist: bool,

    /// Show the first 3 errors when import data.
    #[structopt(name = "show_all_errors", short = "e", long = "show-all-errors")]
    show_all_errors: bool,

    /// Which dataset is the data from. We assume that you have split the data into different datasets. If not, you can treat all data as one dataset. e.g. biomedgps. This feature is used to distinguish different dataset combinations matched with your model.
    #[structopt(name = "dataset", short = "d", long = "dataset")]
    dataset: Option<String>,
}

/// Import embedding files into a database
/// The embedding files are generated by KGE models.
#[derive(StructOpt, PartialEq, Debug)]
#[structopt(setting=structopt::clap::AppSettings::ColoredHelp, name="BioMedGPS - importkge", author="Jingcheng Yang <yjcyxky@163.com>")]
pub struct ImportKGEArguments {
    /// Database url, such as postgres://postgres:postgres@localhost:5432/rnmpdb, if not set, use the value of environment variable DATABASE_URL.
    #[structopt(name = "database_url", short = "d", long = "database-url")]
    database_url: Option<String>,

    /// The file path of the data file to import.
    #[structopt(
        name = "entity_embedding_file",
        short = "e",
        long = "entity-embedding-file"
    )]
    entity_embedding_file: String,

    /// The file path of the data file to import.
    #[structopt(
        name = "relation_embedding_file",
        short = "r",
        long = "relation-embedding-file"
    )]
    relation_embedding_file: String,

    /// The file path of the metadata file to import.
    #[structopt(name = "metadata_file", short = "m", long = "metadata-file")]
    metadata_file: String,

    /// The table name you want to name. e.g. biomedgps, mecfs, etc. This feature is used to distinguish different dataset combinations matched with your model. If not set, we will use the biomedgps as default. But in this case, the dimension of the embedding should be 400.
    #[structopt(
        name = "table_name",
        short = "t",
        long = "table-name",
        default_value = DEFAULT_MODEL_NAME
    )]
    table_name: String,

    /// The model name you want to name. e.g. mecfs_transe, mecfs_distmult, etc. You need to specify the model name when you import the embedding files. This feature is used to distinguish different models. Users can choose the model for their own purpose.
    #[structopt(
        name = "model_name", 
        short = "m", 
        long = "model-name", 
        default_value = DEFAULT_MODEL_NAME
    )]
    model_name: String,

    /// The model type of generated embedding files. e.g. TransE, DistMult, etc.
    #[structopt(name = "model_type", short = "M", long = "model-type")]
    model_type: String,

    /// Which dataset is the data from. We assume that you have split the data into different datasets. If not, you can treat all data as one dataset. e.g. biomedgps. This feature is used to distinguish different dataset combinations matched with your model.
    #[structopt(name = "dataset", long = "dataset", multiple = true)]
    dataset: Vec<String>,

    /// Description of the model.
    #[structopt(name = "description", short = "D", long = "description")]
    description: Option<String>,

    /// Drop the table before import data. If you have multiple files to import, don't use this option. If you use this option, only the last file will be imported successfully.
    #[structopt(name = "drop", short = "D", long = "drop")]
    drop: bool,

    /// Don't check the validity of the data files.
    #[structopt(name = "skip_check", short = "s", long = "skip-check")]
    skip_check: bool,

    /// Show the first 3 errors when import data.
    #[structopt(name = "show_all_errors", short = "e", long = "show-all-errors")]
    show_all_errors: bool,

    /// Annotation file path. This option is only required for legacy relation_embedding file which only contains id, embedding columns.
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
