use super::cache::CacheError;
use super::database::DatabaseError;

pub type ServiceResult<T> = Result<T, ServiceError>;

pub enum ServiceError {
    NotFound,
    Internal(anyhow::Error),
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
