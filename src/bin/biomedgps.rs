#[macro_use]
extern crate log;
#[macro_use]
extern crate lazy_static;

use dotenv::dotenv;
use log::{error, LevelFilter};
use log4rs;
use log4rs::append::console::ConsoleAppender;
use log4rs::config::{Appender, Config, Logger, Root};
use log4rs::encode::pattern::PatternEncoder;
use poem::middleware::AddData;
use poem::EndpointExt;
use poem::{
    async_trait,
    endpoint::EmbeddedFilesEndpoint,
    http::{header, Method, StatusCode},
    listener::TcpListener,
    middleware::Cors,
    Endpoint, Request, Response, Result, Route, Server,
};
use poem_openapi::OpenApiService;
use rnmpdb::api::{model::DatasetPageResponse, route::RnmpdbApi, util};
use rust_embed::RustEmbed;
use sqlx::postgres::PgPoolOptions;
use std::env;
use std::error::Error;
use std::path::Path;
use std::path::Path as OsPath;
use std::sync::Arc;
// use tokio::{self, time::Duration};

use structopt::StructOpt;

fn init_logger(tag_name: &str, level: LevelFilter) -> Result<log4rs::Handle, String> {
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

/// rNMP Database
#[derive(Debug, PartialEq, StructOpt)]
#[structopt(setting=structopt::clap::AppSettings::ColoredHelp, name="rnmpdb", author="Jingcheng Yang <yjcyxky@163.com>")]
struct Opt {
    /// Activate debug mode
    /// short and long flags (--debug) will be deduced from the field's name
    #[structopt(name = "debug", long = "debug")]
    debug: bool,

    /// Activate ui mode
    #[structopt(name = "ui", short = "u", long = "ui")]
    ui: bool,

    /// Activate openapi mode
    #[structopt(name = "openapi", short = "o", long = "openapi")]
    openapi: bool,

    /// Check all dependencies, such as bigwig files, reference genomes.
    #[structopt(name = "check_deps", short = "c", long = "check-deps")]
    check_deps: bool,

    /// 127.0.0.1 or 0.0.0.0
    #[structopt(name = "host", short = "H", long = "host", possible_values=&["127.0.0.1", "0.0.0.0"], default_value = "127.0.0.1")]
    host: String,

    /// Which port.
    #[structopt(name = "port", short = "p", long = "port", default_value = "3000")]
    port: String,

    /// Data directory. It is expected to contain bedgraph files and reference genomes.
    #[structopt(name = "data-dir", short = "D", long = "data-dir")]
    data_dir: String,

    /// Database url, such as postgres:://user:pass@host:port/dbname.
    /// You can also set it with env var: DATABASE_URL.
    #[structopt(name = "database-url", short = "d", long = "database-url")]
    database_url: Option<String>,
}

#[derive(RustEmbed)]
#[folder = "assets/"]
pub struct Assets;

pub(crate) struct HtmlEmbed;

#[async_trait]
impl Endpoint for HtmlEmbed {
    type Output = Response;

    async fn call(&self, req: Request) -> Result<Self::Output> {
        if req.method() != Method::GET {
            return Ok(StatusCode::METHOD_NOT_ALLOWED.into());
        }

        match Assets::get("index.html") {
            Some(content) => {
                let body: Vec<u8> = content.data.into();
                warn!("If you found 404 error when getting static files, please check your frontend's configuration. You might need to set the publicPath to `/assets/`.");
                Ok(Response::builder()
                    .header(header::CONTENT_TYPE, "text/html")
                    .body(body))
            }
            None => Ok(Response::builder().status(StatusCode::NOT_FOUND).finish()),
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), std::io::Error> {
    dotenv().ok();

    let args = Opt::from_args();

    let log_result = if args.debug {
        init_logger("rnmpdb", LevelFilter::Trace)
    } else {
        init_logger("rnmpdb", LevelFilter::Info)
    };

    if let Err(log) = log_result {
        error!(target:"stdout", "Log initialization error, {}", log);
        std::process::exit(1);
    };

    let host = args.host;
    let port = args.port;

    println!(
        "\n\t\t*** Launch rnmpdb on {}:{} ***",
        host,
        port
    );

    let database_url = args.database_url;

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

    let pool = match PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await
    {
        Ok(v) => v,
        Err(e) => {
            error!("Failed to connect to database: {}", e);
            std::process::exit(1);
        }
    };

    if args.check_deps {
        let sample_names = match DatasetPageResponse::get_sample_names(&pool).await {
            Ok(v) => v,
            Err(e) => {
                error!("Failed to get sample names: {}", e);
                std::process::exit(1);
            }
        };

        let path = Path::new(&args.data_dir).join("bedgraph");
        match util::check_bedgraphs(path.as_path(), &sample_names) {
            Ok(_) => {}
            Err(e) => {
                for error in e {
                    error!("{}", error);
                }
                std::process::exit(1);
            }
        }

        let path = Path::new(&args.data_dir).join("bigwig");
        match util::check_bigwigs(path.as_path(), &sample_names) {
            Ok(_) => {}
            Err(e) => {
                for error in e {
                    error!("{}", error);
                }
                std::process::exit(1);
            }
        }

        let ref_genomes = match DatasetPageResponse::get_ref_genomes(&pool).await {
            Ok(v) => v,
            Err(e) => {
                error!("Failed to get reference genomes: {}", e);
                std::process::exit(1);
            }
        };

        let path = Path::new(&args.data_dir).join("genome");
        match util::check_ref_genomes(path.as_path(), &ref_genomes) {
            Ok(_) => {}
            Err(e) => {
                for error in e {
                    error!("{}", error);
                }
                std::process::exit(1);
            }
        }
    }

    let arc_pool = Arc::new(pool);
    let shared_rb = AddData::new(arc_pool.clone());

    let api_service = OpenApiService::new(RnmpdbApi, "rNMPDB", "v0.1.0")
        .summary("A RESTful API for rNMPDB")
        .description("A platform for discovering rNMPs.")
        .license("GNU AFFERO GENERAL PUBLIC LICENSE v3")
        .server(format!("http://{}:{}", host, port));
    let openapi = api_service.swagger_ui();
    let mut spec = api_service.spec();
    // Remove charset=utf-8 from spec for compatibility with Apifox.
    spec = spec.replace("; charset=utf-8", "");

    let route = Route::new().nest("/", api_service);

    let route = if args.openapi {
        info!("OpenApi mode is enabled. You can access the OpenApi spec at /openapi.");
        route
            .nest("/openapi", openapi)
            .at(
                "/spec",
                poem::endpoint::make_sync(move |_| spec.clone()),
            )
    } else {
        warn!("OpenApi mode is disabled. If you need the OpenApi, please use `--openapi` flag.");
        route
    };
    
    let route = if args.ui {
        info!("UI mode is enabled.");
        route.nest("/index.html", HtmlEmbed)
             .nest("/assets", EmbeddedFilesEndpoint::<Assets>::new())
    } else {
        warn!("UI mode is disabled. If you need the UI, please use `--ui` flag.");
        route
    };

    let route = route.with(Cors::new()).with(shared_rb);

    Server::new(TcpListener::bind(format!("{}:{}", host, port)))
        .run(route)
        .await
    // Server::new(TcpListener::bind(format!("{}:{}", host, port)))
    //   .run_with_graceful_shutdown(
    //     route,
    //     async move {
    //       let _ = tokio::signal::ctrl_c().await;
    //     },
    //     Some(Duration::from_secs(5)),
    //   )
    //   .await
}
