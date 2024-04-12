use async_trait::async_trait;

use crate::errors::AppError;

#[async_trait]
pub trait HealthRepository: Send + Sync {
    async fn get_now(&self) -> Result<String, AppError>;
    async fn ping(&self) -> Result<String, AppError>;
}
