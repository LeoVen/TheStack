use anyhow::anyhow;
use axum::extract::Request;
use axum::extract::State;
use axum::middleware::Next;
use axum::response::Response;
use chrono::DateTime;
use chrono::Utc;
use tower_cookies::Cookies;

use super::jwt::JWTService;
use crate::error::api::ApiError;
use crate::error::api::ApiResult;
use crate::service::userlogin::UserLoginService;

#[derive(Clone)]
pub struct UserAuthMiddleware {
    jwt_service: JWTService,
    user_login: UserLoginService,
}

impl UserAuthMiddleware {
    pub fn new(jwt_service: JWTService, user_login: UserLoginService) -> Self {
        Self {
            jwt_service,
            user_login,
        }
    }

    pub async fn auth_middleware(
        State(state): State<UserAuthMiddleware>,
        cookies: Cookies,
        req: Request,
        next: Next,
    ) -> ApiResult<Response> {
        let Some(auth_cookie) = cookies.get(&state.user_login.auth_cookie()) else {
            return Err(ApiError::Unauthorized {
                message: "invalid cookie".to_string(),
                error: None,
            });
        };

        let token_data = state
            .jwt_service
            .decode_token(auth_cookie.value())
            .map_err(|e| ApiError::Unauthorized {
                message: "invalid jwt".to_string(),
                error: Some(e),
            })?;

        let exp = DateTime::from_timestamp(token_data.claims.exp.try_into()?, 0).ok_or(
            ApiError::Internal(anyhow!("exp claim conversion to timestamp error")),
        )?;
        if Utc::now() > exp {
            return Err(ApiError::Unauthorized {
                message: "token expired".to_string(),
                error: None,
            });
        }

        Ok(next.run(req).await)
    }
}
