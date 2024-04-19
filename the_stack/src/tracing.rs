use std::time::Duration;

use serde::Deserialize;
use tracing::level_filters::LevelFilter;
use tracing_subscriber::fmt;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use tracing_subscriber::Layer;

#[derive(Deserialize, Debug)]
pub struct TracingConfig {
    #[serde(rename(deserialize = "environment"))]
    pub env: String,
}

pub fn setup() -> String {
    let env_vars = envy::from_env::<TracingConfig>();
    let config = env_vars.unwrap_or(TracingConfig {
        env: "prod".to_string(),
    });

    if config.env == "dev" || config.env == "test" {
        tracing_subscriber::registry()
            .with(fmt::layer().with_filter(if config.env == "dev" {
                LevelFilter::DEBUG
            } else {
                LevelFilter::INFO
            }))
            .with(
                console_subscriber::ConsoleLayer::builder()
                    .retention(Duration::from_secs(60))
                    .spawn(),
            )
            .init();
    } else {
        tracing_subscriber::fmt().json().init();
    }

    tracing::info!("Tracing setup finished");

    config.env
}
