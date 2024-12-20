use std::str::FromStr;

use anyhow::Context;
use reqwest::Client;
use reqwest::Url;
use the_stack::api::dto::CreateCouponSetDto;
use the_stack::model::coupon::CouponSet;
use uuid::Uuid;

struct IdGenerator;

impl Iterator for IdGenerator {
    type Item = Uuid;

    fn next(&mut self) -> Option<Self::Item> {
        Some(Uuid::new_v4())
    }
}

pub async fn create_set(client: &Client, token: &str, name: String) -> anyhow::Result<CouponSet> {
    let url = Url::from_str("http://localhost:3000/coupon_set")?;

    let payload = CreateCouponSetDto { name: name.clone() };

    let result = client
        .post(url)
        .bearer_auth(token)
        .json(&payload)
        .send()
        .await?
        .error_for_status()
        .with_context(|| format!("Failed to create Coupon Set {}", &name))?
        .json::<CouponSet>()
        .await?;

    Ok(result)
}

pub async fn upload_coupons(
    client: &Client,
    token: &str,
    set_id: i64,
    total_coupons: usize,
) -> anyhow::Result<Vec<String>> {
    let url = Url::from_str(&format!(
        "http://localhost:3000/coupon_set/{}/upload",
        set_id
    ))?;

    let coupons = IdGenerator
        .take(total_coupons)
        .map(|id| id.to_string())
        .collect::<Vec<String>>();

    client
        .post(url)
        .bearer_auth(token)
        .json(&coupons)
        .send()
        .await
        .context("failed to upload coupons")?;

    Ok(coupons)
}
