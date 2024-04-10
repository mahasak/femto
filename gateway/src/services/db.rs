use std::env;
use sqlx::{PgPool, postgres::PgPoolOptions};

pub async fn init() -> PgPool {
    let database_url = env::var("DATABASE_URL").expect("env::DATABASE_URL is missing");

    PgPoolOptions::new()
        .max_connections(20)
        .connect(&database_url)
        .await
        .expect("Unable to connect to DATABASE")
}
