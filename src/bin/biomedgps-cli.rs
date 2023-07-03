extern crate log;
extern crate stderrlog;

use log::*;
use sqlx::migrate::Migrator;
use std::error::Error;
use std::fs::File;
use std::io::Write;
use std::path::Path;
use std::path::PathBuf;
use structopt::StructOpt;
use tempfile::tempdir;

use biomedgps::api::model::{
    CheckData, Entity, Entity2D, EntityMetadata, KnowledgeCuration, Relation, RelationMetadata,
    Subgraph,
};

use biomedgps::api::util::get_delimiter;

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
    filepath: String,

    /// The table name to import data into. supports entity, entity2d, relation, relation_metadata, entity_metadata, knowledge_curation, subgraph, record_response.
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

async fn drop_table(pool: &sqlx::PgPool, table: &str) {
    sqlx::query(&format!(
        "
        DO $$ BEGIN
        IF EXISTS (SELECT FROM information_schema.tables 
                    WHERE  table_schema = 'public' 
                    AND    table_name   = '{}')
        THEN
            DELETE FROM {};
        END IF;
        END $$;
        ",
        table, table
    ))
    .execute(pool)
    .await
    .unwrap();
}

async fn import_file_in_loop(
    pool: &sqlx::PgPool,
    filepath: &PathBuf,
    table_name: &str,
    expected_columns: &Vec<String>,
    unique_columns: &Vec<&str>,
    delimiter: u8,
) -> Result<(), Box<dyn Error>> {
    match sqlx::query("DROP TABLE staging").execute(pool).await {
        Ok(_) => {}
        Err(_) => {}
    }

    let mut tx = pool.begin().await?;
    // Here we replace '{}' and {} placeholders with the appropriate values.
    sqlx::query(&format!(
        "CREATE TEMPORARY TABLE staging (LIKE {} INCLUDING ALL)",
        table_name
    ))
    .execute(&mut tx)
    .await?;

    let columns = expected_columns.join(",");
    let query_str = format!(
        "COPY staging ({}) FROM '{}' DELIMITER E'{}' CSV HEADER",
        columns,
        filepath.display(),
        delimiter as char
    );

    debug!("Importing query string: {}", query_str);

    sqlx::query(&query_str).execute(&mut tx).await?;

    let where_clause = unique_columns
        .iter()
        .map(|c| format!("{}.{} = staging.{}", table_name, c, c))
        .collect::<Vec<String>>()
        .join(" AND ");

    sqlx::query(&format!(
        "INSERT INTO {} ({})
         SELECT {} FROM staging
         WHERE NOT EXISTS (SELECT 1 FROM {} WHERE {})",
        table_name, columns, columns, table_name, where_clause
    ))
    .execute(&mut tx)
    .await?;

    tx.commit().await?;

    Ok(())
}

async fn import_file(
    pool: &sqlx::PgPool,
    filepath: &PathBuf,
    table_name: &str,
    expected_columns: &Vec<String>,
    delimiter: u8,
    drop: bool,
) -> Result<(), Box<dyn Error>> {
    if drop {
        drop_table(&pool, table_name).await;
    };

    let columns = expected_columns.join(", ");

    let stmt = format!(
        r#"COPY {} ({}) FROM '{}' DELIMITER E'{}' CSV HEADER;"#,
        table_name,
        columns,
        filepath.display(),
        delimiter as char
    );

    debug!("Importing query string: {}", stmt);

    sqlx::query(&stmt)
        .execute(pool)
        .await
        .expect("Failed to import data.");
    info!("{} imported.", filepath.display());

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

            if arguments.filepath.is_empty() {
                error!("Please specify the file path.");
                return;
            }

            if arguments.table.is_empty() {
                error!("Please specify the table name.");
                return;
            }

            let pool = sqlx::postgres::PgPoolOptions::new()
                .connect(&database_url)
                .await
                .unwrap();

            let mut files = vec![];
            if std::path::Path::new(&arguments.filepath).is_dir() {
                let paths = std::fs::read_dir(&arguments.filepath).unwrap();
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
                files.push(std::path::PathBuf::from(&arguments.filepath));
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
                } else if table == "entity_metadata" {
                    EntityMetadata::check_csv_is_valid(&file)
                } else if table == "entity2d" {
                    Entity2D::check_csv_is_valid(&file)
                } else if table == "relation" {
                    Relation::check_csv_is_valid(&file)
                } else if table == "relation_metadata" {
                    RelationMetadata::check_csv_is_valid(&file)
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
                } else if table == "entity_metadata" {
                    EntityMetadata::get_column_names(&file)
                } else if table == "entity2d" {
                    Entity2D::get_column_names(&file)
                } else if table == "relation" {
                    Relation::get_column_names(&file)
                } else if table == "relation_metadata" {
                    RelationMetadata::get_column_names(&file)
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
                } else if table == "entity_metadata" {
                    EntityMetadata::select_expected_columns(&file, &temp_filepath)
                } else if table == "entity2d" {
                    Entity2D::select_expected_columns(&file, &temp_filepath)
                } else if table == "relation" {
                    Relation::select_expected_columns(&file, &temp_filepath)
                } else if table == "relation_metadata" {
                    RelationMetadata::select_expected_columns(&file, &temp_filepath)
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
                        if arguments.drop {
                            drop_table(&pool, "staging").await;
                        };

                        import_file_in_loop(
                            &pool,
                            &file,
                            "biomedgps_entity",
                            &expected_columns,
                            &vec!["id", "label"],
                            delimiter,
                        )
                        .await
                        .expect("Failed to import data into the biomedgps_entity table.");
                    }
                    "relation" => {
                        if arguments.drop {
                            drop_table(&pool, "staging").await;
                        };

                        import_file_in_loop(
                            &pool,
                            &file,
                            "biomedgps_relation",
                            &expected_columns,
                            &vec![
                                "relation_type",
                                "source_id",
                                "source_type",
                                "target_id",
                                "target_type",
                            ],
                            delimiter,
                        )
                        .await
                        .expect("Failed to import data into the biomedgps_relation table.");
                    }
                    "entity_metadata" => {
                        import_file(
                            &pool,
                            &file,
                            "biomedgps_entity_metadata",
                            &expected_columns,
                            delimiter,
                            arguments.drop,
                        )
                        .await
                        .expect("Failed to import data into the biomedgps_entity_metadata table.");
                    }
                    "relation_metadata" => {
                        import_file(
                            &pool,
                            &file,
                            "biomedgps_relation_metadata",
                            &expected_columns,
                            delimiter,
                            arguments.drop,
                        )
                        .await
                        .expect(
                            "Failed to import data into the biomedgps_relation_metadata table.",
                        );
                    }
                    "entity2d" => {
                        import_file(
                            &pool,
                            &file,
                            "biomedgps_entity2d",
                            &&expected_columns,
                            delimiter,
                            arguments.drop,
                        )
                        .await
                        .expect("Failed to import data into the biomedgps_entity2d table.");
                    }
                    "knowledge_curation" => {
                        import_file(
                            &pool,
                            &file,
                            "biomedgps_knowledge_curation",
                            &expected_columns,
                            delimiter,
                            arguments.drop,
                        )
                        .await
                        .expect(
                            "Failed to import data into the biomedgps_knowledge_curation table.",
                        );
                    }
                    "subgraph" => {
                        import_file(
                            &pool,
                            &file,
                            "biomedgps_subgraph",
                            &expected_columns,
                            delimiter,
                            arguments.drop,
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
