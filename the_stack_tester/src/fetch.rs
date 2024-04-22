use std::str::FromStr;

use anyhow::Context;
use reqwest::Client;
use reqwest::StatusCode;
use reqwest::Url;
use the_stack::model::coupon::Coupon;

pub enum FetchResult {
    Coupon(Coupon),
    StatusError(StatusCode),
}

pub async fn fetch_coupon(client: &Client, set_id: i64) -> anyhow::Result<FetchResult> {
    let url = Url::from_str(&format!(
        "http://localhost:3000/coupon_set/{}/coupon",
        set_id
    ))?;

    let response = client
        .get(url.clone())
        .send()
        .await
        .context(format!("sending request for set id {}", set_id))?;

    let status = response.status();

    if status.is_success() {
        let coupon = response
            .json::<Coupon>()
            .await
            .context(format!("decoding request for set id {}", set_id))?;
        return Ok(FetchResult::Coupon(coupon));
    }

    let body: serde_json::Value = response.json().await?;
    tracing::warn!("Request Error [{}]: {}", status, body);

    Ok(FetchResult::StatusError(status))
}
