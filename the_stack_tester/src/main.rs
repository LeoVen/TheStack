use std::str::FromStr;

use reqwest::Url;
use uuid::Uuid;

const UPLOAD_TOTAL: usize = 50000;
const SET_ID: i64 = 1;

struct IdGenerator;

impl Iterator for IdGenerator {
    type Item = Uuid;

    fn next(&mut self) -> Option<Self::Item> {
        Some(Uuid::new_v4())
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let client = reqwest::Client::new();

    let url = Url::from_str(&format!(
        "http://localhost:3000/coupon_set/{}/upload",
        SET_ID
    ))?;

    let coupons = IdGenerator
        .take(UPLOAD_TOTAL)
        .map(|id| id.to_string())
        .collect::<Vec<String>>();

    let result = client.post(url).json(&coupons).send().await;

    match result {
        Ok(resp) => println!("{:?}", resp),
        Err(error) => eprintln!("{}", error),
    }

    Ok(())
}
