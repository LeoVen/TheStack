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

impl CouponSet {
    pub fn set_key(id: i64) -> String {
        format!("thestack::coupons::{}", id)
    }

    pub fn set_key_prefix() -> &'static str {
        "thestack::coupons::*"
    }

    pub fn used_key(id: i64) -> String {
        format!("thestack::used::{}", id)
    }

    pub fn used_key_prefix() -> &'static str {
        "thestack::used::*"
    }

    pub fn extract_set_id(key: &str) -> i64 {
        // TODO how to better treat -1 ids
        let id_str = key.split("::").nth(2).unwrap_or("-1");

        str::parse(id_str).unwrap_or(-1)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow, FromRedisValue, ToRedisArgs)]
pub struct Coupon {
    pub id: Uuid,
    pub set_id: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct CouponSetDatabaseStatus {
    pub id: i64,
    pub name: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub total_coupons: i64,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CouponSetCacheStatus {
    pub id: i64,
    pub total_coupons: i64,
}
