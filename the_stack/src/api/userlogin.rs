use std::sync::Arc;

use axum::extract::State;
use axum::Json;
use axum::Router;
use serde::Deserialize;
use serde::Serialize;
use tower_cookies::Cookie;
use tower_cookies::Cookies;

use crate::api::AppState;
use crate::database::userlogin::UserLoginRepository;
use crate::error::api::ApiResult;
use crate::jwt::JWTService;
use crate::service::userlogin::UserLoginService;

use super::AUTH_COOKIE;

struct UserLoginState {
    service: UserLoginService,
    jwt_service: JWTService,
}

pub fn router(ctx: AppState) -> Router {
    Router::<Arc<UserLoginState>>::new()
        .route("/userlogin/create", axum::routing::post(create_user))
        .route("/userlogin/login", axum::routing::post(login_user))
        .with_state(
            (UserLoginState {
                service: UserLoginService::new(UserLoginRepository::new(ctx.db)),
                jwt_service: ctx.jwt_service.clone(),
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

#[derive(Deserialize)]
struct UserLoginDto {
    pub email: String,
    pub password: String,
}

#[derive(Serialize)]
struct UserLoginResponseDto {
    pub ok: bool,
}

#[tracing::instrument(skip_all)]
async fn login_user(
    cookies: Cookies,
    State(ctx): State<Arc<UserLoginState>>,
    Json(payload): Json<UserLoginDto>,
) -> ApiResult<Json<UserLoginResponseDto>> {
    ctx.service
        .validate_user(&payload.email, &payload.password)
        .await?;

    let auth_token = ctx.jwt_service.create_token(&payload.email)?;

    cookies.add(Cookie::new(AUTH_COOKIE, auth_token));

    Ok(Json(UserLoginResponseDto { ok: true }))
}
