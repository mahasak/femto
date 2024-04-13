use crate::{
    errors::AppError, models::application::Application, models::merchant_channel::MerchantChannel,
};
use moka::future::Cache;
use sqlx::{postgres::PgPoolOptions, PgPool};
use std::{env, time::Duration};

#[derive(Clone, Debug)]
pub struct Database {
    client: PgPool,
    eligibility: Cache<String, bool>,
}

impl Database {
    pub async fn init() -> Self {
        let database_url = env::var("DATABASE_URL").expect("env::DATABASE_URL is missing");
        let client = PgPoolOptions::new()
            .max_connections(20)
            .connect(&database_url)
            .await
            .expect("Unable to connect to database");
        let eligibility: Cache<String, bool> = Cache::builder()
            .max_capacity(10_000) // Max 10,000 entries
            .time_to_live(Duration::from_secs(30 * 60)) // Time to live (TTL): 30 minutes
            .time_to_idle(Duration::from_secs(5 * 60)) // Time to idle (TTI):  5 minutes
            .build();

        Database {
            client,
            eligibility,
        }
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

    pub async fn get_application(&self, app_id: String) -> Result<Option<Application>, AppError> {
        let res = sqlx::query_as!(
            Application,
            "SELECT app_id, app_name, topic, enabled from application WHERE app_id = $1",
            app_id
        )
        .fetch_optional(&self.client)
        .await
        .expect("Error fetching application by id");

        Ok(res)
    }

    pub async fn get_merchant_channels(&self) -> Result<Vec<MerchantChannel>, AppError> {
        let res = sqlx::query_as!(
            MerchantChannel,
            "SELECT id, ref_id, name, ref_type, token from merchant_channel"
        )
        .fetch_all(&self.client)
        .await
        .expect("Error fetching merchant channel list");

        Ok(res)
    }

    pub async fn get_merchant_channel(
        &self,
        ref_id: String,
    ) -> Result<Option<MerchantChannel>, AppError> {
        let res = sqlx::query_as!(
            MerchantChannel,
            "SELECT id, ref_id, name, ref_type, token from merchant_channel where ref_id = $1",
            ref_id
        )
        .fetch_optional(&self.client)
        .await
        .expect("Error fetching merchant channel by id");

        Ok(res)
    }

    pub async fn is_merchant_channel_eligible(&self, ref_id: String) -> Result<bool, AppError> {
        let cache_result = self.eligibility.get(&ref_id).await;

        let eligible = match cache_result {
            Some(cache_result) => {
                println!("cache hit");
                cache_result
            }
            None => {
                println!("cache missed");
                let res = sqlx::query!(
                    "SELECT COUNT(*) from merchant_channel where ref_id = $1",
                    ref_id
                )
                .fetch_one(&self.client)
                .await
                .expect("Error fetching merchant channel by id");
                let count = res.count;

                let eligible = match count {
                    Some(count) => count > 0,
                    None => false,
                };
                self.eligibility.insert(ref_id.to_string(), eligible).await;

                eligible
            }
        };

        Ok(eligible)
    }

    pub async fn remove_eligible(&self, id: &str) {
        self.eligibility.invalidate(id).await;
    }

    pub async fn flush_eligible(&self) {
        self.eligibility.invalidate_all();
    }

    pub async fn get_sequence(&self, id: &str) -> Result<i32, AppError> {
        let mut tx = self.client.begin().await?;

        sqlx::query!("SELECT * FROM sequencers WHERE name = $1 FOR UPDATE", id)
            .fetch_one(&mut *tx)
            .await?;

        sqlx::query!("UPDATE sequencers SET data = data + 1 WHERE name = $1", id)
            .execute(&mut *tx)
            .await?;

        tx.commit().await?;

        let res = sqlx::query!("SELECT data from sequencers where name = $1", id)
            .fetch_one(&self.client)
            .await
            .expect("Error fetching merchant channel by id");
        let seq = res.data;

        Ok(seq)
    }
}

#[derive(Debug, thiserror::Error)]
#[error("sqlx error: {0}")]
pub struct DbError(#[from] sqlx::Error);
