extern crate log;
extern crate stderrlog;

use log::*;
use serde::Deserialize;
use sqlx::migrate::Migrator;
use std::fs::File;
use std::io::Write;
use std::path::Path;
use std::{error::Error, path::PathBuf};
use structopt::StructOpt;
use tempfile::tempdir;

use rnmpdb::api::model::{BackgroundFrequency, Chromosome, Count, Dataset, Gene};

/// A cli for rnmpdb.
#[derive(StructOpt, Debug)]
#[structopt(setting=structopt::clap::AppSettings::ColoredHelp, name = "A cli for rnmpdb.", author="Jingcheng Yang <yjcyxky@163.com>;")]
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
    #[structopt(name = "import")]
    Import(ImportArguments),
}

/// Init database.
#[derive(StructOpt, PartialEq, Debug)]
#[structopt(setting=structopt::clap::AppSettings::ColoredHelp, name="rNMP Database - initdb", author="Jingcheng Yang <yjcyxky@163.com>")]
pub struct InitDbArguments {
    /// Database url, such as postgres://postgres:postgres@localhost:5432/rnmpdb
    #[structopt(name = "database_url", short = "d", long = "database-url")]
    database_url: Option<String>,
}

/// Import data files into database.
#[derive(StructOpt, PartialEq, Debug)]
#[structopt(setting=structopt::clap::AppSettings::ColoredHelp, name="rNMP Database - import", author="Jingcheng Yang <yjcyxky@163.com>")]
pub struct ImportArguments {
    /// Database url, such as postgres://postgres:postgres@localhost:5432/rnmpdb
    #[structopt(name = "database_url", short = "d", long = "database-url")]
    database_url: Option<String>,

    /// The file path of the data file to import. It may be a file or a directory.
    #[structopt(name = "filepath", short = "f", long = "filepath")]
    filepath: String,

    /// The table name to import data into. supports dataset, count, and background_frequency, gene, chromosome.
    #[structopt(name = "table", short = "t", long = "table")]
    table: String,

    /// Drop the table before import data.
    #[structopt(name = "drop", short = "D", long = "drop")]
    drop: bool,
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

#[derive(Debug, Deserialize)]
#[serde(untagged)]
enum Record {
    Dataset(Dataset),
    Count(Count),
    BackgroundFrequency(BackgroundFrequency),
}

// Implement the check function
fn check_csv_by_dataset(filepath: &PathBuf) -> Result<(), csv::Error> {
    // Build the CSV reader
    let mut reader = csv::ReaderBuilder::new()
        .delimiter(b'\t')
        .from_path(filepath)?; // Use tab as delimiter

    // Try to deserialize each record
    for result in reader.deserialize() {
        let _record: Dataset = result?;
    }

    Ok(())
}

fn check_csv_by_count(filepath: &PathBuf) -> Result<(), csv::Error> {
    // Build the CSV reader
    let mut reader = csv::ReaderBuilder::new()
        .delimiter(b'\t')
        .from_path(filepath)?; // Use tab as delimiter

    // Try to deserialize each record
    for result in reader.deserialize() {
        let _record: Count = result?;
    }

    Ok(())
}

fn check_csv_by_bg_freq(filepath: &PathBuf) -> Result<(), csv::Error> {
    // Build the CSV reader
    let mut reader = csv::ReaderBuilder::new()
        .delimiter(b'\t')
        .from_path(filepath)?; // Use tab as delimiter

    // Try to deserialize each record
    for result in reader.deserialize() {
        let _record: BackgroundFrequency = result?;
    }

    Ok(())
}

fn check_csv_by_chromosome(filepath: &PathBuf) -> Result<(), csv::Error> {
    // Build the CSV reader
    let mut reader = csv::ReaderBuilder::new()
        .delimiter(b'\t')
        .from_path(filepath)?; // Use tab as delimiter

    // Try to deserialize each record
    for result in reader.deserialize() {
        let _record: Chromosome = result?;
    }

    Ok(())
}

fn check_csv_by_gene(filepath: &PathBuf) -> Result<(), csv::Error> {
    // Build the CSV reader
    let mut reader = csv::ReaderBuilder::new()
        .delimiter(b'\t')
        .from_path(filepath)?; // Use tab as delimiter

    // Try to deserialize each record
    for result in reader.deserialize() {
        let _record: Gene = result?;
    }

    Ok(())
}

async fn import_count_file(
    pool: &sqlx::PgPool,
    filepath: &PathBuf,
    count_columns: &Vec<String>,
) -> sqlx::Result<()> {
    match sqlx::query("DROP TABLE staging").execute(pool).await {
        Ok(_) => {}
        Err(_) => {}
    }

    let mut tx = pool.begin().await?;
    // Here we replace '{}' and {} placeholders with the appropriate values.
    sqlx::query("CREATE TEMPORARY TABLE staging (LIKE rnmpdb_count INCLUDING ALL)")
        .execute(&mut tx)
        .await?;

    sqlx::query(&format!(
        "COPY staging ({}) FROM '{}' DELIMITER E'\t' CSV HEADER",
        count_columns.join(","),
        filepath.display()
    ))
    .execute(&mut tx)
    .await?;

    sqlx::query(&format!(
        "INSERT INTO rnmpdb_count ({})
                 SELECT {} FROM staging
                 WHERE NOT EXISTS (SELECT 1 FROM rnmpdb_count 
                                   WHERE rnmpdb_count.sample_name = staging.sample_name 
                                   AND rnmpdb_count.ref_genome = staging.ref_genome 
                                   AND rnmpdb_count.entrez_id = staging.entrez_id)",
        count_columns.join(","),
        count_columns.join(",")
    ))
    .execute(&mut tx)
    .await?;

    tx.commit().await?;

    Ok(())
}

fn get_column_names(filepath: &PathBuf) -> Result<Vec<String>, Box<dyn Error>> {
    let mut reader = csv::ReaderBuilder::new()
        .delimiter(b'\t')
        .from_path(filepath)?; // Use tab as delimiter

    let headers = reader.headers()?;
    let mut column_names = Vec::new();
    for header in headers {
        column_names.push(header.to_string());
    }

    Ok(column_names)
}

#[tokio::main]
async fn main() {
    let opt = Opt::from_args();

    stderrlog::new()
        .module(module_path!())
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
        SubCommands::Import(arguments) => {
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
                    if path.is_file() && path.extension().unwrap() == "tsv" {
                        files.push(path);
                    }
                }
            } else {
                files.push(std::path::PathBuf::from(&arguments.filepath));
            }

            if files.is_empty() {
                error!("No valid files found. Only tsv files are supported.");
                std::process::exit(1);
            }

            for file in files {
                let filename = file.to_str().unwrap();
                info!("Importing {} into {}...", filename, arguments.table);

                let table = arguments.table.as_str();
                let check_csv = match table {
                    "dataset" => check_csv_by_dataset,
                    "count" => check_csv_by_count,
                    "background_frequency" => check_csv_by_bg_freq,
                    "chromosome" => check_csv_by_chromosome,
                    "gene" => check_csv_by_gene,
                    _ => {
                        error!("{} is not a valid table name.", table);
                        std::process::exit(1);
                    }
                };

                match check_csv(&file) {
                    Ok(_) => info!("{}", format!("{} is valid.", filename)),
                    Err(e) => {
                        let columns = get_column_names(&file).unwrap();

                        match *e.kind() {
                            csv::ErrorKind::Deserialize { pos: Some(ref pos), ref err, .. } => {
                                error!(
                                    "CSV does not match the related table, line: {}, field: {}, details: {}",
                                    pos.line(),
                                    columns[err.field().unwrap() as usize],
                                    err.kind()
                                )
                            }
                            _ => {
                                error!("Failed to parse CSV: {}", e);
                            }
                        }

                        continue;
                    }
                }

                match arguments.table.as_str() {
                    "count" => {
                        if arguments.drop {
                            drop_table(&pool, "staging").await;
                        };

                        let count_columns = match get_column_names(&file) {
                            Ok(columns) => columns,
                            Err(e) => {
                                error!("Failed to get column names: {}", e);
                                return;
                            }
                        };

                        import_count_file(&pool, &file, &count_columns)
                            .await
                            .expect("Failed to import data.");
                        info!("{} imported.", filename);
                    }
                    "dataset" => {
                        if arguments.drop {
                            drop_table(&pool, "rnmpdb_dataset").await;
                        };

                        let dataset_columns = match get_column_names(&file) {
                            Ok(columns) => columns,
                            Err(e) => {
                                error!("Failed to get column names: {}", e);
                                return;
                            }
                        };

                        let stmt = format!(
                            r#"COPY rnmpdb_dataset ({}) FROM '{}' DELIMITER E'\t' CSV HEADER;"#,
                            dataset_columns.join(", "),
                            file.display()
                        );
                        sqlx::query(&stmt)
                            .execute(&pool)
                            .await
                            .expect("Failed to import data.");
                        info!("{} imported.", filename);
                    }
                    "chromosome" => {
                        if arguments.drop {
                            drop_table(&pool, "rnmpdb_chromosome").await;
                        };

                        let chromosome_columns = match get_column_names(&file) {
                            Ok(columns) => columns,
                            Err(e) => {
                                error!("Failed to get column names: {}", e);
                                return;
                            }
                        };

                        let stmt = format!(
                            r#"COPY rnmpdb_chromosome ({}) FROM '{}' DELIMITER E'\t' CSV HEADER;"#,
                            chromosome_columns.join(", "),
                            file.display()
                        );
                        sqlx::query(&stmt)
                            .execute(&pool)
                            .await
                            .expect("Failed to import data.");
                        info!("{} imported.", filename);
                    }
                    "gene" => {
                        if arguments.drop {
                            drop_table(&pool, "rnmpdb_gene").await;
                        };

                        let gene_columns = match get_column_names(&file) {
                            Ok(columns) => columns,
                            Err(e) => {
                                error!("Failed to get column names: {}", e);
                                return;
                            }
                        };

                        let stmt = format!(
                            r#"COPY rnmpdb_gene ({}) FROM '{}' DELIMITER E'\t' CSV HEADER;"#,
                            gene_columns.join(", "),
                            file.display()
                        );
                        sqlx::query(&stmt)
                            .execute(&pool)
                            .await
                            .expect("Failed to import data.");
                        info!("{} imported.", filename);
                    }
                    "background_frequency" => {
                        if arguments.drop {
                            drop_table(&pool, "rnmpdb_background_frequency").await;
                        };

                        let background_frequency_columns = match get_column_names(&file) {
                            Ok(columns) => columns,
                            Err(e) => {
                                error!("Failed to get column names: {}", e);
                                return;
                            }
                        };

                        let stmt = format!(
                            r#"COPY rnmpdb_background_frequency ({}) FROM '{}' DELIMITER E'\t' CSV HEADER;"#,
                            background_frequency_columns.join(", "),
                            file.display()
                        );
                        sqlx::query(&stmt)
                            .execute(&pool)
                            .await
                            .expect("Failed to import data.");
                        info!("{} imported.", filename);
                    }
                    _ => {
                        error!("Unsupported table name: {}", arguments.table);
                        return;
                    }
                };
            }
        }
    }
}
