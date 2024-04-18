use std::collections::HashSet;
use std::str::FromStr;

use anyhow::Context;
use reqwest::Url;
use the_stack::model::coupon::Coupon;
use the_stack::model::coupon::CouponSet;
use tokio::task::JoinSet;

use crate::TOTAL_UPLOAD;

#[tracing::instrument(skip_all)]
pub async fn run_benchmark(sets: Vec<(CouponSet, Vec<String>)>) -> anyhow::Result<()> {
    let mut js = JoinSet::new();

    for set in sets.into_iter() {
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
async fn fetch_all(data: (CouponSet, Vec<String>)) -> anyhow::Result<()> {
    let set = data.0;
    let coupons = data.1;

    let mut coupons: HashSet<String> = HashSet::from_iter(coupons.into_iter());

    tracing::info!(
        "Fetching from {} (id: {}) a total of {} coupons",
        set.name,
        set.id,
        coupons.len()
    );

    let client = reqwest::Client::new();
    let url = Url::from_str(&format!(
        "http://localhost:3000/coupon_set/{}/coupon",
        set.id
    ))?;

    let mut pct = 0.1;

    for _ in 0..coupons.len() {
        let coupon = client
            .get(url.clone())
            .send()
            .await
            .context(format!("sending request for set id {}", set.id))?
            .json::<Coupon>()
            .await
            .context(format!("decoding request for set id {}", set.id))?;

        coupons.remove(&coupon.id.to_string());

        let rem = coupons.len();

        if (rem as f64 / TOTAL_UPLOAD as f64) <= (1.0 - pct) {
            tracing::info!(
                "[{}] {} processed {:.2}% ({} left)",
                set.id,
                set.name,
                pct * 100.0,
                rem
            );

            pct += 0.1;
        }
    }

    if !coupons.is_empty() {
        eprintln!(
            "Coupon set {} still has {} coupons that were not fetched",
            set.id,
            coupons.len()
        );
    }

    Ok(())
}
