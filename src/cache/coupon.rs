use redis::aio::MultiplexedConnection;
use redis::AsyncCommands;
use redis::RedisResult;

use crate::model::coupon::Coupon;

#[derive(Clone)]
pub struct CouponCache {
    conn: MultiplexedConnection,
}

impl CouponCache {
    pub fn new(conn: MultiplexedConnection) -> Self {
        Self { conn }
    }

    pub async fn get_by_id(&mut self, id: i64) -> RedisResult<Option<String>> {
        let result: Option<Vec<String>> = self.conn.get(Coupon::redis_key(id)).await?;

        Ok(result.and_then(|v| v.into_iter().next()))
    }
}
