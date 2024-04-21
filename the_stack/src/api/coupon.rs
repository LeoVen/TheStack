use std::sync::Arc;

use axum::extract::Path;
use axum::extract::State;
use axum::Json;
use axum::Router;

use crate::api::AppState;
use crate::cache::coupon::CouponCache;
use crate::database::coupon::CouponRepository;
use crate::error::api::ApiError;
use crate::error::api::ApiResult;
use crate::model::coupon::Coupon;
use crate::model::coupon::CouponSet;
use crate::model::coupon::CouponSetStatus;
use crate::model::coupon::CreateCouponSetDto;
use crate::service::coupon::CouponService;

struct CouponAppState {
    service: CouponService,
}

pub fn router(ctx: AppState) -> Router {
    Router::<Arc<CouponAppState>>::new()
        .route("/coupon_set/:set_id/coupon", axum::routing::get(pop_coupon))
        .route(
            "/coupon_set/:set_id/upload",
            axum::routing::post(upload_coupons),
        )
        .route("/coupon_set", axum::routing::post(create_set))
        .route("/coupon_set/status", axum::routing::get(set_status))
        .with_state(
            (CouponAppState {
                service: CouponService::new(
                    CouponRepository::new(ctx.db),
                    CouponCache::new(ctx.cache),
                    ctx.metrics.clone(),
                ),
            })
            .into(),
        )
}

#[tracing::instrument(skip_all)]
async fn pop_coupon(
    State(ctx): State<Arc<CouponAppState>>,
    Path(set_id): Path<i64>,
) -> ApiResult<Json<Coupon>> {
    let value = ctx.service.pop_coupon(set_id).await?;
    Ok(Json(value))
}

#[tracing::instrument(skip_all)]
async fn upload_coupons(
    State(ctx): State<Arc<CouponAppState>>,
    Path(set_id): Path<i64>,
    Json(coupons): Json<Vec<String>>,
) -> ApiResult<()> {
    if coupons.is_empty() {
        return Err(ApiError::BadRequest("Empty coupon list".to_string()));
    }

    ctx.service.spawn_upload_job(set_id, coupons).await;

    Ok(())
}

#[tracing::instrument(skip_all)]
async fn create_set(
    State(ctx): State<Arc<CouponAppState>>,
    Json(create_dto): Json<CreateCouponSetDto>,
) -> ApiResult<Json<CouponSet>> {
    let result = ctx.service.create_coupon_set(create_dto).await?;
    Ok(Json(result))
}

#[tracing::instrument(skip_all)]
async fn set_status(
    State(ctx): State<Arc<CouponAppState>>,
) -> ApiResult<Json<Vec<CouponSetStatus>>> {
    let result = ctx.service.set_status().await?;
    Ok(Json(result))
}
