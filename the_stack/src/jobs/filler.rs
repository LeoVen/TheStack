use std::sync::Arc;
use std::sync::Mutex;

use anyhow::Context;
use anyhow::Result;
use redis::aio::MultiplexedConnection;

use serde::Deserialize;
use sqlx::Pool;
use sqlx::Postgres;

use crate::metrics::Metrics;

#[derive(Deserialize, Debug)]
pub struct WorkerConfig {
    #[serde(rename(deserialize = "worker_timeout_seconds"))]
    pub timeout_seconds: u64,
}

#[tracing::instrument(skip_all)]
pub fn setup(
    cache: MultiplexedConnection,
    db: Pool<Postgres>,
    metrics: Metrics,
) -> Result<Arc<Mutex<u64>>> {
    tracing::info!("Setting up worker");

    let config = envy::from_env::<WorkerConfig>().context("Failed to get env vars")?;

    let timeout = Arc::new(Mutex::new(config.timeout_seconds));

    {
        let timeout = timeout.clone();
        tokio::task::spawn(async move {
            filler_worker(cache, db, metrics, timeout).await;
        });
    }

    Ok(timeout)
}

#[tracing::instrument(skip_all)]
pub async fn filler_worker(
    _cache: MultiplexedConnection,
    _db: Pool<Postgres>,
    _metrics: Metrics,
    _timeout: Arc<Mutex<u64>>,
) {
    // Fills up the redis cache before they run out of coupons
    todo!()
}
