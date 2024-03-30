use sqlx::{Pool, Postgres};

use crate::model::coupon::Coupon;

#[derive(Clone)]
pub struct CouponRepository {
    conn: Pool<Postgres>,
}

impl CouponRepository {
    pub fn new(conn: Pool<Postgres>) -> Self {
        Self { conn }
    }

    pub async fn get_by_set(&self, set_id: i64) -> Result<Vec<Coupon>, sqlx::Error> {
        sqlx::query_as("select * from coupon where set_id = $1")
            .bind(set_id)
            .fetch_all(&self.conn)
            .await
    }

    pub async fn pop_coupon(&self, set_id: i64) -> Result<Coupon, sqlx::Error> {
        sqlx::query_as(
        "with pop as (delete from coupon where id in (select id from coupon where set_id = $1 limit 1) returning *) select * from pop",
        )
        .bind(set_id)
        .fetch_one(&self.conn)
        .await
    }
}