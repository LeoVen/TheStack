use std::num::NonZeroUsize;

use redis::aio::MultiplexedConnection;
use redis::AsyncCommands;
use redis::Direction;

use crate::error::cache::CacheResult;
use crate::model::coupon::Coupon;
use crate::model::coupon::CouponSet;
use crate::model::coupon::CouponSetCacheStatus;

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

        let _: () = self
            .conn
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

    pub async fn pop_coupon_list(&mut self, key: &str) -> CacheResult<Vec<String>> {
        let result = self.conn.lpop(key, NonZeroUsize::new(10000)).await?;

        Ok(result)
    }

    pub async fn set_status(&mut self) -> CacheResult<Vec<CouponSetCacheStatus>> {
        let keys = self
            .conn
            .keys::<_, Vec<String>>(CouponSet::set_key_prefix())
            .await?;

        tracing::info!("found {} keys", keys.len());

        let mut pipe = redis::Pipeline::with_capacity(keys.len());

        for key in keys.iter() {
            pipe.llen(key);
        }

        let lengths: Vec<i64> = pipe.query_async(&mut self.conn).await?;

        // Assume that keys.len() == lengths.len()
        let result = keys
            .into_iter()
            .zip(lengths)
            .map(|(key, len)| CouponSetCacheStatus {
                id: CouponSet::extract_set_id(&key),
                total_coupons: len,
            })
            .collect();

        Ok(result)
    }
}
