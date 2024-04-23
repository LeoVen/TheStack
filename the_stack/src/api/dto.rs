use serde::Deserialize;
use serde::Serialize;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateCouponSetDto {
    pub name: String,
}

#[derive(Serialize)]
pub struct CouponStatusResponseDto {
    pub id: i64,
    pub name: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub total_database: i64,
    pub total_cache: i64,
}
