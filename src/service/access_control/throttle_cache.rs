use std::collections::HashMap;
use std::sync::Arc;

use entity::repository::user_throttles_repo::UserThrottlesRepository;
use tokio::sync::RwLock;

#[derive(Clone)]
pub struct ThrottleCache {
    limits: Arc<RwLock<HashMap<u64, u64>>>,
}

impl ThrottleCache {
    pub async fn load(repo: &UserThrottlesRepository) -> anyhow::Result<Self> {
        let rows = repo.get_all().await?;
        let limits = rows
            .into_iter()
            .map(|row| (row.user_id, row.requests_per_second))
            .collect();

        Ok(Self {
            limits: Arc::new(RwLock::new(limits)),
        })
    }

    pub async fn limit_for(&self, user_id: u64) -> Option<u64> {
        self.limits.read().await.get(&user_id).copied()
    }

    pub async fn upsert(&self, user_id: u64, requests_per_second: u64) {
        self.limits.write().await.insert(user_id, requests_per_second);
    }

    pub async fn remove(&self, user_id: u64) {
        self.limits.write().await.remove(&user_id);
    }
}
