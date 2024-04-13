use crate::{errors::AppError, models::application::Application};
use sqlx::{postgres::PgPoolOptions, PgPool};
use std::env;

#[derive(Clone, Debug)]
pub struct Database {
    client: PgPool,
}

impl Database {
    pub async fn init() -> Self {
        let database_url = env::var("DATABASE_URL").expect("env::DATABASE_URL is missing");
        let client = PgPoolOptions::new()
            .max_connections(20)
            .connect(&database_url)
            .await
            .expect("Unable to connect to database");
        Database { client }
    }

    pub async fn get_now(&self) -> Result<String, AppError> {
        let res: (String,) = sqlx::query_as("SELECT NOW()::VARCHAR;")
            .bind(150_i64)
            .fetch_one(&self.client)
            .await
            .expect("");
        let date_now = res.0;

        Ok(date_now)
    }

    pub async fn get_applications(&self) -> Result<Vec<Application>, AppError> {
        let res = sqlx::query_as!(
            Application,
            "SELECT app_id, app_name, topic, enabled from application"
        )
        .fetch_all(&self.client)
        .await
        .expect("Error fetching application list");

        Ok(res)
    }
}

#[derive(Debug, thiserror::Error)]
#[error("sqlx error: {0}")]
pub struct DbError(#[from] sqlx::Error);