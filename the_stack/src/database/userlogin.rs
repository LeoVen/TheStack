use sqlx::Pool;
use sqlx::Postgres;

use crate::error::database::DatabaseResult;
use crate::model::userlogin::UserLogin;

#[derive(Clone)]
pub struct UserLoginRepository {
    conn: Pool<Postgres>,
}

impl UserLoginRepository {
    pub fn new(conn: Pool<Postgres>) -> Self {
        Self { conn }
    }

    pub async fn create(&self, user: UserLogin) -> DatabaseResult<u64> {
        let result = sqlx::query("insert into userlogin (email, password) values ($1, $2)")
            .bind(user.email)
            .bind(user.password)
            .execute(&self.conn)
            .await?;

        Ok(result.rows_affected())
    }

    pub async fn get_by_email(&self, email: &str) -> DatabaseResult<UserLogin> {
        let result = sqlx::query_as("select * from userlogin where email = $1")
            .bind(email)
            .fetch_one(&self.conn)
            .await?;

        Ok(result)
    }
}
