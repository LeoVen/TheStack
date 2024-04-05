use crate::cache::coupon::CouponCache;
use crate::database::coupon::CouponRepository;
use crate::error::service::ServiceResult;
use crate::model::coupon::Coupon;

pub struct CouponService {
    repo: CouponRepository,
    cache: CouponCache,
}

impl CouponService {
    pub fn new(repo: CouponRepository, cache: CouponCache) -> Self {
        Self { repo, cache }
    }

    pub async fn get_by_set(&self, set_id: i64) -> ServiceResult<Vec<Coupon>> {
        // TODO add cache
        let _ = self.cache;
        let result = self.repo.get_by_set(set_id).await?;
        Ok(result)
    }

    pub async fn pop_coupon(&self, set_id: i64) -> ServiceResult<Coupon> {
        // TODO add cache
        let _ = self.cache;
        let result = self.repo.pop_coupon(set_id).await?;
        Ok(result)
    }
}
