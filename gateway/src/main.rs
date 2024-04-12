use axum::{routing::get, Router};
use cache::Cache;
use database::Database;
use dotenv::dotenv;
use handlers::{healthcheck, SharedState};
use std::{env, error::Error, io};
use tokio::net::TcpListener;
use tower_http::cors::{Any, CorsLayer};

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
    let database = Database::init().await;
    let cache = Cache::init().await;

    let result = run(database, cache).await;
    if result.is_err() {
        log::error!("{}", result.unwrap_err().to_string());
        std::process::exit(1)
    }

    Ok(())
}

pub async fn run(database: Database, cache: Cache) -> Result<(), Box<dyn Error>> {
    println!("{:?}", database);
    println!("{:?}", cache);
    let app_environment = env::var("APP_ENVIRONMENT").unwrap_or("development".to_string());
    let app_host = env::var("APP_HOST").unwrap_or("0.0.0.0".to_string());
    let app_port = env::var("APP_PORT").unwrap_or("3000".to_string());
    println!("environment config: {}", app_environment);
    println!("host config: {}", &app_host);
    println!("port config: {}", &app_port);

    let bind_address = app_host + ":" + &app_port;
    let listener = TcpListener::bind(&bind_address).await.unwrap();

    println!("listening on {}", bind_address);

    axum::serve(listener, app(database, cache).into_make_service())
        .await
        .unwrap();
    Ok(())
}

pub fn app(database: Database, cache: Cache) -> Router {
    let cors = CorsLayer::new().allow_origin(Any);
    Router::new()
        .route("/", get(handlers::root))
        .route("/healthcheck", get(healthcheck))
        .layer(cors)
        .with_state(SharedState { database, cache })
}
