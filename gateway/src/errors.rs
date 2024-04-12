use log::error;
use thiserror::Error;
use tokio::task::JoinError;
use axum::{
  Router,
  Json,
  Extension,
  body::{self, Bytes},
  routing::get,
  http::StatusCode,
  response::{IntoResponse, Response},
};
use serde_json::json;

#[derive(Debug, Error)]
#[error("...")]
pub enum AppError {
    #[error("{}", _0)]
    NotFound(#[from] NotFound),

    #[error("{}", _0)]
    BadRequest(#[from] BadRequest),

    #[error("{}", _0)]
    InternalServerError(String),

    #[error("{0}")]
    RunSyncTask(#[from] JoinError),
}

impl AppError {
  fn get_codes(&self) -> (StatusCode, u16) {
    match *self {
      // 4XX Errors
      AppError::BadRequest(_) => (StatusCode::BAD_REQUEST, 40002),
      AppError::NotFound(_) => (StatusCode::NOT_FOUND, 40003),

      // 5XX Errors
      AppError::InternalServerError(_) => (StatusCode::INTERNAL_SERVER_ERROR, 5001),
      AppError::RunSyncTask(_) => (StatusCode::INTERNAL_SERVER_ERROR, 5005),
    }
  }

  pub fn bad_request() -> Self {
    AppError::BadRequest(BadRequest {})
  }

  pub fn not_found() -> Self {
    AppError::NotFound(NotFound {})
  }
}

impl From<redis::RedisError> for AppError {
  fn from(err: redis::RedisError) -> Self {
      AppError::InternalServerError(err.to_string())
  }
}

impl From<sqlx::Error> for AppError {
  fn from(err: sqlx::Error) -> Self {
      AppError::InternalServerError(err.to_string())
  }
}

impl IntoResponse for AppError {
  fn into_response(self) -> Response {
    let (status_code, code) = self.get_codes();
    let message = self.to_string();
    let body = Json(json!({ "code": code, "message": message }));

    (status_code, body).into_response()
  }
}

#[derive(thiserror::Error, Debug)]
#[error("Bad Request")]
pub struct BadRequest {}

#[derive(thiserror::Error, Debug)]
#[error("Not found")]
pub struct NotFound {}
