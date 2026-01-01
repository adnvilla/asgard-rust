use crate::application::ports::RepoError;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct ErrorBody {
  pub error: String,
}

#[derive(Debug)]
pub struct ApiError {
  pub status: StatusCode,
  pub message: String,
}

impl ApiError {
  pub fn new(status: StatusCode, message: impl Into<String>) -> Self {
    Self { status, message: message.into() }
  }
}

impl From<RepoError> for ApiError {
  fn from(value: RepoError) -> Self {
    match value {
      RepoError::NotFound => ApiError::new(StatusCode::NOT_FOUND, "not found"),
      RepoError::Conflict => ApiError::new(StatusCode::CONFLICT, "conflict"),
      RepoError::Unexpected(msg) => ApiError::new(StatusCode::INTERNAL_SERVER_ERROR, msg),
    }
  }
}

impl IntoResponse for ApiError {
  fn into_response(self) -> Response {
    (self.status, Json(ErrorBody { error: self.message })).into_response()
  }
}


