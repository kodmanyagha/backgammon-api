use std::collections::HashMap;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

use parking_lot::Mutex;

const PURGE_THRESHOLD: usize = 10_000;

#[derive(Clone, Default)]
pub struct RateLimiter {
    windows: Arc<Mutex<HashMap<u64, (u64, u64)>>>,
}

impl RateLimiter {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn allow(&self, user_id: u64, requests_per_second: u64) -> bool {
        let current_second = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        let mut windows = self.windows.lock();

        if windows.len() > PURGE_THRESHOLD {
            windows.retain(|_, (window_second, _)| *window_second == current_second);
        }

        let entry = windows.entry(user_id).or_insert((current_second, 0));

        if entry.0 != current_second {
            entry.0 = current_second;
            entry.1 = 0;
        }

        if entry.1 >= requests_per_second {
            return false;
        }

        entry.1 += 1;
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stale_windows_are_dropped_once_the_map_grows_past_the_threshold() {
        let limiter = RateLimiter::new();
        {
            let mut windows = limiter.windows.lock();
            for user_id in 0..(PURGE_THRESHOLD as u64 + 5) {
                windows.insert(user_id, (0, 1));
            }
        }

        assert!(limiter.allow(1, 10));

        assert_eq!(limiter.windows.lock().len(), 1);
    }

    #[test]
    fn enforces_the_per_second_budget() {
        let limiter = RateLimiter::new();

        assert!(limiter.allow(7, 2));
        assert!(limiter.allow(7, 2));
        assert!(!limiter.allow(7, 2));
    }
}
