use moka::future::Cache;
use std::{env, time::Duration};

use crate::errors::AppError;

#[derive(Clone, Debug)]
pub struct CacheService {
    redis: redis::Client,
    eligibility: Cache<String, bool>,
}

impl CacheService {
    pub async fn init() -> Self {
        let redis_url = env::var("REDIS_URL").expect("env::REDIS_URL is missing");
        let client = redis::Client::open(redis_url).expect("Error to init redis client");
        let eligibility: Cache<String, bool> = Cache::builder()
            .max_capacity(10_000) // Max 10,000 entries
            .time_to_live(Duration::from_secs(30 * 60)) // Time to live (TTL): 30 minutes
            .time_to_idle(Duration::from_secs(5 * 60)) // Time to idle (TTI):  5 minutes
            .build();

        CacheService {
            redis: client,
            eligibility: eligibility,
        }
    }

    pub async fn ping(&self) -> Result<String, AppError> {
        let mut con = self.redis.get_multiplexed_async_connection().await?;
        let pong: String = redis::cmd("PING").query_async(&mut con).await?;
        Ok(pong)
    }

    pub async fn is_eligible(&self, id: &str) -> Option<bool> {
      self.eligibility.get(id).await
    }

    pub async fn set_eligible(&self, id: &str, status: bool) {
      self.eligibility.insert(id.to_string(), status).await;
    }

    pub async fn remove_eligible(&self, id: &str) {
      self.eligibility.invalidate(id).await;
    }
}
