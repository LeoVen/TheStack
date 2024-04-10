use std::sync::Arc;
use std::sync::Mutex;
use std::time::Duration;

use anyhow::Context;
use anyhow::Result;
use redis::aio::MultiplexedConnection;
use redis::AsyncCommands;
use serde::Deserialize;
use sqlx::Pool;
use sqlx::Postgres;

use crate::metrics::Metrics;
use crate::model::coupon::CouponSet;

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
            cleanup_worker(cache, db, metrics, timeout).await;
        });
    }

    Ok(timeout)
}

#[tracing::instrument(skip_all)]
pub async fn cleanup_worker(
    mut cache: MultiplexedConnection,
    db: Pool<Postgres>,
    metrics: Metrics,
    timeout: Arc<Mutex<u64>>,
) {
    tracing::info!("starting cleanup worker loop");

    loop {
        tracing::info!("cleaning up used coupons");
        metrics.job_cleanup.inc();

        let keys = match cache
            .keys::<_, Vec<String>>(CouponSet::used_key_prefix())
            .await
        {
            Ok(result) => result,
            Err(err) => {
                let error = anyhow::Error::from(err).to_string();
                tracing::error!(error, "Error getting all keys for cleanup");
                continue;
            }
        };

        // TODO
        let _ = db;
        println!("{:?}", keys);

        let timeout = *timeout.lock().unwrap();
        tracing::info!(timeout, "cleaning up finished and now waiting");
        tokio::time::sleep(Duration::from_secs(timeout)).await;
    }
}
