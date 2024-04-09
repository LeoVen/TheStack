use uuid::Uuid;

use crate::cache::coupon::CouponCache;
use crate::database::coupon::CouponRepository;
use crate::error::service::ServiceError;
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

    #[tracing::instrument(skip(self))]
    pub async fn pop_coupon(&self, set_id: i64) -> ServiceResult<Coupon> {
        let mut cache = self.cache.clone();

        let cached = cache.pop_coupon(set_id).await?;

        if let Some(cached) = cached {
            return Ok(Coupon {
                id: Uuid::try_parse(&cached)?,
                set_id,
                used: true,
            });
        }

        // TODO this can be improved

        let coupons = self.repo.pop_coupons(set_id, 100).await?;

        cache.batch_insert(set_id, &coupons).await?;

        let cached = cache.pop_coupon(set_id).await?;

        if let Some(cached) = cached {
            return Ok(Coupon {
                id: Uuid::try_parse(&cached)?,
                set_id,
                used: true,
            });
        }

        Err(ServiceError::NotFound)
    }
}
