use std::str::FromStr;

use anyhow::Context;
use reqwest::Client;
use reqwest::Url;
use the_stack::model::coupon::Coupon;

pub async fn fetch_coupon(client: &Client, set_id: i64) -> anyhow::Result<Option<Coupon>> {
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
        return Ok(Some(coupon));
    } else {
        let body: serde_json::Value = response.json().await?;

        tracing::error!("Request Error [{}]: {}", status, body);
    }

    Ok(None)
}
