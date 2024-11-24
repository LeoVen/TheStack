use std::collections::HashSet;

use the_stack::model::coupon::CouponSet;
use tokio::task::JoinSet;

use crate::auth::CredentialsManager;
use crate::fetch::fetch_coupon;
use crate::fetch::FetchResult;
use crate::TesterConfig;

#[tracing::instrument(skip_all)]
pub async fn run_benchmark(
    config: TesterConfig,
    sets: Vec<(CouponSet, Vec<String>)>,
    cred_manager: CredentialsManager,
) -> anyhow::Result<()> {
    let mut js = JoinSet::new();

    for set in sets.into_iter() {
        let config = config.clone();
        let cred_manager = cred_manager.clone();
        js.spawn(async { fetch_all(config, set, cred_manager).await });
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
async fn fetch_all(
    config: TesterConfig,
    data: (CouponSet, Vec<String>),
    mut cred_manager: CredentialsManager,
) -> anyhow::Result<()> {
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

    let mut pct = 0.1;

    for _ in 0..coupons.len() {
        let FetchResult::Coupon(coupon) =
            fetch_coupon(&client, set.id, &cred_manager.kc_token().await?).await?
        else {
            continue;
        };

        coupons.remove(&coupon.id.to_string());

        let rem = coupons.len();

        if (rem as f64 / config.total_uploads as f64) <= (1.0 - pct) {
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
