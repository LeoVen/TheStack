pub mod coupon;
pub mod dto;
pub mod files;
pub mod metrics;
pub mod userlogin;
pub mod worker;

use std::net::Ipv4Addr;
use std::net::SocketAddrV4;
use std::sync::Arc;
use std::sync::Mutex;
use std::time::Duration;

use anyhow::Context;
use anyhow::Result;
use axum::body::Body;
use axum::extract::MatchedPath;
use axum::http::Request;
use axum::response::Response;
use axum::Router;
use redis::aio::MultiplexedConnection;
use serde::Deserialize;
use serde::Serialize;
use sqlx::Pool;
use sqlx::Postgres;
use tower_cookies::CookieManagerLayer;
use tower_http::trace::TraceLayer;

use crate::jwt::JWTService;
use crate::metrics::Metrics;

pub static AUTH_COOKIE: &'static str = "auth-token";

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
    pub timeout: Arc<Mutex<u64>>,
    pub jwt_service: JWTService,
}

#[tracing::instrument(skip(ctx))]
pub async fn setup(env: &str, ctx: AppState) -> Result<()> {
    let config = envy::from_env::<AxumApiConfig>().context("Failed to get env vars")?;

    if env == "dev" {
        let config_str = serde_json::to_string(&config).unwrap_or("Serialize Error".to_string());
        tracing::info!(config = config_str);
    }

    let cookies = CookieManagerLayer::new();
    let metrics = ctx.metrics.clone();
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
            move |response: &Response<Body>, duration: Duration, _span: &tracing::Span| {
                let status = response.status();

                metrics.req_elapsed.observe(duration.as_secs_f64());

                if status.is_success() {
                    metrics.api_2xx.inc();
                } else if status.is_client_error() {
                    metrics.api_4xx.inc();
                } else if status.is_server_error() {
                    metrics.api_5xx.inc();
                }
            },
        );

    let coupons = coupon::router(ctx.clone()).layer(trace_layer.clone());
    let files = files::router();
    let metrics = metrics::router();
    let userlogin = userlogin::router(ctx.clone())
        .layer(trace_layer.clone())
        .layer(cookies.clone());
    let workers = worker::router(ctx.clone()).layer(trace_layer.clone());

    let app = Router::new()
        .merge(metrics)
        .merge(coupons)
        .merge(workers)
        .merge(userlogin)
        .fallback_service(files);

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
