pub mod runner;
pub mod upload;

use std::time::Duration;

use crate::upload::create_set;
use crate::upload::upload_coupons;

pub const TOTAL_UPLOAD: usize = 50000;
pub const TOTAL_SETS: usize = 20;
pub const WAIT_SECS: u64 = 10; // wait for data to be in DB

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
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

    runner::run_benchmark(sets).await?;

    Ok(())
}
