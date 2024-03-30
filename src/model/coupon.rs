use redis_macros::{FromRedisValue, ToRedisArgs};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Serialize, Deserialize, FromRow, Clone, FromRedisValue, ToRedisArgs)]
pub struct CouponSet {
    pub id: i64,
    pub name: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Serialize, Deserialize, FromRow, Clone, FromRedisValue, ToRedisArgs)]
pub struct Coupon {
    pub id: Uuid,
    pub set_id: i64,
    pub used: bool,
}

impl Coupon {
    pub fn redis_key(id: i64) -> String {
        format!("thestack:coupons:{}", id)
    }
}

#[derive(Serialize, Deserialize, FromRow, Clone, FromRedisValue, ToRedisArgs)]
pub struct CreateCouponSetDto {
    pub name: String,
}
