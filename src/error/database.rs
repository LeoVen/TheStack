pub type DatabaseResult<T> = Result<T, DatabaseError>;

pub enum DatabaseError {
    NotFound,
    Internal(anyhow::Error),
}

impl From<sqlx::Error> for DatabaseError {
    fn from(err: sqlx::Error) -> Self {
        match err {
            sqlx::Error::RowNotFound => Self::NotFound,
            _ => Self::Internal(err.into()),
        }
    }
}
