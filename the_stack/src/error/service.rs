use std::fmt::Display;

use crate::error::cache::CacheError;
use crate::error::database::DatabaseError;

pub type ServiceResult<T> = Result<T, ServiceError>;

#[derive(Debug)]
pub enum ServiceError {
    NotFound,
    Internal(anyhow::Error),
}

impl Display for ServiceError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ServiceError::NotFound => write!(f, "ServiceError: Not Found"),
            ServiceError::Internal(err) => write!(f, "ServiceError: Internal: {}", err),
        }
    }
}

impl From<DatabaseError> for ServiceError {
    fn from(err: DatabaseError) -> Self {
        match err {
            DatabaseError::NotFound => ServiceError::NotFound,
            DatabaseError::Internal(err) => ServiceError::Internal(err),
        }
    }
}

impl From<CacheError> for ServiceError {
    fn from(err: CacheError) -> Self {
        match err {
            CacheError::NotFound => ServiceError::NotFound,
            CacheError::Internal(err) => ServiceError::Internal(err),
        }
    }
}

impl From<uuid::Error> for ServiceError {
    fn from(err: uuid::Error) -> Self {
        ServiceError::Internal(err.into())
    }
}
