use redis_macros::FromRedisValue;
use redis_macros::ToRedisArgs;
use serde::Deserialize;
use serde::Serialize;
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct CouponSet {
    pub id: i64,
    pub name: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct CouponSetStatus {
    pub id: i64,
    pub name: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub total_coupons: i64,
}

impl CouponSet {
    pub fn set_key(id: i64) -> String {
        format!("thestack::coupons::{}", id)
    }

    pub fn used_key(id: i64) -> String {
        format!("thestack::used::{}", id)
    }

    pub fn used_key_prefix() -> &'static str {
        "thestack::used::*"
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow, FromRedisValue, ToRedisArgs)]
pub struct Coupon {
    pub id: Uuid,
    pub set_id: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateCouponSetDto {
    pub name: String,
}
