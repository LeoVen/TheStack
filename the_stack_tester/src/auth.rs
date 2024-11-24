use std::str::FromStr;

use anyhow::Context;
use chrono::Duration;
use chrono::Utc;
use reqwest::header::HeaderMap;
use reqwest::header::CONTENT_TYPE;
use reqwest::Url;
use serde::Deserialize;

use crate::TesterConfig;

#[derive(Deserialize)]
pub struct KeycloakResponse {
    pub access_token: String,
    pub expires_in: i64,
    pub refresh_expires_in: i64,
    pub refresh_token: String,
    // And other properties which I won't bother mapping
}

#[derive(Clone)]
struct Credentials {
    pub username: String,
    pub password: String,

    pub access_token: String,
    pub expires_at: chrono::DateTime<chrono::Utc>,
    pub refresh_token: String,
    pub refresh_expires_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Clone)]
pub struct CredentialsManager {
    kc: Credentials,
    kc_endpoint: Url,
}

impl CredentialsManager {
    pub async fn new(config: &TesterConfig) -> anyhow::Result<Self> {
        let kc_endpoint = Url::from_str(&config.kc_auth_endpoint)
            .expect("Could not parse kc_auth_endpoint to URL");

        let kc = Self::kc_login(&kc_endpoint, &config.username, &config.password).await?;

        Ok(Self { kc, kc_endpoint })
    }

    pub async fn kc_token(&mut self) -> anyhow::Result<String> {
        let now = Utc::now();

        if now < self.kc.expires_at {
            return Ok(self.kc.access_token.clone());
        }

        if now < self.kc.refresh_expires_at {
            self.kc_refresh();
            return Ok(self.kc.access_token.clone());
        }

        self.kc = Self::kc_login(&self.kc_endpoint, &self.kc.username, &self.kc.password).await?;

        Ok(self.kc.access_token.clone())
    }

    fn kc_refresh(&mut self) {
        let _ = self.kc.refresh_token;
        todo!() // TODO: I might implement this..
    }

    async fn kc_login(url: &Url, username: &str, password: &str) -> anyhow::Result<Credentials> {
        let client = reqwest::Client::new();

        let mut headers = HeaderMap::new();
        headers.insert(
            CONTENT_TYPE,
            "application/x-www-form-urlencoded".parse().unwrap(),
        );

        let result = client
            .post(url.clone())
            .headers(headers)
            .body(format!(
                "username={}&password={}&client_id=admin-cli&grant_type=password",
                username, password
            ))
            .send()
            .await?
            .error_for_status()
            .context("Failed to get initial credentials for Keycloak")?
            .json::<KeycloakResponse>()
            .await?;

        Ok(Credentials {
            username: username.to_string(),
            password: password.to_string(),
            access_token: result.access_token,
            expires_at: Utc::now() + Duration::seconds(result.expires_in),
            refresh_token: result.refresh_token,
            refresh_expires_at: Utc::now() + Duration::seconds(result.refresh_expires_in),
        })
    }
}
