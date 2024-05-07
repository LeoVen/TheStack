use anyhow::Context;
use serde::Deserialize;

pub mod coupon;
pub mod userlogin;

#[derive(Deserialize, Clone)]
pub struct BatchInsertConfig {
    #[serde(rename(deserialize = "batch_insert_total"))]
    pub insert_total: i64,
    #[serde(rename(deserialize = "batch_insert_lock_prefix"))]
    pub lock_prefix: String,
}

impl BatchInsertConfig {
    pub fn new() -> anyhow::Result<Self> {
        envy::from_env().context("Failed to get env vars")
    }
}
