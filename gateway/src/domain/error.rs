use log::error;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum DomainError {
    #[error("{}", _0)]
    NotFound(String),

    #[error("{}", _0)]
    BadRequest(String),

    #[error("{}", _0)]
    InternalServerError(String),
}

impl From<redis::RedisError> for DomainError {
  fn from(err: redis::RedisError) -> Self {
      DomainError::InternalServerError(err.to_string())
  }
}

impl From<sqlx::Error> for DomainError {
  fn from(err: sqlx::Error) -> Self {
      DomainError::InternalServerError(err.to_string())
  }
}
