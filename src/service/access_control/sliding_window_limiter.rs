use std::{
    collections::VecDeque,
    sync::Arc,
    time::{Duration, Instant},
};

use parking_lot::Mutex;

#[derive(Clone, Default)]
pub struct SlidingWindowLimiter {
    events: Arc<Mutex<VecDeque<Instant>>>,
}

impl SlidingWindowLimiter {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn check(&self, limit: u32, window: Duration) -> Result<(), Duration> {
        self.check_at(Instant::now(), limit, window)
    }

    fn check_at(&self, now: Instant, limit: u32, window: Duration) -> Result<(), Duration> {
        let mut events = self.events.lock();

        while events
            .front()
            .is_some_and(|oldest| now.saturating_duration_since(*oldest) >= window)
        {
            events.pop_front();
        }

        if events.len() >= limit as usize {
            let retry_after = events
                .front()
                .map_or(window, |oldest| (*oldest + window).saturating_duration_since(now));
            return Err(retry_after);
        }

        events.push_back(now);
        Ok(())
    }

    #[cfg(test)]
    fn recorded_events(&self) -> usize {
        self.events.lock().len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const HOUR: Duration = Duration::from_secs(60 * 60);

    fn minutes(count: u64) -> Duration {
        Duration::from_secs(count * 60)
    }

    #[test]
    fn allows_up_to_the_limit_then_reports_when_the_oldest_slot_frees_up() {
        let limiter = SlidingWindowLimiter::new();
        let start = Instant::now();

        assert!(limiter.check_at(start, 2, HOUR).is_ok());
        assert!(limiter.check_at(start + minutes(10), 2, HOUR).is_ok());

        let retry_after = limiter
            .check_at(start + minutes(20), 2, HOUR)
            .expect_err("third event inside the hour must be denied");
        assert_eq!(retry_after, minutes(40));
    }

    #[test]
    fn the_window_slides_instead_of_resetting_all_at_once() {
        let limiter = SlidingWindowLimiter::new();
        let start = Instant::now();

        assert!(limiter.check_at(start, 2, HOUR).is_ok());
        assert!(limiter.check_at(start + minutes(30), 2, HOUR).is_ok());
        assert_eq!(limiter.check_at(start + minutes(45), 2, HOUR), Err(minutes(15)));

        assert!(limiter.check_at(start + minutes(61), 2, HOUR).is_ok());
        assert_eq!(limiter.check_at(start + minutes(62), 2, HOUR), Err(minutes(28)));
    }

    #[test]
    fn denied_attempts_are_not_recorded_and_do_not_extend_the_block() {
        let limiter = SlidingWindowLimiter::new();
        let start = Instant::now();
        assert!(limiter.check_at(start, 1, HOUR).is_ok());

        for second in 1..=50 {
            assert!(limiter.check_at(start + Duration::from_secs(second), 1, HOUR).is_err());
        }

        assert_eq!(limiter.recorded_events(), 1);
        assert!(limiter.check_at(start + HOUR, 1, HOUR).is_ok());
    }

    #[test]
    fn a_limit_of_zero_blocks_everything_for_the_whole_window() {
        let limiter = SlidingWindowLimiter::new();

        assert_eq!(limiter.check_at(Instant::now(), 0, HOUR), Err(HOUR));
        assert_eq!(limiter.recorded_events(), 0);
    }

    #[test]
    fn expired_events_are_dropped_so_memory_stays_bounded_by_the_limit() {
        let limiter = SlidingWindowLimiter::new();
        let start = Instant::now();
        for second in 0..100 {
            assert!(limiter.check_at(start + Duration::from_secs(second), 100, HOUR).is_ok());
        }
        assert_eq!(limiter.recorded_events(), 100);

        assert!(limiter.check_at(start + HOUR + minutes(5), 100, HOUR).is_ok());

        assert_eq!(limiter.recorded_events(), 1);
    }
}
