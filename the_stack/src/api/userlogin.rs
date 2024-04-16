use std::sync::Arc;

use axum::extract::State;
use axum::Json;
use axum::Router;
use serde::Deserialize;
use serde::Serialize;

use crate::api::AppState;
use crate::database::userlogin::UserLoginRepository;
use crate::error::api::ApiResult;
use crate::service::userlogin::UserLoginService;

struct UserLoginState {
    service: UserLoginService,
}

pub fn router(ctx: AppState) -> Router {
    Router::<Arc<UserLoginState>>::new()
        .route("/userlogin/create", axum::routing::post(create_user))
        .route("/userlogin/login", axum::routing::post(login_user))
        .with_state(
            (UserLoginState {
                service: UserLoginService::new(UserLoginRepository::new(ctx.db)),
            })
            .into(),
        )
}

#[derive(Deserialize)]
struct CreateUserDto {
    pub email: String,
    pub password: String,
}

#[tracing::instrument(skip_all)]
async fn create_user(
    State(ctx): State<Arc<UserLoginState>>,
    Json(payload): Json<CreateUserDto>,
) -> ApiResult<()> {
    ctx.service
        .create_account(payload.email, payload.password)
        .await?;

    Ok(())
}

#[derive(Serialize)]
struct UserLoginResponse {
    pub ok: bool,
}

#[tracing::instrument(skip_all)]
async fn login_user(
    State(ctx): State<Arc<UserLoginState>>,
    Json(payload): Json<CreateUserDto>,
) -> ApiResult<Json<UserLoginResponse>> {
    let ok = ctx
        .service
        .validate_user(payload.email, payload.password)
        .await?;

    Ok(Json(UserLoginResponse { ok }))
}
