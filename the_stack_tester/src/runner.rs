use std::collections::BTreeMap;
use std::collections::BTreeSet;
use std::time::Duration;

use rand::Rng;
use reqwest::StatusCode;
use the_stack::model::coupon::CouponSet;
use tokio::task::JoinSet;

use crate::fetch::fetch_coupon;
use crate::fetch::FetchResult;
use crate::TesterConfig;

type SetData = BTreeMap<i64, BTreeSet<String>>;

#[tracing::instrument(skip_all)]
pub async fn simulation(
    config: TesterConfig,
    sets: Vec<(CouponSet, Vec<String>)>,
) -> anyhow::Result<()> {
    let reference: Vec<i64> = sets.iter().map(|set_data| set_data.0.id).collect();
    let set_data: SetData = sets.into_iter().fold(BTreeMap::new(), |mut acc, set| {
        acc.entry(set.0.id)
            .or_insert(BTreeSet::from_iter(set.1.into_iter()));
        acc
    });

    let mut js = JoinSet::new();
    for id in 0..config.total_sets {
        let config = config.clone();
        let reference = reference.clone();
        js.spawn(async move { fetch_all(id, config, reference).await });
    }

    let mut merged_data: SetData = BTreeMap::new();

    while let Some(result) = js.join_next().await {
        match result {
            Ok(ret) => match ret {
                Ok(result_data) => {
                    for (set_id, coupons) in result_data.into_iter() {
                        merged_data
                            .entry(set_id)
                            .and_modify(|value| value.extend(coupons.clone().into_iter()))
                            .or_insert(coupons);
                    }
                }
                Err(err) => tracing::error!("{}", err),
            },
            Err(err) => tracing::error!("{}", err),
        }
    }

    if set_data != merged_data {
        tracing::error!("Fetched data does not equal to uploaded data");
    }

    tracing::info!("SUCCESS! Everything matches!");

    Ok(())
}

#[tracing::instrument(skip_all)]
async fn fetch_all(
    id: usize,
    config: TesterConfig,
    mut reference: Vec<i64>,
) -> anyhow::Result<SetData> {
    let mut result: SetData = reference
        .iter()
        .map(|set_id| (*set_id, BTreeSet::new()))
        .collect();

    let client = reqwest::Client::new();

    let mut pct = 0.1;
    let mut gotten = 0;
    let mut total_errors = 0;

    loop {
        if reference.is_empty() {
            tracing::info!(id, "Reference is empty. Stopping...");
            break;
        }

        // There should be only one error per set
        if total_errors > config.total_sets {
            tracing::error!(id, "TOO MANY ERRORS!");
            return Ok(result);
        }

        // Randomly select a coupon set to fetch from
        let idx = {
            let mut rng = rand::thread_rng();
            rng.gen_range(0..reference.len())
        };

        let selected_id = reference[idx];

        let coupon = match fetch_coupon(&client, selected_id).await? {
            FetchResult::Coupon(coupon) => coupon,
            FetchResult::StatusError(status) => {
                total_errors += 1;

                if status == StatusCode::NOT_FOUND {
                    // Set should be exhausted, remove it from reference
                    let set_id = reference.remove(idx);
                    tracing::info!(id, "Set exhausted: {}", set_id);
                    continue;
                }

                tracing::error!(id, "Status code error: {}", status);
                break;
            }
        };

        gotten += 1;
        let coupon_id = coupon.id.to_string();

        result
            .entry(selected_id)
            .and_modify(|value| {
                value.insert(coupon_id.clone());
            })
            .or_insert(BTreeSet::from_iter(vec![coupon_id].into_iter()));

        if (gotten as f64 / config.total_uploads as f64) >= pct {
            tracing::info!(id, "Approximately fetched {:.2}% of coupons", pct * 100.0);
            pct += 0.1;
        }

        // Random timeout to simulate "real world usage"
        if config.timeout > 0 {
            let millis = {
                let mut rng = rand::thread_rng();
                rng.gen_range(0..config.timeout)
            };
            tokio::time::sleep(Duration::from_millis(millis)).await;
        }
    }

    Ok(result)
}
