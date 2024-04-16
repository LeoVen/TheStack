use std::fmt::Display;

use password_hash::Error;

use crate::error::cache::CacheError;
use crate::error::database::DatabaseError;

pub type ServiceResult<T> = Result<T, ServiceError>;

#[derive(Debug)]
pub enum ServiceError {
    NotFound,
    Unauthorized,
    Conflict(anyhow::Error, String),
    Internal(anyhow::Error),
}

impl Display for ServiceError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ServiceError::NotFound => write!(f, "ServiceError: Not Found"),
            ServiceError::Internal(err) => write!(f, "ServiceError: Internal: {}", err),
            ServiceError::Unauthorized => write!(f, "ServiceError: Unauthorized"),
            ServiceError::Conflict(err, _) => {
                write!(f, "ServiceError: Conflict: {}", err)
            }
        }
    }
}

impl From<DatabaseError> for ServiceError {
    fn from(err: DatabaseError) -> Self {
        match err {
            DatabaseError::NotFound => ServiceError::NotFound,
            DatabaseError::Internal(err) => ServiceError::Internal(err),
            DatabaseError::ConstraintError(err, constraint) => {
                ServiceError::Conflict(err, constraint)
            }
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

impl From<Error> for ServiceError {
    fn from(err: Error) -> Self {
        match err {
            Error::Password => ServiceError::Unauthorized,
            _ => ServiceError::Internal(err.into()),
        }
    }
}
