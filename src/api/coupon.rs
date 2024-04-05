use std::sync::Arc;

use axum::extract::Path;
use axum::extract::State;
use axum::Json;
use axum::Router;

use crate::api::AppState;
use crate::cache::coupon::CouponCache;
use crate::database::coupon::CouponRepository;
use crate::error::api::ApiResult;
use crate::model::coupon::Coupon;
use crate::service::coupon::CouponService;

struct CouponAppState {
    coupons: CouponService,
}

pub fn router(ctx: AppState) -> Router {
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
                coupons: CouponService::new(
                    CouponRepository::new(ctx.db),
                    CouponCache::new(ctx.cache),
                ),
            })
            .into(),
        )
}

#[tracing::instrument(skip_all)]
async fn get_by_coupon_set_id(
    State(ctx): State<Arc<CouponAppState>>,
    Path(set_id): Path<i64>,
) -> ApiResult<Json<Vec<Coupon>>> {
    let values = ctx.coupons.get_by_set(set_id).await?;
    Ok(Json(values))
}

#[tracing::instrument(skip_all)]
async fn pop_coupon(
    State(ctx): State<Arc<CouponAppState>>,
    Path(set_id): Path<i64>,
) -> ApiResult<Json<Coupon>> {
    let value = ctx.coupons.pop_coupon(set_id).await?;
    Ok(Json(value))
}
