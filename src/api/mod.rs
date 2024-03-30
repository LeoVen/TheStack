pub mod coupon;
pub mod metrics;

use std::net::{Ipv4Addr, SocketAddrV4};

use anyhow::{Context, Result};
use axum::{body::Body, extract::MatchedPath, http::Request, response::Response, Router};
use redis::aio::MultiplexedConnection;
use serde::{Deserialize, Serialize};
use sqlx::{Pool, Postgres};
use tower_http::trace::TraceLayer;

use crate::metrics::Metrics;

#[derive(Serialize, Deserialize, Debug)]
struct AxumApiConfig {
    #[serde(rename(deserialize = "api_axum_port"))]
    pub port: u16,
}

#[derive(Clone)]
pub struct AppState {
    pub db: Pool<Postgres>,
    pub cache: MultiplexedConnection,
    pub metrics: Metrics,
}

#[tracing::instrument(skip(state))]
pub async fn setup(env: &str, state: AppState) -> Result<()> {
    let config = envy::from_env::<AxumApiConfig>().context("Failed to get env vars")?;

    if env == "dev" {
        let config_str = serde_json::to_string(&config).unwrap_or("Serialize Error".to_string());
        tracing::info!(config = config_str);
    }

    let metrics = state.metrics.clone();
    let trace_layer = TraceLayer::new_for_http()
        .make_span_with(|request: &Request<Body>| {
            let matched_path = request
                .extensions()
                .get::<MatchedPath>()
                .map(MatchedPath::as_str);

            tracing::info_span!(
                "http_request",
                method = ?request.method(),
                matched_path,
            )
        })
        .on_request(move |_request: &Request<Body>, _span: &tracing::Span| {
            metrics.api_count.inc();
        })
        .on_response(
            move |response: &Response<Body>, _duration, _span: &tracing::Span| {
                let status = response.status();

                if status.is_success() {
                    metrics.api_2xx.inc();
                } else if status.is_client_error() {
                    metrics.api_4xx.inc();
                } else if status.is_server_error() {
                    metrics.api_5xx.inc();
                }
            },
        );

    let coupons = coupon::router(state.clone()).layer(trace_layer.clone());
    let metrics = metrics::router();

    let app = Router::new().merge(metrics).merge(coupons);

    let listener =
        tokio::net::TcpListener::bind(SocketAddrV4::new(Ipv4Addr::new(0, 0, 0, 0), config.port))
            .await
            .context("Failed to bind to TCP port")?;

    tracing::info!("Listening on port {}", config.port);

    axum::serve(listener, app)
        .await
        .context("Axum serve failed")?;

    Ok(())
}
