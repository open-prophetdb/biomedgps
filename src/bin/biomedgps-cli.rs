extern crate log;

use biomedgps::{import_data, run_migrations, init_logger};
use log::*;
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

    /// Drop the table before import data. If you have multiple files to import, don't use this option. If you use this option, only the last file will be imported successfully.
    #[structopt(name = "drop", short = "D", long = "drop")]
    drop: bool,

    /// Don't check other related tables in the database. Such as knowledge_curation which might be related to entity.
    #[structopt(name = "skip_check", short = "s", long = "skip-check")]
    skip_check: bool,

    /// Show the first 3 errors when import data.
    #[structopt(name = "show_all_errors", short = "e", long = "show-all-errors")]
    show_all_errors: bool,
}

#[tokio::main]
async fn main() {
    let opt = Opt::from_args();

    let log_result = if opt.debug {
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

            import_data(
                &database_url,
                &arguments.filepath,
                &arguments.table,
                arguments.drop,
                arguments.skip_check,
                arguments.show_all_errors,
            )
            .await
        }
    }
}
