pub mod coupon;
pub mod userlogin;

use anyhow::Context;
use serde::Deserialize;
use serde::Serialize;
use sqlx::postgres::PgConnectOptions;
use sqlx::postgres::PgPoolOptions;
use sqlx::Pool;
use sqlx::Postgres;
use tokio_retry::strategy::FixedInterval;
use tokio_retry::Retry;

#[derive(Serialize, Deserialize, Debug)]
struct DatabasePostgresConfig {
    #[serde(rename(deserialize = "database_postgres_host"))]
    pub host: String,
    #[serde(rename(deserialize = "database_postgres_port"))]
    pub port: u16,
    #[serde(rename(deserialize = "database_postgres_username"))]
    pub username: String,
    #[serde(rename(deserialize = "database_postgres_password"))]
    pub password: String,
    #[serde(rename(deserialize = "database_postgres_database"))]
    pub database: String,
}

#[tracing::instrument]
pub async fn setup(env: &str) -> anyhow::Result<Pool<Postgres>> {
    tracing::info!("Setting up database");

    let config = envy::from_env::<DatabasePostgresConfig>().context("Failed to get env vars")?;

    if env == "dev" {
        let config_str = serde_json::to_string(&config).unwrap_or("Serialize Error".to_string());
        tracing::info!(config = config_str);
    }

    let conn_opt = PgConnectOptions::new()
        .host(&config.host)
        .port(config.port)
        .username(&config.username)
        .password(&config.password)
        .database(&config.database);

    let pool = Retry::spawn(FixedInterval::from_millis(1000).take(5), || {
        tracing::info!("Attempting PostgreSQL database connection");
        PgPoolOptions::new().connect_with(conn_opt.clone())
    })
    .await
    .context("Failed to connect to PostgreSQL")?;

    tracing::info!("Database setup finished");

    Ok(pool)
}
