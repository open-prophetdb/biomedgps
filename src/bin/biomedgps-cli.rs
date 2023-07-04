extern crate log;
extern crate stderrlog;

use log::*;
use sqlx::migrate::Migrator;
use std::fs::File;
use std::io::Write;
use std::path::Path;
use std::path::PathBuf;
use structopt::StructOpt;
use tempfile::tempdir;

use biomedgps::api::model::{
    CheckData, Entity, Entity2D, EntityEmbedding, KnowledgeCuration, Relation, RelationEmbedding,
    Subgraph,
};

use biomedgps::api::util::{
    drop_table, get_delimiter, import_file, import_file_in_loop, update_entity_metadata,
    update_relation_metadata,
};

/// A cli for rnmpdb.
#[derive(StructOpt, Debug)]
#[structopt(setting=structopt::clap::AppSettings::ColoredHelp, name = "A cli for biomedgps service.", author="Jingcheng Yang <yjcyxky@163.com>;")]
struct Opt {
    /// A flag which control whether show more messages, true if used in the command line
    #[structopt(short = "q", long = "quiet")]
    quiet: bool,

    /// The number of occurrences of the `v/verbose` flag
    /// Verbose mode (-v/Debug, -vv/Trace, etc.)
    #[structopt(short = "v", long = "verbose", parse(from_occurrences))]
    verbose: usize,

    /// Timestamp (sec, ms, ns, none)
    #[structopt(short = "t", long = "timestamp")]
    ts: Option<stderrlog::Timestamp>,

    #[structopt(subcommand)]
    cmd: SubCommands,
}

#[derive(Debug, PartialEq, StructOpt)]
enum SubCommands {
    #[structopt(name = "initdb")]
    InitDB(InitDbArguments),
    #[structopt(name = "importdb")]
    ImportDB(ImportDBArguments),
    // #[structopt(name = "importgraph")]
    // ImportGraph(ImportGraphArguments),
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

    /// The file path of the data file to import. It may be a file or a directory.
    #[structopt(name = "filepath", short = "f", long = "filepath")]
    filepath: Option<String>,

    /// The table name to import data into. supports entity, entity2d, relation, relation_metadata, entity_metadata, knowledge_curation, subgraph, entity_embedding, relation_embedding
    #[structopt(name = "table", short = "t", long = "table")]
    table: String,

    /// Drop the table before import data.
    #[structopt(name = "drop", short = "D", long = "drop")]
    drop: bool,

    /// Show the first 3 errors when import data.
    #[structopt(name = "show_all_errors", short = "e", long = "show-all-errors")]
    show_all_errors: bool,
}

const MIGRATIONS: include_dir::Dir = include_dir::include_dir!("migrations");

async fn run_migrations(database_url: &str) -> sqlx::Result<()> {
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

#[tokio::main]
async fn main() {
    let opt = Opt::from_args();

    stderrlog::new()
        .module(module_path!())
        .module("biomedgps")
        .quiet(opt.quiet)
        .show_module_names(true)
        .verbosity(opt.verbose + 2)
        .timestamp(opt.ts.unwrap_or(stderrlog::Timestamp::Second))
        .init()
        .unwrap();

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

            run_migrations(&database_url).await.unwrap();
        }
        SubCommands::ImportDB(arguments) => {
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

            if arguments.table.is_empty() {
                error!("Please specify the table name.");
                return;
            }

            let pool = sqlx::postgres::PgPoolOptions::new()
                .connect(&database_url)
                .await
                .unwrap();

            if arguments.table == "relation_metadata" {
                update_relation_metadata(&pool, true).await.unwrap();
                return;
            } else if arguments.table == "entity_metadata" {
                update_entity_metadata(&pool, true).await.unwrap();
                return;
            } else if arguments.table == "entity_embedding"
                || arguments.table == "relation_embedding"
            {
                let file = match arguments.filepath {
                    Some(file) => PathBuf::from(file),
                    None => {
                        error!("Please specify the file path.");
                        return;
                    }
                };

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

                if arguments.table == "entity_embedding" {
                    match EntityEmbedding::import_entity_embeddings(
                        &pool,
                        &file,
                        delimiter,
                        arguments.drop,
                    )
                    .await
                    {
                        Ok(_) => {
                            info!("Import entity embeddings successfully.");
                            return;
                        }
                        Err(e) => {
                            error!("Failed to import entity embeddings: {}", e);
                            return;
                        }
                    }
                } else {
                    match RelationEmbedding::import_relation_embeddings(
                        &pool,
                        &file,
                        delimiter,
                        arguments.drop,
                    )
                    .await
                    {
                        Ok(_) => {
                            info!("Import relation embeddings successfully.");
                            return;
                        }
                        Err(e) => {
                            error!("Failed to import relation embeddings: {}", e);
                            return;
                        }
                    }
                }
            } else {
                let filepath = match arguments.filepath {
                    Some(filepath) => filepath,
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
                    info!("Importing {} into {}...", filename, arguments.table);

                    let table = arguments.table.as_str();

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
                        if !arguments.show_all_errors {
                            warn!("Only show the 3 validation errors, if you want to see all errors, use --show-all-errors.");
                            for e in validation_errors.iter().take(3) {
                                error!("{}", e);
                            }

                            warn!("Hide {} validation errors.", validation_errors.len() - 3);
                        } else {
                            for e in validation_errors {
                                error!("{}", e);
                            }
                        }
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

                    match arguments.table.as_str() {
                        "entity" => {
                            let table_name = "biomedgps_entity";
                            if arguments.drop {
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
                            if arguments.drop {
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
                            import_file_in_loop(
                                &pool,
                                &file,
                                "biomedgps_entity2d",
                                &expected_columns,
                                &Entity2D::unique_fields(),
                                delimiter,
                            )
                            .await
                            .expect("Failed to import data into the biomedgps_entity2d table.");
                        }
                        "knowledge_curation" => {
                            import_file_in_loop(
                            &pool,
                            &file,
                            "biomedgps_knowledge_curation",
                            &expected_columns,
                            &KnowledgeCuration::unique_fields(),
                            delimiter,
                        )
                        .await
                        .expect(
                            "Failed to import data into the biomedgps_knowledge_curation table.",
                        );
                        }
                        "subgraph" => {
                            import_file_in_loop(
                                &pool,
                                &file,
                                "biomedgps_subgraph",
                                &expected_columns,
                                &Subgraph::unique_fields(),
                                delimiter,
                            )
                            .await
                            .expect("Failed to import data into the biomedgps_subgraph table.");
                        }
                        _ => {
                            error!("Unsupported table name: {}", arguments.table);
                            return;
                        }
                    };

                    info!("{} imported.\n\n", filename);
                }
            }
        }
    }
}
