use std::collections::HashSet;
use std::time::Duration;

use itertools::Itertools;
use rand::Rng;
use the_stack::model::coupon::CouponSet;
use tokio::task::JoinSet;

use crate::fetch::fetch_coupon;
use crate::fetch::FetchResult;
use crate::TesterConfig;

#[tracing::instrument(skip_all)]
pub async fn simulation(
    config: TesterConfig,
    sets: Vec<(CouponSet, Vec<String>)>,
) -> anyhow::Result<()> {
    // Chunk coupons into sets.len() and distribute a bit of every set for each job
    let len = sets.len();
    let chunk_size = config.total_uploads / len;

    let mut chunked: Vec<(CouponSet, Vec<Vec<String>>)> = sets
        .into_iter()
        .map(|(set, coupons)| {
            let chunks = coupons
                .into_iter()
                .chunks(chunk_size)
                .into_iter()
                .map(|c| c.collect())
                .collect();

            (set, chunks)
        })
        .collect();

    let mut chunked_sets: Vec<Vec<(CouponSet, Vec<String>)>> = vec![];

    for _ in 0..len {
        let mut nth_chunk = vec![];

        for chunk in chunked.iter_mut() {
            let coupons = chunk.1.pop().unwrap_or_default();

            nth_chunk.push((chunk.0.clone(), coupons));
        }

        chunked_sets.push(nth_chunk);
    }

    let mut js = JoinSet::new();
    for set in chunked_sets.into_iter() {
        let config = config.clone();
        js.spawn(async { fetch_all(config, set).await });
    }

    while let Some(result) = js.join_next().await {
        match result {
            Ok(ret) => match ret {
                Ok(data) => {
                    for data in data.into_iter() {
                        tracing::error!("Set {} still has {} coupons", data.0.id, data.1.len());
                    }
                }
                Err(err) => tracing::error!("{}", err),
            },
            Err(err) => tracing::error!("{}", err),
        }
    }

    Ok(())
}

#[tracing::instrument(skip_all)]
async fn fetch_all(
    config: TesterConfig,
    data: Vec<(CouponSet, Vec<String>)>,
) -> anyhow::Result<Vec<(CouponSet, HashSet<String>)>> {
    let total_coupons = data.iter().fold(0, |acc, (_, coupons)| acc + coupons.len());

    tracing::info!(
        "Fetching randomly from {} chunked sets with {} coupons in total",
        data.len(),
        total_coupons,
    );

    let mut data: Vec<(CouponSet, HashSet<String>)> = data
        .into_iter()
        .map(|(set, coupons)| (set, HashSet::from_iter(coupons.into_iter())))
        .collect();

    let client = reqwest::Client::new();

    let mut pct = 0.1;
    let mut total_errors = 0;

    loop {
        if data.is_empty() {
            tracing::info!("Data is empty. Stopping...");
            break;
        }

        // TODO add total_errors to env var
        if total_errors > 1000 {
            tracing::error!("TOO MANY ERRORS!");
            return Ok(data);
        }

        // Randomly select a coupon set to extract from
        let idx = {
            let mut rng = rand::thread_rng();
            rng.gen_range(0..data.len())
        };

        let selected = &mut data[idx];
        let set_id = selected.0.id;

        if selected.1.is_empty() {
            let _ = data.remove(idx);
            continue;
        }

        let coupon = match fetch_coupon(&client, selected.0.id).await? {
            FetchResult::Coupon(coupon) => coupon,
            FetchResult::StatusError(status) => {
                total_errors += 1;

                if status == 404 {
                    continue; // TODO should never happen
                }

                break;
            }
        };

        selected.1.remove(&coupon.id.to_string());

        let rem = data.iter().fold(0, |acc, (_, coupons)| acc + coupons.len());
        if (rem as f64 / total_coupons as f64) <= (1.0 - pct) {
            tracing::info!("[{}] Processed {:.2}% ({} left)", set_id, pct * 100.0, rem);

            pct += 0.1;
        }

        // // Random timeout to simulate "real world usage" :D
        if config.timeout > 0 {
            let millis = {
                let mut rng = rand::thread_rng();
                rng.gen_range(0..config.timeout)
            };
            tokio::time::sleep(Duration::from_millis(millis)).await;
        }
    }

    Ok(data)
}
