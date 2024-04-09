use redis::RedisError;

pub type CacheResult<T> = Result<T, CacheError>;

pub enum CacheError {
    NotFound,
    Internal(anyhow::Error),
}

impl From<RedisError> for CacheError {
    fn from(err: RedisError) -> Self {
        CacheError::Internal(err.into())
    }
}
