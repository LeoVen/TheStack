use super::database::DatabaseError;

pub type ServiceResult<T> = Result<T, ServiceError>;

pub enum ServiceError {
    NotFound,
    Internal(anyhow::Error),
}

impl<E> From<E> for ServiceError
where
    E: Into<DatabaseError>,
{
    fn from(err: E) -> Self {
        let err = err.into();
        match err {
            DatabaseError::NotFound => ServiceError::NotFound,
            DatabaseError::Internal(err) => ServiceError::Internal(err),
        }
    }
}
