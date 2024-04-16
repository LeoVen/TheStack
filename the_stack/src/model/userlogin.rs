use serde::Deserialize;
use serde::Serialize;
use sqlx::FromRow;

#[derive(Serialize, Deserialize, FromRow, Clone)]
pub struct UserLogin {
    pub id: i64,
    pub email: String,
    pub password: String,
}
