use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::response::Response;
use serde::Serialize;

use crate::error::service::ServiceError;

pub type ApiResult<T> = Result<T, ApiError>;

#[derive(Debug)]
pub enum ApiError {
    Internal(anyhow::Error),
    NotFound(String),
}

#[derive(Serialize)]
struct ResponseBody {
    message: String,
}

impl ResponseBody {
    pub fn from(message: &str) -> String {
        serde_json::to_string(&Self {
            message: message.to_string(),
        })
        .unwrap_or_default()
    }
}

// Tell axum how to convert `ApiError` into a response.
impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        match self {
            ApiError::Internal(error) => {
                let error = error.to_string();
                tracing::error!(error, "Internal Server Error");

                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    ResponseBody::from("Internal Server Error"),
                )
                    .into_response()
            }
            ApiError::NotFound(message) => {
                tracing::info!(error = message, "Not Found");

                (StatusCode::NOT_FOUND, ResponseBody::from("Not Found")).into_response()
            }
        }
    }
}

impl From<ServiceError> for ApiError {
    fn from(err: ServiceError) -> Self {
        let err = err;
        match err {
            ServiceError::NotFound => ApiError::NotFound("Not Found".to_string()),
            ServiceError::Internal(err) => ApiError::Internal(err),
        }
    }
}

// This enables using `?` on functions that return `Result<_, anyhow::Error>` to turn them into
// `Result<_, ApiError>`. That way you don't need to do that manually.
impl<E> From<E> for ApiError
where
    E: Into<anyhow::Error>,
{
    fn from(err: E) -> Self {
        Self::Internal(err.into())
    }
}
