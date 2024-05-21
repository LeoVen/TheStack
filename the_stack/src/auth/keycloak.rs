use std::str::FromStr;

use anyhow::anyhow;
use axum::extract::Request;
use axum::extract::State;
use axum::http::HeaderMap;
use axum::middleware::Next;
use axum::response::Response;
use chrono::DateTime;
use chrono::Utc;
use jsonwebtoken::jwk::JwkSet;
use jsonwebtoken::Algorithm;
use jsonwebtoken::DecodingKey;
use jsonwebtoken::Validation;
use reqwest::Url;
use serde::Deserialize;

use super::jwt::Claims;
use crate::error::api::ApiError;
use crate::error::api::ApiResult;

#[derive(Deserialize)]
struct AuthConfigFromEnv {
    #[serde(rename(deserialize = "auth_keycloak_jwks_endpoint"))]
    pub jwks_endpoint: String,
}

#[derive(Clone)]
struct AuthConfig {
    jwks_endpoint: Url,
}

#[derive(Clone)]
pub struct KeycloakAuthMiddleware {
    config: AuthConfig,
    client: reqwest::Client,
}

impl KeycloakAuthMiddleware {
    pub fn new() -> anyhow::Result<Self> {
        let config = envy::from_env::<AuthConfigFromEnv>()?;

        let jwks_endpoint = Url::from_str(&config.jwks_endpoint)?;

        Ok(Self {
            config: AuthConfig { jwks_endpoint },
            client: reqwest::Client::new(),
        })
    }

    pub async fn authenticate(
        State(state): State<KeycloakAuthMiddleware>,
        headers: HeaderMap,
        req: Request,
        next: Next,
    ) -> ApiResult<Response> {
        let Some(bearer_auth) = headers.get(axum::http::header::AUTHORIZATION) else {
            return Err(ApiError::default_unauthorized());
        };
        let bearer_auth = bearer_auth.to_str()?.to_string();
        let token = bearer_auth
            .strip_prefix("Bearer ")
            .ok_or(ApiError::default_unauthorized())?;

        // TODO cache this
        let jwks = state
            .client
            .get(state.config.jwks_endpoint.to_owned())
            .send()
            .await?
            .error_for_status()?
            .json::<JwkSet>()
            .await?;

        let headers = jsonwebtoken::decode_header(token)?;

        let key_id = headers
            .kid
            .ok_or(ApiError::Internal(anyhow!("invalid kid header")))?;

        let public_key = jwks.find(&key_id).ok_or(ApiError::Internal(anyhow!(
            "no matching public key in jwks"
        )))?;

        let decoding_key = DecodingKey::from_jwk(public_key)?;

        // TODO use different type for Claims
        let token_data = jsonwebtoken::decode::<Claims>(
            token,
            &decoding_key,
            &Validation::new(Algorithm::RS256),
        )
        .map_err(|e| ApiError::Unauthorized {
            message: "invalid jwt".to_string(),
            error: Some(e.into()),
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
