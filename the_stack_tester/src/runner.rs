use std::collections::HashSet;

use anyhow::Error;
use itertools::Itertools;
use rand::Rng;
use the_stack::model::coupon::CouponSet;
use tokio::task::JoinSet;

use crate::fetch::fetch_coupon;
use crate::fetch::FetchResult;
use crate::TOTAL_UPLOAD;

#[tracing::instrument(skip_all)]
pub async fn run_real_world_simulation(sets: Vec<(CouponSet, Vec<String>)>) -> anyhow::Result<()> {
    // Chunk coupons into sets.len() and distribute a bit of every set for each job
    let len = sets.len();
    let chunk_size = TOTAL_UPLOAD / len;

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
        js.spawn(async { fetch_all(set).await });
    }

    while let Some(result) = js.join_next().await {
        match result {
            Ok(ret) => match ret {
                Ok(_) => {}
                Err(err) => tracing::error!("{}", err),
            },
            Err(err) => tracing::error!("{}", err),
        }
    }

    Ok(())
}

#[tracing::instrument(skip_all)]
async fn fetch_all(data: Vec<(CouponSet, Vec<String>)>) -> anyhow::Result<()> {
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
            break;
        }

        // TODO add total_errors to env var
        if total_errors > 100 {
            return Err(Error::msg("TOO MANY ERRORS!"));
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
                    tracing::error!(
                        "Set {} still has {} coupons to fetch but got none from API",
                        set_id,
                        selected.1.len()
                    );
                    continue;
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
        // let millis = {
        //     let mut rng = rand::thread_rng();
        //     rng.gen_range(1..100)
        // };
        // tokio::time::sleep(Duration::from_millis(millis)).await;
    }

    Ok(())
}
