use sqlx::postgres::PgQueryResult;
use sqlx::Pool;
use sqlx::Postgres;
use uuid::Uuid;

use crate::error::database::DatabaseResult;
use crate::model::coupon::Coupon;
use crate::model::coupon::CouponSet;
use crate::model::coupon::CouponSetStatus;
use crate::model::coupon::CreateCouponSetDto;

static POP_COUPONS_QUERY: &str = r"
WITH upd AS
    (UPDATE coupon SET used = true WHERE id IN
            (SELECT id FROM coupon WHERE set_id = $1 AND used = false ORDER BY id LIMIT $2)
    RETURNING *)
SELECT * FROM upd
";

#[derive(Clone)]
pub struct CouponRepository {
    conn: Pool<Postgres>,
}

impl CouponRepository {
    pub fn new(conn: Pool<Postgres>) -> Self {
        Self { conn }
    }

    pub async fn batch_insert(&self, coupons: Vec<Coupon>) -> DatabaseResult<u64> {
        let (ids, set_ids) =
            coupons
                .into_iter()
                .fold((vec![], vec![]), |(mut ids, mut set_ids), item| {
                    ids.push(item.id);
                    set_ids.push(item.set_id);
                    (ids, set_ids)
                });

        let result: PgQueryResult = sqlx::query(
            "insert into coupon(id, set_id) select * from unnest($1::uuid[], $2::int8[])",
        )
        .bind(&ids)
        .bind(&set_ids)
        .execute(&self.conn)
        .await?;

        Ok(result.rows_affected())
    }

    pub async fn pop_coupons(&self, set_id: i64, limit: i64) -> DatabaseResult<Vec<Coupon>> {
        let result = sqlx::query_as(POP_COUPONS_QUERY)
            .bind(set_id)
            .bind(limit)
            .fetch_all(&self.conn)
            .await?;

        Ok(result)
    }

    pub async fn delete_coupons(&self, coupons: &[Uuid]) -> DatabaseResult<u64> {
        if coupons.is_empty() {
            return Ok(0);
        }

        let result =
            sqlx::query("delete from coupon where id in (select * from unnest($1::uuid[]))")
                .bind(coupons)
                .execute(&self.conn)
                .await?;

        Ok(result.rows_affected())
    }

    pub async fn create_set(&self, create_dto: CreateCouponSetDto) -> DatabaseResult<CouponSet> {
        let result = sqlx::query_as(
            "with add as (insert into coupon_set (name) values ($1) returning *) select * from add",
        )
        .bind(create_dto.name)
        .fetch_one(&self.conn)
        .await?;

        Ok(result)
    }

    pub async fn set_status(&self) -> DatabaseResult<Vec<CouponSetStatus>> {
        let result = sqlx::query_as(
            "select *, (select count(*) from coupon c where c.set_id = s.id) as total_coupons from coupon_set s",
        )
        .fetch_all(&self.conn)
        .await?;

        Ok(result)
    }
}
