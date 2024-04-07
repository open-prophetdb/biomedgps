#[macro_use]
extern crate log;
#[macro_use]
extern crate lazy_static;

use biomedgps::api::auth::fetch_and_store_jwks;
use biomedgps::api::route::BiomedgpsApi;
use biomedgps::model::core::EntityMetadata;
use biomedgps::model::kge::init_kge_models;
use biomedgps::model::llm::init_prompt_templates;
use biomedgps::model::util::update_existing_colors;
use biomedgps::proxy::website::{
    proxy_website, proxy_website_data, PROXY_DATA_PREFIX, PROXY_PREFIX,
};
use biomedgps::{check_db_version, connect_db, connect_graph_db, init_logger};
use dotenv::dotenv;
use itertools::Itertools;
use log::LevelFilter;
use poem::{
    async_trait,
    endpoint::EmbeddedFilesEndpoint,
    get, handler,
    http::{header, Method, StatusCode},
    listener::TcpListener,
    middleware::{AddData, Cors},
    web::Redirect,
    Endpoint, EndpointExt, Request, Response, Result, Route, Server,
};
use poem_openapi::OpenApiService;
use rust_embed::RustEmbed;
use std::sync::Arc;

use structopt::StructOpt;

/// BioMedGPS backend server.
#[derive(Debug, PartialEq, StructOpt)]
#[structopt(setting=structopt::clap::AppSettings::ColoredHelp, name="biomedgps", author="Jingcheng Yang <yjcyxky@163.com>")]
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

    /// Enable simple CORS support.
    #[structopt(name = "cors", short = "c", long = "cors")]
    cors: bool,

    /// 127.0.0.1 or 0.0.0.0
    #[structopt(name = "host", short = "H", long = "host", possible_values=&["127.0.0.1", "0.0.0.0"], default_value = "127.0.0.1")]
    host: String,

    /// Which port.
    #[structopt(name = "port", short = "p", long = "port", default_value = "3000")]
    port: String,

    /// Database url, such as postgres:://user:pass@host:port/dbname.
    /// You can also set it with env var: DATABASE_URL.
    #[structopt(name = "database-url", short = "d", long = "database-url")]
    database_url: Option<String>,

    /// Pool size for database connection.
    #[structopt(name = "pool-size", short = "s", long = "pool-size")]
    pool_size: Option<u32>,

    /// Graph Database url, such as neo4j:://user:pass@host:port. We will always use the default database.
    /// You can also set it with env var: NEO4J_URL.
    #[structopt(name = "neo4j-url", short = "g", long = "neo4j-url")]
    neo4j_url: Option<String>,

    /// JWT secret key.
    /// You can also set it with env var: JWT_SECRET_KEY.
    /// If you don't set it, the server will disable JWT verification with HS256 algorithm. You can use the API with Authorization header and set it to any value.
    #[structopt(name = "jwt-secret-key", short = "k", long = "jwt-secret-key")]
    jwt_secret_key: Option<String>,

    /// JWT client id.
    /// You can also set it with env var: JWT_CLIENT_ID.
    /// If you don't set it, the server will disable JWT verification with RS256 algorithm. You can use the API with Authorization header and set it to any value.
    #[structopt(name = "jwt-client-id", short = "i", long = "jwt-client-id")]
    jwt_client_id: Option<String>,

    /// JWT jwks url.
    /// You can also set it with env var: JWT_JWKS_URL. such as https://biomedgps.jp.auth0.com/.well-known/jwks.json.
    /// If you don't set it, the server will disable JWT verification with RS256 algorithm. You can use the API with Authorization header and set it to any value.
    #[structopt(name = "jwt-jwks-url", short = "j", long = "jwt-jwks-url")]
    jwt_jwks_url: Option<String>,
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

#[handler]
async fn index() -> Redirect {
    Redirect::moved_permanent("/index.html")
}

#[tokio::main]
async fn main() -> Result<(), std::io::Error> {
    dotenv().ok();

    let args = Opt::from_args();

    let log_result = if args.debug {
        init_logger("biomedgps", LevelFilter::Debug)
    } else {
        init_logger("biomedgps", LevelFilter::Info)
    };

    if let Err(log) = log_result {
        error!(target:"stdout", "Log initialization error, {}", log);
        std::process::exit(1);
    };

    let host = args.host;
    let port = args.port;

    println!("\n\t\t*** Launch biomedgps on {}:{} ***", host, port);

    // Set up and check JWT environment variables.
    // For HS256 algorithm, such as integrating with Label Studio.
    if args.jwt_secret_key.is_none() {
        match std::env::var("JWT_SECRET_KEY") {
            Ok(v) => {
                if v.is_empty() {
                    warn!("You don't set JWT_SECRET_KEY environment variable, so we will skip JWT verification, but users also need to set the Authorization header to access the API.");
                    None
                } else {
                    Some(v)
                }
            }
            Err(_) => {
                warn!("You don't set JWT_SECRET_KEY environment variable, so we will skip JWT verification, but users also need to set the Authorization header to access the API.");
                None
            }
        }
    } else {
        std::env::set_var("JWT_SECRET_KEY", args.jwt_secret_key.unwrap());
        None
    };

    // For RS256 algorithm, such as integrating with Auth0.
    let jwt_client_id = if args.jwt_client_id.is_none() {
        match std::env::var("JWT_CLIENT_ID") {
            Ok(v) => {
                if v.is_empty() {
                    warn!("You don't set JWT_CLIENT_ID environment variable, so we will skip JWT verification, but users also need to set the Authorization header to access the API.");
                    None
                } else {
                    Some(v)
                }
            }
            Err(_) => {
                warn!("You don't set JWT_CLIENT_ID environment variable, so we will skip JWT verification, but users also need to set the Authorization header to access the API.");
                None
            }
        }
    } else {
        let _jwt_client_id = args.jwt_client_id.unwrap();
        std::env::set_var("JWT_CLIENT_ID", &_jwt_client_id);
        Some(_jwt_client_id)
    };

    // For RS256 algorithm, such as integrating with Auth0.
    let jwks_url = if args.jwt_jwks_url.is_none() {
        match std::env::var("JWT_JWKS_URL") {
            Ok(v) => {
                if v.is_empty() {
                    warn!("You don't set JWT_JWKS_URL environment variable, so we will skip JWT verification, but users also need to set the Authorization header to access the API.");
                    None
                } else {
                    Some(v)
                }
            }
            Err(_) => {
                warn!("You don't set JWT_JWKS_URL environment variable, so we will skip JWT verification, but users also need to set the Authorization header to access the API.");
                None
            }
        }
    } else {
        let _jwks_url = args.jwt_jwks_url.unwrap();
        Some(_jwks_url)
    };

    if jwt_client_id.is_some() && jwks_url.is_some() {
        let _ = match fetch_and_store_jwks(&jwks_url.unwrap()).await {
            Ok(_) => {
                debug!("Fetching and storing jwks for RS256 algorithm successfully.");
                Some(())
            }
            Err(err) => {
                error!(
                    "Fetching and storing jwks for RS256 algorithm failed, {}",
                    err
                );
                None
            }
        };
    }

    // Connect to database.
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

    let pool_size = args.pool_size.unwrap_or(10);
    let pool = connect_db(&database_url, pool_size).await;
    let arc_pool = Arc::new(pool);
    let shared_rb = AddData::new(arc_pool.clone());

    // Check the environment, such as database version.
    match check_db_version(&arc_pool.clone()).await {
        Ok(_) => (),
        Err(err) => {
            error!("Check database version failed, {}", err);
            std::process::exit(1);
        }
    };

    // Update existing colors.
    let entity_types: Vec<String> = match EntityMetadata::get_entity_metadata(&arc_pool).await {
        Ok(entity_types) => entity_types.into_iter().map(|x| x.entity_type).collect(),
        Err(err) => {
            error!("Get entity metadata failed, {}", err);
            std::process::exit(1);
        }
    };
    let unique_entity_types: Vec<String> = entity_types.into_iter().unique().collect();
    update_existing_colors(&unique_entity_types);

    // Initialize KGE models.
    let _ = match init_kge_models(&arc_pool).await {
        Ok(_) => {
            debug!("Initialize KGE models successfully.");
            Some(())
        }
        Err(err) => {
            error!("Initialize KGE models failed, {}", err);
            None
        }
    };

    // Connect to graph database.
    let neo4j_url = args.neo4j_url;
    let _neo4j_url = if neo4j_url.is_none() {
        match std::env::var("NEO4J_URL") {
            Ok(v) => v,
            Err(_) => {
                error!("{}", "NEO4J_URL is not set.");
                std::process::exit(1);
            }
        }
    } else {
        neo4j_url.unwrap()
    };

    // Initialize the prompt templates.
    match std::env::var("OPENAI_API_KEY") {
        Ok(openai_api_key) => {
            init_prompt_templates();
        }
        Err(e) => {
            let err = format!("Failed to get OPENAI_API_KEY: {}, so we will skip initializing the prompt templates.", e);
            warn!("{}", err);
        }
    };

    let graph_pool = connect_graph_db(&_neo4j_url).await;
    let arc_graph_pool = Arc::new(graph_pool);
    let shared_graph_pool = AddData::new(arc_graph_pool.clone());

    let api_service = OpenApiService::new(BiomedgpsApi, "BioMedGPS", "v0.1.0")
        .summary("A RESTful API Service for BioMedGPS.")
        .description("A knowledge graph system with graph neural network for drug discovery, disease mechanism and biomarker screening.")
        .license("GNU AFFERO GENERAL PUBLIC LICENSE v3")
        .server(format!("http://{}:{}", host, port));
    let openapi = api_service.swagger_ui();
    let mut spec = api_service.spec();

    // Remove charset=utf-8 from spec for compatibility with Apifox.
    spec = spec.replace("; charset=utf-8", "");

    let route = Route::new();

    let route = if args.openapi {
        info!("OpenApi mode is enabled. You can access the OpenApi spec at /openapi.");
        route
            .nest("/openapi", openapi)
            .at("/spec", poem::endpoint::make_sync(move |_| spec.clone()))
    } else {
        warn!("OpenApi mode is disabled. If you need the OpenApi, please use `--openapi` flag.");
        route
    };

    let route = if args.ui {
        info!("UI mode is enabled.");
        route
            .at("/", HtmlEmbed)
            .nest("/index.html", HtmlEmbed)
            .nest("/assets", EmbeddedFilesEndpoint::<Assets>::new())
    } else {
        warn!("UI mode is disabled. If you need the UI, please use `--ui` flag.");
        route
    };

    // Proxy website. such as /proxy/sanger_cosmic?gene_symbol=TP53. if you want to know more about the proxy website and query parameters, please check the website module.
    let route = route.at(format!("{}/*", PROXY_PREFIX), get(proxy_website));
    // All other requests related to the proxy website will be transferred to the proxy-data route.
    let route = route.at(
        format!("{}/*", PROXY_DATA_PREFIX),
        get(proxy_website_data).post(proxy_website_data),
    );

    let route = route
        .nest_no_strip("/api/v1", api_service)
        .with(shared_rb)
        .with(shared_graph_pool);

    if args.cors {
        info!("CORS mode is enabled.");
        let route = route.with(Cors::new().allow_origin("*"));
        Server::new(TcpListener::bind(format!("{}:{}", host, port)))
            .run(route)
            .await
    } else {
        warn!("CORS mode is disabled. If you need the CORS, please use `--cors` flag.");
        Server::new(TcpListener::bind(format!("{}:{}", host, port)))
            .run(route)
            .await
    }
}
