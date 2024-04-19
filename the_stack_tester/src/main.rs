pub mod bench;
pub mod fetch;
pub mod runner;
pub mod upload;

use std::time::Duration;

use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use tracing_subscriber::Layer;

use crate::upload::create_set;
use crate::upload::upload_coupons;

// TODO extract these to env vars
pub const TOTAL_UPLOAD: usize = 10000;
pub const TOTAL_SETS: usize = 20;
pub const WAIT_SECS: u64 = 5; // wait for data to be in DB
pub const BENCHMARK: bool = false;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
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

    let client = reqwest::Client::new();
    let mut sets = vec![];

    for i in 0..TOTAL_SETS {
        let set = create_set(&client, format!("Campaign {}", i)).await?;
        let coupons = upload_coupons(&client, set.id).await?;

        tracing::info!("Uploaded {} coupons to set {}", coupons.len(), set.id);

        sets.push((set, coupons));
    }

    tracing::info!(
        "Waiting {} seconds for data to be inserted into the database",
        WAIT_SECS
    );
    tokio::time::sleep(Duration::from_secs(WAIT_SECS)).await;

    if BENCHMARK {
        bench::run_benchmark(sets).await?;
    } else {
        runner::run_real_world_simulation(sets).await?;
    }

    Ok(())
}
