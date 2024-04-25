pub mod bench;
pub mod fetch;
pub mod runner;
pub mod upload;

use std::time::Duration;

use anyhow::Context;
use serde::Deserialize;
use serde::Serialize;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use tracing_subscriber::Layer;

use crate::upload::create_set;
use crate::upload::upload_coupons;

#[derive(Serialize, Deserialize, Debug, Clone)]
enum TesterMode {
    #[serde(rename(deserialize = "benchmark"))]
    Benchmark,
    #[serde(rename(deserialize = "simulation"))]
    Simulation,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct TesterConfig {
    #[serde(rename(deserialize = "tester_total_uploads"))]
    pub total_uploads: usize,
    #[serde(rename(deserialize = "tester_total_sets"))]
    pub total_sets: usize,
    #[serde(rename(deserialize = "tester_wait_secs"))]
    pub wait_secs: u64,
    #[serde(rename(deserialize = "tester_timeout_milliseconds"))]
    pub timeout: u64,
    #[serde(rename(deserialize = "tester_mode"))]
    pub mode: TesterMode,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenvy::dotenv().ok();

    tracing_subscriber::registry()
        .with(
            tracing_subscriber::fmt::layer()
                .with_filter(tracing_subscriber::filter::LevelFilter::INFO),
        )
        .with(
            console_subscriber::ConsoleLayer::builder()
                .retention(Duration::from_secs(60))
                .spawn(),
        )
        .init();

    let config = envy::from_env::<TesterConfig>().context("Failed to get env vars")?;

    tracing::info!("Running in {:?} mode", config.mode);

    let client = reqwest::Client::new();
    let mut sets = vec![];

    for i in 0..config.total_sets {
        let set = create_set(&client, format!("Campaign {}", i)).await?;
        let coupons = upload_coupons(&client, set.id, config.total_uploads).await?;

        tracing::info!("Uploaded {} coupons to set {}", coupons.len(), set.id);

        sets.push((set, coupons));
    }

    tracing::info!(
        "Waiting {} seconds for data to be inserted into the database",
        config.wait_secs
    );
    tokio::time::sleep(Duration::from_secs(config.wait_secs)).await;

    match config.mode {
        TesterMode::Benchmark => bench::run_benchmark(config, sets).await?,
        TesterMode::Simulation => runner::simulation(config, sets).await?,
    }

    Ok(())
}
