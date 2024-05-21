use std::fmt::Display;

use anyhow::anyhow;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::response::Response;
use serde::Serialize;

use crate::error::service::ServiceError;

pub type ApiResult<T> = Result<T, ApiError>;

#[derive(Debug)]
pub enum ApiError {
    Internal(anyhow::Error),
    Conflict(anyhow::Error, String),
    NotFound(String),
    BadRequest(String),
    Unauthorized {
        message: String,
        error: Option<anyhow::Error>,
    },
}

impl ApiError {
    pub fn default_unauthorized() -> Self {
        Self::Unauthorized {
            message: "Unauthorized".to_string(),
            error: None,
        }
    }
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

impl Display for ApiError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ApiError::Internal(err) => write!(f, "ApiError: Internal: {}", err),
            ApiError::NotFound(message) => write!(f, "ApiError: NotFound: {}", message),
            ApiError::BadRequest(message) => write!(f, "ApiError: BadRequest: {}", message),
            ApiError::Unauthorized { message, error } => {
                write!(
                    f,
                    "ApiError: Unauthorized: {}: {}",
                    message,
                    error.as_ref().unwrap_or(&anyhow!("Error!"))
                )
            }
            ApiError::Conflict(err, _) => write!(f, "ApiError: Conflict: {}", err),
        }
    }
}

// Tell axum how to convert `ApiError` into a response.
impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        match self {
            // https://docs.rs/anyhow/latest/anyhow/struct.Error.html#display-representations
            ApiError::Internal(error) => {
                tracing::error!("Internal Server Error: {:#}", error);

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
            ApiError::BadRequest(message) => {
                tracing::error!(message, "Bad Request");

                (
                    StatusCode::BAD_REQUEST,
                    ResponseBody::from(message.as_str()),
                )
                    .into_response()
            }
            ApiError::Unauthorized { message, error } => {
                let error = error.unwrap_or(anyhow!("Error")).to_string();
                tracing::info!(message, error, "Unauthorized");

                (StatusCode::UNAUTHORIZED, ResponseBody::from("Unauthorized")).into_response()
            }
            ApiError::Conflict(error, constraint) => {
                let error = error.to_string();

                tracing::info!(error, "Conflict on {}", constraint);

                (
                    StatusCode::CONFLICT,
                    ResponseBody::from("Resource Already Exists"),
                )
                    .into_response()
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
            ServiceError::Unauthorized => ApiError::default_unauthorized(),
            ServiceError::Conflict(err, conflict) => ApiError::Conflict(err, conflict),
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
