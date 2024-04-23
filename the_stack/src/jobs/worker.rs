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
use uuid::Uuid;

use crate::cache::coupon::CouponCache;
use crate::database::coupon::CouponRepository;
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

        if keys.is_empty() {
            let timeout = *timeout.lock().expect("Could not acquire lock for timeout");
            tracing::info!(timeout, "nothing to cleanup and now waiting");
            tokio::time::sleep(Duration::from_secs(timeout)).await;
            continue;
        }

        tracing::info!("cleaning up {} coupon sets", &keys.len());

        let mut coupon_cache = CouponCache::new(cache.clone());
        let mut coupons_to_delete = vec![];

        for key in keys.iter() {
            let Ok(mut coupons) = coupon_cache.pop_coupon_list(key).await else {
                continue;
            };

            coupons_to_delete.append(&mut coupons);
        }

        // TODO this number does not match rows_affected
        tracing::info!(
            "cleaning up {} coupons from the database",
            coupons_to_delete.len()
        );

        let coupon_database = CouponRepository::new(db.clone());

        let coupons_to_delete = coupons_to_delete
            .iter()
            .map(|value| Uuid::try_parse(value).expect("Could not parse UUID"))
            .collect::<Vec<Uuid>>();

        let result = coupon_database.delete_coupons(&coupons_to_delete).await;

        match result {
            Ok(rows_affected) => tracing::info!(rows_affected, "coupons deleted from the database"),
            Err(error) => {
                let error = error.to_string();
                tracing::error!(error, "error when deleting coupons from the database");
            }
        }

        let timeout = *timeout.lock().expect("Could not acquire lock for timeout");
        tracing::info!(timeout, "cleaning up finished and now waiting");
        tokio::time::sleep(Duration::from_secs(timeout)).await;
    }
}
