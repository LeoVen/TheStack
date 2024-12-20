pub mod coupon;
pub mod lock;

use anyhow::Context;
use anyhow::Result;
use redis::aio::MultiplexedConnection;
use redis::ConnectionAddr;
use redis::ConnectionInfo;
use redis::RedisConnectionInfo;
use serde::Deserialize;
use serde::Serialize;

#[derive(Serialize, Deserialize, Debug)]
struct RedisConfig {
    #[serde(rename(deserialize = "cache_redis_host"))]
    pub host: String,
    #[serde(rename(deserialize = "cache_redis_port"))]
    pub port: u16,
    #[serde(rename(deserialize = "cache_redis_database"))]
    pub database: i64,
}

#[tracing::instrument]
pub async fn setup(env: &str) -> Result<(MultiplexedConnection, lock::DistributedLock)> {
    tracing::info!("Setting up redis cache");

    let config = envy::from_env::<RedisConfig>().context("Failed to get env vars")?;

    if env == "dev" {
        let config_str = serde_json::to_string(&config).unwrap_or("Serialize Error".to_string());
        tracing::info!(config = config_str);
    }

    let client = redis::Client::open(ConnectionInfo {
        addr: ConnectionAddr::Tcp(config.host.clone(), config.port),
        redis: RedisConnectionInfo {
            db: config.database,
            username: None,
            password: None,
            protocol: redis::ProtocolVersion::RESP3,
        },
    })
    .context("Failed to setup redis connection manager")?;

    let conn = client
        .get_multiplexed_tokio_connection()
        .await
        .context("Failed to get redis multiplexed connection")?;

    let lock = lock::DistributedLock::new(&config);

    tracing::info!("Redis cache setup finished");

    Ok((conn, lock))
}
