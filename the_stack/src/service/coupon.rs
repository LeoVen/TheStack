use uuid::Uuid;

use crate::cache::coupon::CouponCache;
use crate::database::coupon::CouponRepository;
use crate::error::service::ServiceError;
use crate::error::service::ServiceResult;
use crate::metrics::Metrics;
use crate::model::coupon::Coupon;
use crate::model::coupon::CouponSet;
use crate::model::coupon::CouponSetStatus;
use crate::model::coupon::CreateCouponSetDto;

pub struct CouponService {
    repo: CouponRepository,
    cache: CouponCache,
    metrics: Metrics,
}

impl CouponService {
    pub fn new(repo: CouponRepository, cache: CouponCache, metrics: Metrics) -> Self {
        Self {
            repo,
            cache,
            metrics,
        }
    }

    #[tracing::instrument(skip(self))]
    pub async fn spawn_upload_job(&self, set_id: i64, payload: Vec<String>) {
        // TODO time how long this takes, for fun

        let repo = self.repo.clone();
        let metrics = self.metrics.clone();

        tokio::task::spawn(async move {
            let original = payload.len();
            let mut coupons = Vec::with_capacity(payload.len());

            for coupon in payload.into_iter() {
                if let Ok(id) = Uuid::parse_str(&coupon) {
                    coupons.push(Coupon { id, set_id })
                }
            }

            let mapped = coupons.len();

            if original != mapped {
                tracing::warn!(
                    set_id,
                    diff = original - mapped,
                    "could not map all coupons"
                );
            }

            match repo.batch_insert(coupons).await {
                Ok(rows_affected) => {
                    metrics.batch_inserts.inc();

                    tracing::info!(rows_affected, set_id, "added coupons");
                }
                Err(err) => {
                    let err_str = err.to_string();

                    tracing::error!(set_id, error = err_str, "failed to add coupons");
                }
            }
        });
    }

    #[tracing::instrument(skip(self))]
    pub async fn pop_coupon(&self, set_id: i64) -> ServiceResult<Coupon> {
        let mut cache = self.cache.clone();

        let cached = cache.pop_coupon(set_id).await?;

        if let Some(cached) = cached {
            self.metrics.cache_hit.inc();

            return Ok(Coupon {
                id: Uuid::try_parse(&cached)?,
                set_id,
            });
        }

        self.metrics.cache_miss.inc();

        // TODO this can be improved

        let coupons = self.repo.pop_coupons(set_id, 1000).await?;

        cache.batch_insert(set_id, &coupons).await?;

        let cached = cache.pop_coupon(set_id).await?;

        if let Some(cached) = cached {
            return Ok(Coupon {
                id: Uuid::try_parse(&cached)?,
                set_id,
            });
        }

        Err(ServiceError::NotFound)
    }

    #[tracing::instrument(skip(self))]
    pub async fn create_coupon_set(
        &self,
        create_dto: CreateCouponSetDto,
    ) -> ServiceResult<CouponSet> {
        let result = self.repo.create_set(create_dto).await?;
        Ok(result)
    }

    #[tracing::instrument(skip_all)]
    pub async fn set_status(&self) -> ServiceResult<Vec<CouponSetStatus>> {
        let result = self.repo.set_status().await?;
        Ok(result)
    }
}
