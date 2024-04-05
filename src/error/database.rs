pub type DatabaseResult<T> = Result<T, DatabaseError>;

pub enum DatabaseError {
    NotFound,
    Internal(anyhow::Error),
}

impl<E> From<E> for DatabaseError
where
    E: Into<sqlx::Error>,
{
    fn from(err: E) -> Self {
        let err: sqlx::Error = err.into();
        match err {
            sqlx::Error::RowNotFound => Self::NotFound,
            _ => Self::Internal(err.into()),
        }
    }
}
