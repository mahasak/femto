use axum::ServiceExt;
use axum::extract::Request;
use cache::CacheService;
use database::Database;
use dotenv::dotenv;
use emit::{__emit_get_event_data, emit, info};
use emit::{
    collectors::stdio::StdioCollector, formatters::text::PlainTextFormatter, PipelineBuilder,
};
use std::{env, error::Error, io};
use std::sync::Mutex;
use tokio::net::TcpListener;
use tower_http::normalize_path::NormalizePathLayer;
use tower_layer::Layer;
use utils::emit_seq::SeqCollector;

use crate::handlers::router;

extern crate slog;
extern crate hostname;
extern crate slog_term;
extern crate slog_async;
extern crate slog_gelf;

use slog::{o, Drain, Logger};

mod cache;
mod database;
mod errors;
mod handlers;
mod models;
mod utils;

#[tokio::main]
async fn main() -> io::Result<()> {
    dotenv().ok();
    env_logger::init();
    let server_url = env::var("SEQ_SERVER").expect("env::SEQ_SERVER is missing");
    let api_key = env::var("SEQ_API_KEY").expect("env::SEQ_API_KEY is missing");
    let gelf_address = env::var("GELF_UDP_ADDRESS").expect("env::GELF_UDP_ADDRESS is missing");
    let server_api_url = format!("{server_url}/api/events/raw?clef");

    let seq_builder = SeqCollector::builder()
        .server_url(server_api_url)
        .api_key(api_key);

    let _flush = PipelineBuilder::new()
        .write_to(StdioCollector::new(PlainTextFormatter::new()))
        .send_to(seq_builder.build())
        .init();
    let hostname = hostname::get()?;

    let decorator = slog_term::TermDecorator::new().build();

    let term = slog_term::FullFormat::new(decorator).build().fuse();
    let gelf = slog_gelf::Gelf::new(&*hostname.into_string().unwrap(), &*gelf_address)?
        .fuse();

    let drains = Mutex::new(slog::Duplicate::new(term, gelf)).fuse();
    let drains = slog_async::Async::new(drains)
        //.overflow_strategy(OverflowStrategy::Block)
        .build()
        .fuse();
    let logger = slog::Logger::root(drains, o!("key" => "value"));
    let database = Database::init().await;
    let cache = CacheService::init().await;

    let result = run(database, cache, logger).await;
    if result.is_err() {
        log::error!("{}", result.unwrap_err().to_string());
        std::process::exit(1)
    } else {
      info!("Successfully start server !", );
    }

    Ok(())
}

pub async fn run(database: Database, cache: CacheService, logger: Logger) -> Result<(), Box<dyn Error>> {
    let app_environment = env::var("APP_ENVIRONMENT").unwrap_or("development".to_string());
    let app_host = env::var("APP_HOST").unwrap_or("0.0.0.0".to_string());
    let app_port = env::var("APP_PORT").unwrap_or("3000".to_string());
    let bind_address =format!("{app_host}:{app_port}");
    let listener = TcpListener::bind(&bind_address).await.unwrap();

    slog::info!(logger,
        "Environment configs: {environment} ",
        environment = app_environment.clone()
    );

    slog::info!(logger,
        "host config: {app_host}",
        app_host = app_host.clone()
    );

    slog::info!(logger,
        "port config: {app_port}",
        app_port = app_port.clone()
    );

    slog::info!(logger,
        "address config: {address}",
        address = bind_address.clone()
    );


    let app =  NormalizePathLayer::trim_trailing_slash().layer(router(database, cache.clone(), logger));
    let app = ServiceExt::<Request>::into_make_service(app);
    axum::serve(listener, app)
        .await
        .unwrap();

    Ok(())
}

