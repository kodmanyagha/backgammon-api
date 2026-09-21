use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};

use super::session::GameSession;

pub const REHYDRATED_TURN_GRACE: Duration = Duration::from_secs(30);

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct LiveSnapshot {
    pub session: GameSession,
    #[serde(default)]
    pub turn_deadline_at_ms: Option<i64>,
    pub saved_at_ms: i64,
}

pub fn now_unix_ms() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|elapsed| elapsed.as_millis() as i64)
        .unwrap_or(0)
}

pub fn deadline_to_unix_ms(deadline: Instant, now: Instant, now_ms: i64) -> i64 {
    now_ms + deadline.saturating_duration_since(now).as_millis() as i64
}

pub fn deadline_from_unix_ms(deadline_at_ms: i64, now: Instant, now_ms: i64) -> Instant {
    let remaining = Duration::from_millis((deadline_at_ms - now_ms).max(0) as u64);
    now + remaining.max(REHYDRATED_TURN_GRACE)
}

#[cfg(test)]
mod tests {
    use tavla_core::Player;

    use super::*;

    const NOW_MS: i64 = 1_790_000_000_000;

    #[test]
    fn a_deadline_round_trips_through_unix_time() {
        let now = Instant::now();
        let deadline = now + Duration::from_secs(45);

        let at_ms = deadline_to_unix_ms(deadline, now, NOW_MS);

        assert_eq!(at_ms, NOW_MS + 45_000);
        assert_eq!(deadline_from_unix_ms(at_ms, now, NOW_MS), deadline);
    }

    #[test]
    fn a_deadline_that_already_passed_is_saved_as_now() {
        let now = Instant::now();
        let overdue = now.checked_sub(Duration::from_secs(5)).unwrap_or(now);

        assert_eq!(deadline_to_unix_ms(overdue, now, NOW_MS), NOW_MS);
    }

    #[test]
    fn a_restored_deadline_never_gives_less_than_the_grace_period() {
        let now = Instant::now();

        assert_eq!(deadline_from_unix_ms(NOW_MS + 3_000, now, NOW_MS), now + REHYDRATED_TURN_GRACE);
        assert_eq!(deadline_from_unix_ms(NOW_MS - 60_000, now, NOW_MS), now + REHYDRATED_TURN_GRACE);
        assert_eq!(deadline_from_unix_ms(NOW_MS + 50_000, now, NOW_MS), now + Duration::from_secs(50));
    }

    #[test]
    fn a_snapshot_survives_a_json_round_trip() {
        let snapshot = LiveSnapshot {
            session: GameSession::new(1, 2, Player::Black),
            turn_deadline_at_ms: Some(NOW_MS + 1),
            saved_at_ms: NOW_MS,
        };

        let json = serde_json::to_string(&snapshot).unwrap();

        assert_eq!(serde_json::from_str::<LiveSnapshot>(&json).unwrap(), snapshot);
    }

    #[test]
    fn a_snapshot_without_a_deadline_loads() {
        let json = serde_json::to_string(&LiveSnapshot {
            session: GameSession::new(1, 2, Player::White),
            turn_deadline_at_ms: None,
            saved_at_ms: NOW_MS,
        })
        .unwrap()
        .replace(",\"turn_deadline_at_ms\":null", "");

        assert_eq!(serde_json::from_str::<LiveSnapshot>(&json).unwrap().turn_deadline_at_ms, None);
    }
}
