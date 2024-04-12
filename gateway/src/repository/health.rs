use std::sync::Arc;

use async_trait::async_trait;
use sqlx::PgPool;

use crate::domain::{health::repository::HealthRepository};
use crate::errors::AppError;
pub struct AppHealthRepository {
  pg_pool: Arc<sqlx::PgPool>,
  redis_client: Arc<redis::Client>
}

impl AppHealthRepository {
    pub fn new(pg_pool: Arc<sqlx::PgPool>, redis_client: Arc<redis::Client>) -> Self {
      Self { pg_pool, redis_client}
    }
}

#[async_trait]
impl HealthRepository for AppHealthRepository {
  async fn get_now(&self) -> Result<String, AppError> {
    let res:(String,) = sqlx::query_as("SELECT NOW()::VARCHAR;")
        .bind(150_i64)
        .fetch_one(&*self.pg_pool).await?;
    let response = res.0;
    Ok(response)
  }

  async fn ping(&self) -> Result<String, AppError> {
    let mut con = self.redis_client.get_multiplexed_async_connection().await?;
    let pong: String = redis::cmd("PING").query_async(&mut con).await?;
    Ok(pong)
  }
}