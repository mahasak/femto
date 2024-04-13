use crate::{
    errors::AppError, models::application::Application, models::merchant_channel::MerchantChannel,
};
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
        Ok(eligible)
    }
}

#[derive(Debug, thiserror::Error)]
#[error("sqlx error: {0}")]
pub struct DbError(#[from] sqlx::Error);
