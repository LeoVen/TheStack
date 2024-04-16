use std::fmt::Display;

pub type DatabaseResult<T> = Result<T, DatabaseError>;

#[derive(Debug)]
pub enum DatabaseError {
    NotFound,
    ConstraintError(anyhow::Error, String),
    Internal(anyhow::Error),
}

impl Display for DatabaseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DatabaseError::NotFound => write!(f, "DatabaseError: Not Found"),
            DatabaseError::Internal(err) => write!(f, "DatabaseError: Internal: {}", err),
            DatabaseError::ConstraintError(err, constraint) => {
                write!(f, "DatabaseError: Constraint on {}: {}", constraint, err)
            }
        }
    }
}

impl From<sqlx::Error> for DatabaseError {
    fn from(err: sqlx::Error) -> Self {
        match err {
            sqlx::Error::RowNotFound => Self::NotFound,
            sqlx::Error::Database(err) => {
                if let Some(constraint) = err.constraint() {
                    let constraint = constraint.to_string();
                    Self::ConstraintError(err.into(), constraint)
                } else {
                    Self::Internal(err.into())
                }
            }
            _ => Self::Internal(err.into()),
        }
    }
}
