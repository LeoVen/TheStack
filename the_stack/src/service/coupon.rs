use tokio_retry::strategy::jitter;
use tokio_retry::strategy::ExponentialBackoff;
use tokio_retry::Retry;
use uuid::Uuid;

use crate::api::dto::CouponStatusResponseDto;
use crate::api::dto::CreateCouponSetDto;
use crate::cache::coupon::CouponCache;
use crate::cache::lock::DistributedLock;
use crate::database::coupon::CouponRepository;
use crate::error::service::ServiceError;
use crate::error::service::ServiceResult;
use crate::metrics::Metrics;
use crate::model::coupon::Coupon;
use crate::model::coupon::CouponSet;
use crate::service::BatchInsertConfig;

pub struct CouponService {
    repo: CouponRepository,
    cache: CouponCache,
    metrics: Metrics,
    lock: DistributedLock,
    batch_config: BatchInsertConfig,
}

impl CouponService {
    pub fn new(
        repo: CouponRepository,
        cache: CouponCache,
        metrics: Metrics,
        lock: DistributedLock,
        batch_config: BatchInsertConfig,
    ) -> Self {
        Self {
            repo,
            cache,
            metrics,
            lock,
            batch_config,
        }
    }

    #[tracing::instrument(skip(self, payload))]
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

        if let Some(lock) = self
            .lock
            .lock(&format!("{}{}", self.batch_config.lock_prefix, set_id))
            .await
        {
            let unlock = || async {
                self.lock.unlock(lock).await;
            };

            let mut coupons = self
                .repo
                .pop_coupons(set_id, self.batch_config.insert_total)
                .await?;

            if coupons.is_empty() {
                unlock().await;
                return self.pop_from_cache(set_id).await;
            }

            let coupon = coupons.pop().ok_or(ServiceError::NotFound)?;

            cache.batch_insert(set_id, &coupons).await?;

            unlock().await;

            return Ok(Coupon {
                id: coupon.id,
                set_id: coupon.set_id,
            });
        }

        // Retry since some other thread might be inserting coupons at the same time
        let coupon = Retry::spawn(
            ExponentialBackoff::from_millis(100).map(jitter).take(3),
            || async { self.pop_from_cache(set_id).await },
        )
        .await?;

        Ok(coupon)
    }

    async fn pop_from_cache(&self, set_id: i64) -> ServiceResult<Coupon> {
        let mut cache = self.cache.clone();

        let cached = cache.pop_coupon(set_id).await?;

        match cached {
            Some(cached) => Ok(Coupon {
                id: Uuid::try_parse(&cached)?,
                set_id,
            }),
            None => Err(ServiceError::NotFound),
        }
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
    pub async fn set_status(&self) -> ServiceResult<Vec<CouponStatusResponseDto>> {
        let mut cache = self.cache.clone();

        let cache = cache.set_status().await?;
        let database = self.repo.set_status().await?;

        let mut result = vec![];

        for status in database.into_iter() {
            let id = status.id;

            let in_cache = cache
                .iter()
                .find(|c| c.id == id)
                .unwrap_or(&Default::default())
                .total_coupons;

            result.push(CouponStatusResponseDto {
                id,
                name: status.name,
                created_at: status.created_at,
                total_cache: in_cache,
                total_database: status.total_coupons,
            });
        }

        Ok(result)
    }
}
