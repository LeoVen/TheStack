use std::fmt::Display;

pub type DatabaseResult<T> = Result<T, DatabaseError>;

#[derive(Debug)]
pub enum DatabaseError {
    NotFound,
    Internal(anyhow::Error),
}

impl Display for DatabaseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DatabaseError::NotFound => write!(f, "DatabaseError: Not Found"),
            DatabaseError::Internal(err) => write!(f, "DatabaseError: Internal: {}", err),
        }
    }
}

impl From<sqlx::Error> for DatabaseError {
    fn from(err: sqlx::Error) -> Self {
        match err {
            sqlx::Error::RowNotFound => Self::NotFound,
            _ => Self::Internal(err.into()),
        }
    }
}
