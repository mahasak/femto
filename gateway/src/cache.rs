use moka::future::Cache;
use std::{env, time::Duration};

use crate::errors::AppError;

#[derive(Clone, Debug)]
pub struct CacheService {
    redis: redis::Client,
}

impl CacheService {
    pub async fn init() -> Self {
        let redis_url = env::var("REDIS_URL").expect("env::REDIS_URL is missing");
        let client = redis::Client::open(redis_url).expect("Error to init redis client");

        CacheService { redis: client }
    }

    pub async fn ping(&self) -> Result<String, AppError> {
        let mut con = self.redis.get_multiplexed_async_connection().await?;
        let pong: String = redis::cmd("PING").query_async(&mut con).await?;
        Ok(pong)
    }
}
