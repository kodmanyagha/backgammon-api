use std::{
    collections::HashMap,
    net::{IpAddr, Ipv6Addr},
    sync::Arc,
    time::{Duration, Instant},
};

use parking_lot::Mutex;

const PURGE_INTERVAL: Duration = Duration::from_secs(60);
const IPV6_NETWORK_PREFIX_MASK: u128 = !0u128 << 64;

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum RateLimitScope {
    AuthRequests,
    PublicRequests,
    GuestCreation,
    CaptchaStart,
    CaptchaVerify,
}

struct WindowCounter {
    expires_at: Instant,
    count: u32,
}

struct LimiterState {
    counters: HashMap<(RateLimitScope, IpAddr), WindowCounter>,
    last_purge_at: Instant,
}

#[derive(Clone)]
pub struct IpRateLimiter {
    state: Arc<Mutex<LimiterState>>,
}

impl Default for IpRateLimiter {
    fn default() -> Self {
        Self::new()
    }
}

impl IpRateLimiter {
    pub fn new() -> Self {
        Self {
            state: Arc::new(Mutex::new(LimiterState {
                counters: HashMap::new(),
                last_purge_at: Instant::now(),
            })),
        }
    }

    pub fn check(
        &self,
        scope: RateLimitScope,
        ip: IpAddr,
        limit: u32,
        window: Duration,
    ) -> Result<(), Duration> {
        self.check_at(Instant::now(), scope, ip, limit, window)
    }

    fn check_at(
        &self,
        now: Instant,
        scope: RateLimitScope,
        ip: IpAddr,
        limit: u32,
        window: Duration,
    ) -> Result<(), Duration> {
        let mut state = self.state.lock();

        if now.saturating_duration_since(state.last_purge_at) >= PURGE_INTERVAL {
            state.counters.retain(|_, counter| counter.expires_at > now);
            state.last_purge_at = now;
        }

        let counter = state
            .counters
            .entry((scope, network_key(ip)))
            .or_insert(WindowCounter {
                expires_at: now + window,
                count: 0,
            });

        if now >= counter.expires_at {
            counter.expires_at = now + window;
            counter.count = 0;
        }

        if counter.count >= limit {
            return Err(counter.expires_at.saturating_duration_since(now));
        }

        counter.count += 1;
        Ok(())
    }

    #[cfg(test)]
    fn tracked_clients(&self) -> usize {
        self.state.lock().counters.len()
    }
}

fn network_key(ip: IpAddr) -> IpAddr {
    match ip {
        IpAddr::V4(_) => ip,
        IpAddr::V6(v6) => match v6.to_ipv4_mapped() {
            Some(v4) => IpAddr::V4(v4),
            None => IpAddr::V6(Ipv6Addr::from(u128::from(v6) & IPV6_NETWORK_PREFIX_MASK)),
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const WINDOW: Duration = Duration::from_secs(60);

    fn ipv4(last_octet: u8) -> IpAddr {
        IpAddr::V4(std::net::Ipv4Addr::new(203, 0, 113, last_octet))
    }

    fn ipv6(text: &str) -> IpAddr {
        IpAddr::V6(text.parse().expect("valid ipv6"))
    }

    #[test]
    fn allows_up_to_the_limit_and_then_reports_the_remaining_window() {
        let limiter = IpRateLimiter::new();
        let start = Instant::now();

        for _ in 0..3 {
            assert!(limiter.check_at(start, RateLimitScope::AuthRequests, ipv4(1), 3, WINDOW).is_ok());
        }

        let later = start + Duration::from_secs(20);
        let retry_after = limiter
            .check_at(later, RateLimitScope::AuthRequests, ipv4(1), 3, WINDOW)
            .expect_err("fourth request must be denied");
        assert_eq!(retry_after, Duration::from_secs(40));
    }

    #[test]
    fn window_expiry_resets_the_budget() {
        let limiter = IpRateLimiter::new();
        let start = Instant::now();

        assert!(limiter.check_at(start, RateLimitScope::AuthRequests, ipv4(1), 1, WINDOW).is_ok());
        assert!(limiter.check_at(start, RateLimitScope::AuthRequests, ipv4(1), 1, WINDOW).is_err());
        assert!(limiter
            .check_at(start + WINDOW, RateLimitScope::AuthRequests, ipv4(1), 1, WINDOW)
            .is_ok());
    }

    #[test]
    fn clients_and_scopes_have_separate_budgets() {
        let limiter = IpRateLimiter::new();
        let now = Instant::now();

        assert!(limiter.check_at(now, RateLimitScope::AuthRequests, ipv4(1), 1, WINDOW).is_ok());
        assert!(limiter.check_at(now, RateLimitScope::AuthRequests, ipv4(2), 1, WINDOW).is_ok());
        assert!(limiter.check_at(now, RateLimitScope::GuestCreation, ipv4(1), 1, WINDOW).is_ok());
        assert!(limiter.check_at(now, RateLimitScope::AuthRequests, ipv4(1), 1, WINDOW).is_err());
    }

    #[test]
    fn ipv6_addresses_in_the_same_slash_64_share_one_budget() {
        let limiter = IpRateLimiter::new();
        let now = Instant::now();

        assert!(limiter
            .check_at(now, RateLimitScope::GuestCreation, ipv6("2001:db8:1:2::1"), 1, WINDOW)
            .is_ok());
        assert!(limiter
            .check_at(now, RateLimitScope::GuestCreation, ipv6("2001:db8:1:2:ffff::9"), 1, WINDOW)
            .is_err());
        assert!(limiter
            .check_at(now, RateLimitScope::GuestCreation, ipv6("2001:db8:1:3::1"), 1, WINDOW)
            .is_ok());
    }

    #[test]
    fn ipv4_mapped_ipv6_shares_the_budget_with_plain_ipv4() {
        let limiter = IpRateLimiter::new();
        let now = Instant::now();

        assert!(limiter.check_at(now, RateLimitScope::AuthRequests, ipv4(7), 1, WINDOW).is_ok());
        assert!(limiter
            .check_at(now, RateLimitScope::AuthRequests, ipv6("::ffff:203.0.113.7"), 1, WINDOW)
            .is_err());
    }

    #[test]
    fn expired_windows_are_purged_so_memory_does_not_grow_forever() {
        let limiter = IpRateLimiter::new();
        let start = Instant::now();

        for last_octet in 0..=200u8 {
            let _ = limiter.check_at(start, RateLimitScope::AuthRequests, ipv4(last_octet), 5, WINDOW);
        }
        assert_eq!(limiter.tracked_clients(), 201);

        let much_later = start + WINDOW + PURGE_INTERVAL;
        let _ = limiter.check_at(much_later, RateLimitScope::AuthRequests, ipv4(1), 5, WINDOW);
        assert_eq!(limiter.tracked_clients(), 1);
    }
}
