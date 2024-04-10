use axum::Extension;
use axum::{routing::get, Router};
use axum::response::IntoResponse;
use domain::error::DomainError;
use dotenv::dotenv;
use redis::Client;
use repository::health::AppHealthRepository;
use services::{cache, db};
use sqlx::PgPool;
use std::env;
use std::error::Error;
use std::io;
use std::sync::{Arc, Mutex};
use std::thread;
use tokio::net::TcpListener;
use tower_http::cors::{Any, CorsLayer};

mod domain;
mod repository;
mod services;

#[tokio::main]
async fn main() -> io::Result<()> {
    dotenv().ok();
    env_logger::init();
    let pg_pool = db::init().await;
    let redis_client = cache::init();

    let result = run(pg_pool, redis_client).await;
    if result.is_err() {
        log::error!("{}", result.unwrap_err().to_string());
        std::process::exit(1)
    }

    Ok(())
}

pub async fn run(pg_pool: PgPool, redis_client: Client) -> Result<(), Box<dyn Error>> {
    println!("{:?}", pg_pool);
    println!("{:?}", redis_client);
    let app_environment = env::var("APP_ENVIRONMENT").unwrap_or("development".to_string());
    let app_host = env::var("APP_HOST").unwrap_or("0.0.0.0".to_string());
    let app_port = env::var("APP_PORT").unwrap_or("3000".to_string());
    println!("environment config: {}", app_environment);
    println!("host config: {}", &app_host);
    println!("port config: {}", &app_port);

    let bind_address = app_host + ":" + &app_port;
    let listener = TcpListener::bind(&bind_address).await.unwrap();

    println!("listening on {}", bind_address);

    axum::serve(listener, app(pg_pool, redis_client).into_make_service())
        .await
        .unwrap();
    Ok(())
}

pub fn app(pg_pool: PgPool, redis_client: Client) -> axum::Router {
    let cors = CorsLayer::new().allow_origin(Any);
    Router::new()
        .route("/", get(root))
        .route("/healthcheck", get(healthcheck))
        .layer(cors)
        .layer(Extension(pg_pool))
        .layer(Extension(redis_client))
}

async fn healthcheck(
    pg_pool: Extension<PgPool>,
    redis_client: Extension<redis::Client>,
) ->  axum::response::Result<String>  {
    let res: (String,) = sqlx::query_as("SELECT NOW()::VARCHAR;")
        .bind(150_i64)
        .fetch_one(&*pg_pool)
        .await.expect("");
    let date_now = res.0;

    let mut con = redis_client.get_multiplexed_async_connection().await.expect("");
    let pong: String = redis::cmd("PING").query_async(&mut con).await.expect("");

    return Ok(format!("POSTGRES: {date_now} REDIS: {pong}"));
}

async fn root() -> &'static str {
    "Hello, World!"
}
