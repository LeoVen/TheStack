use std::fmt::Display;

use redis::RedisError;

pub type CacheResult<T> = Result<T, CacheError>;

#[derive(Debug)]
pub enum CacheError {
    NotFound,
    Internal(anyhow::Error),
}

impl Display for CacheError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CacheError::NotFound => write!(f, "CacheError: Not Found"),
            CacheError::Internal(err) => write!(f, "CacheError: Internal: {}", err),
        }
    }
}

impl From<RedisError> for CacheError {
    fn from(err: RedisError) -> Self {
        CacheError::Internal(err.into())
    }
}
