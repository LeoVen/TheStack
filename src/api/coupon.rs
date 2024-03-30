use std::sync::Arc;

use axum::{
    extract::{Path, State},
    Json, Router,
};

use crate::{
    api::AppState, cache::coupon::CouponCache, database::coupon::CouponRepository,
    error::ApiResult, metrics::Metrics, model::coupon::Coupon,
};

struct CouponAppState {
    cache: CouponCache,
    repo: CouponRepository,
    metrics: Metrics,
}

pub fn router(app_state: AppState) -> Router {
    Router::<Arc<CouponAppState>>::new()
        .route(
            "/coupon_set/:set_id/coupons",
            axum::routing::get(get_by_coupon_set_id),
        )
        .route(
            "/coupon_set/:set_id/coupons/fetch",
            axum::routing::get(pop_coupon),
        )
        .with_state(
            (CouponAppState {
                cache: CouponCache::new(app_state.cache),
                metrics: app_state.metrics,
                repo: CouponRepository::new(app_state.db),
            })
            .into(),
        )
}

#[tracing::instrument(skip_all)]
async fn get_by_coupon_set_id(
    State(ctx): State<Arc<CouponAppState>>,
    Path(set_id): Path<i64>,
) -> ApiResult<Json<Vec<Coupon>>> {
    let values = ctx.repo.get_by_set(set_id).await?;

    Ok(Json(values))
}

#[tracing::instrument(skip_all)]
async fn pop_coupon(
    State(ctx): State<Arc<CouponAppState>>,
    Path(set_id): Path<i64>,
) -> ApiResult<Json<Coupon>> {
    let value = ctx.repo.pop_coupon(set_id).await?;
    Ok(Json(value))
}
