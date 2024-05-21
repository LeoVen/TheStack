use std::time::SystemTime;
use std::time::UNIX_EPOCH;

use anyhow::Context;
use jsonwebtoken::Algorithm;
use jsonwebtoken::DecodingKey;
use jsonwebtoken::EncodingKey;
use jsonwebtoken::Header;
use jsonwebtoken::TokenData;
use jsonwebtoken::Validation;
use serde::Deserialize;
use serde::Serialize;

const ALGORITHM: Algorithm = Algorithm::RS256;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct JWTConfigEnv {
    #[serde(rename(deserialize = "jwt_public_key"))]
    pub public_key: String,
    #[serde(rename(deserialize = "jwt_private_key"))]
    pub private_key: String,
    #[serde(rename(deserialize = "jwt_token_expiry_seconds"))]
    pub token_expiry: u64,
}

#[derive(Clone)]
pub struct JWTService {
    decoding: DecodingKey,
    encoding: EncodingKey,
    token_expiry: u64,
}

#[tracing::instrument]
pub fn setup() -> anyhow::Result<JWTService> {
    tracing::info!("Setting up JWT service");

    let config = envy::from_env::<JWTConfigEnv>().context("Failed to get env vars")?;

    let decoding = DecodingKey::from_rsa_pem(config.public_key.as_bytes())
        .with_context(|| format!("Key: {}", config.public_key))?;
    let encoding = EncodingKey::from_rsa_pem(config.private_key.as_bytes())
        .with_context(|| format!("Key: {}", config.private_key))?;

    Ok(JWTService {
        encoding,
        decoding,
        token_expiry: config.token_expiry,
    })
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,
    pub exp: u64,
}

impl JWTService {
    pub fn create_token(&self, user: &str) -> anyhow::Result<String> {
        let claims = Claims {
            sub: user.to_string(),
            exp: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .context("Failed to generate token claims")?
                .as_secs()
                + self.token_expiry,
        };

        let token = jsonwebtoken::encode(&Header::new(ALGORITHM), &claims, &self.encoding)
            .context("Failed to generate token")?;

        Ok(token)
    }

    pub fn decode_token(&self, token: &str) -> anyhow::Result<TokenData<Claims>> {
        let mut validation = Validation::new(ALGORITHM);
        validation.set_required_spec_claims(&["sub", "exp"]);
        let result = jsonwebtoken::decode::<Claims>(token, &self.decoding, &validation)?;

        Ok(result)
    }
}
