use serde::Deserialize;

use crate::database::userlogin::UserLoginRepository;
use crate::error::service::ServiceResult;
use crate::hash::hash_password;
use crate::hash::verify_password;
use crate::model::userlogin::UserLogin;

#[derive(Clone, Deserialize)]
struct AuthConfig {
    #[serde(rename(deserialize = "auth_cookie_key"))]
    pub auth_cookie: String,
}

#[derive(Clone)]
pub struct UserLoginService {
    config: AuthConfig,
    repo: UserLoginRepository,
}

impl UserLoginService {
    pub fn new(repo: UserLoginRepository) -> anyhow::Result<Self> {
        let config = envy::from_env::<AuthConfig>()?;
        Ok(Self { config, repo })
    }

    pub fn auth_cookie(&self) -> String {
        self.config.auth_cookie.clone()
    }

    pub async fn create_account(&self, email: String, password: String) -> ServiceResult<()> {
        let password = hash_password(&password)?;

        tracing::info!("creating user {}", email);

        let _ = self
            .repo
            .create(UserLogin {
                id: 0, // doesn't matter which value
                email,
                password,
            })
            .await?;

        Ok(())
    }

    pub async fn validate_user(&self, email: &str, password: &str) -> ServiceResult<()> {
        let dbuser = self.repo.get_by_email(email).await?;

        verify_password(&dbuser.password, password)?;

        Ok(())
    }
}
