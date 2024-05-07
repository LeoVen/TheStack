use rslock::Lock;
use rslock::LockManager;

use super::RedisConfig;

#[derive(Clone)]
pub struct DistributedLock {
    lock_manager: LockManager,
}

impl DistributedLock {
    pub(super) fn new(config: &RedisConfig) -> Self {
        // URL format: `{redis|rediss}://[<username>][:<password>@]<hostname>[:port][/<db>]`
        let conn_str = format!(
            "redis://{}:{}/{}",
            config.host, config.port, config.database
        );

        let lock_manager = LockManager::new(vec![conn_str]);

        Self { lock_manager }
    }

    pub async fn lock(&self, resource: &str) -> Option<Lock> {
        let lock = self.lock_manager.lock(resource.as_bytes(), 100).await;

        lock.ok()
    }

    pub async fn unlock(&self, lock: Lock<'_>) {
        self.lock_manager.unlock(&lock).await;
    }
}
