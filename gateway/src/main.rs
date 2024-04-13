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
use tokio::net::TcpListener;
use tower_http::normalize_path::NormalizePathLayer;
use tower_layer::Layer;
use utils::emit_seq::SeqCollector;

use crate::handlers::router;

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
    let server_api_url = format!("{server_url}/api/events/raw?clef");

    let seq_builder = SeqCollector::builder()
        .server_url(server_api_url)
        .api_key(api_key);

    let _flush = PipelineBuilder::new()
        .write_to(StdioCollector::new(PlainTextFormatter::new()))
        .send_to(seq_builder.build())
        .init();

    let database = Database::init().await;
    let cache = CacheService::init().await;

    log::debug!("{:?}", database);
    log::debug!("{:?}", cache);

    let result = run(database, cache).await;
    info!("Successfully start server 1!", );
    if result.is_err() {
        log::error!("{}", result.unwrap_err().to_string());
        std::process::exit(1)
    } else {
      info!("Successfully start server !", );
    }

    Ok(())
}

pub async fn run(database: Database, cache: CacheService) -> Result<(), Box<dyn Error>> {
    let app_environment = env::var("APP_ENVIRONMENT").unwrap_or("development".to_string());
    let app_host = env::var("APP_HOST").unwrap_or("0.0.0.0".to_string());
    let app_port = env::var("APP_PORT").unwrap_or("3000".to_string());
    let bind_address =format!("{app_host}:{app_port}");
    let listener = TcpListener::bind(&bind_address).await.unwrap();

    info!("environment config: {}", environment: app_environment);
    info!("host config: {}", host: app_host);
    info!("port config: {}", port: app_port);
    info!("listening on {}", address :bind_address);


    let app =  NormalizePathLayer::trim_trailing_slash().layer(router(database, cache.clone()));
    let app = ServiceExt::<Request>::into_make_service(app);
    axum::serve(listener, app)
        .await
        .unwrap();

    Ok(())
}

