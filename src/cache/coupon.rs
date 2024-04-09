use redis::aio::MultiplexedConnection;
use redis::AsyncCommands;
use redis::Direction;

use crate::error::cache::CacheResult;
use crate::model::coupon::Coupon;
use crate::model::coupon::CouponSet;

#[derive(Clone)]
pub struct CouponCache {
    conn: MultiplexedConnection,
}

impl CouponCache {
    pub fn new(conn: MultiplexedConnection) -> Self {
        Self { conn }
    }

    #[tracing::instrument(skip(self, coupons))]
    pub async fn batch_insert(&mut self, set_id: i64, coupons: &[Coupon]) -> CacheResult<()> {
        if coupons.is_empty() {
            tracing::warn!("can't batch insert because there are no more coupons!");
            return Ok(());
        }

        tracing::info!("batch inserting {} coupons", coupons.len());

        self.conn
            .lpush(
                CouponSet::set_key(set_id),
                coupons
                    .iter()
                    .map(|c| c.id.to_string())
                    .collect::<Vec<String>>(),
            )
            .await?;

        Ok(())
    }

    pub async fn get_by_id(&mut self, id: i64) -> CacheResult<Option<String>> {
        let result: Option<Vec<String>> = self.conn.get(Coupon::redis_key(id)).await?;

        Ok(result.and_then(|v| v.into_iter().next()))
    }

    pub async fn pop_coupon(&mut self, set_id: i64) -> CacheResult<Option<String>> {
        let result = self
            .conn
            .lmove(
                CouponSet::set_key(set_id),
                CouponSet::used_key(set_id),
                Direction::Right,
                Direction::Left,
            )
            .await?;

        Ok(result)
    }
}
