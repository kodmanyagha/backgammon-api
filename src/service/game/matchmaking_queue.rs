use std::{sync::Arc, time::Duration};

use serde::{Deserialize, Serialize};
use tokio::sync::Mutex;

use crate::service::{game::live_state::now_unix_ms, redis_service::redis_service::RedisService};

pub const WAIT_TIMEOUT: Duration = Duration::from_secs(60);

const WAITING_STALE_AFTER: Duration = Duration::from_secs(6);

const MATCH_PICKUP_WINDOW: Duration = WAITING_STALE_AFTER;

const WAITING_KEY: &str = "mm:waiting";
const WAITING_KEY_TTL_SECONDS: u64 = 2 * WAIT_TIMEOUT.as_secs();

fn matched_key(user_id: u64) -> String {
    format!("mm:matched:{user_id}")
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
struct WaitingEntry {
    user_id: u64,
    joined_at_ms: i64,
    last_seen_ms: i64,
}

#[derive(Debug, PartialEq, Eq)]
pub enum QueueEvent {
    AlreadyMatched(u64),
    NowWaiting,
    OpponentFound(u64),
    NoOpponent,
}

#[derive(Debug, PartialEq, Eq)]
enum QueueAction {
    HandOutMatch(u64),
    KeepWaiting(WaitingEntry),
    PairWith(u64),
    StartWaiting(WaitingEntry),
    TimeOut,
}

fn is_older_than(now_ms: i64, then_ms: i64, limit: Duration) -> bool {
    now_ms.saturating_sub(then_ms) > limit.as_millis() as i64
}

fn decide(now_ms: i64, user_id: u64, matched_game: Option<u64>, waiting: Option<WaitingEntry>) -> QueueAction {
    if let Some(game_id) = matched_game {
        return QueueAction::HandOutMatch(game_id);
    }

    let fresh_entry = || WaitingEntry { user_id, joined_at_ms: now_ms, last_seen_ms: now_ms };

    match waiting {
        Some(entry) if entry.user_id == user_id => {
            if is_older_than(now_ms, entry.last_seen_ms, WAITING_STALE_AFTER) {
                QueueAction::StartWaiting(fresh_entry())
            } else if is_older_than(now_ms, entry.joined_at_ms, WAIT_TIMEOUT) {
                QueueAction::TimeOut
            } else {
                QueueAction::KeepWaiting(WaitingEntry { last_seen_ms: now_ms, ..entry })
            }
        }
        Some(entry) if !is_older_than(now_ms, entry.last_seen_ms, WAITING_STALE_AFTER) => {
            QueueAction::PairWith(entry.user_id)
        }
        _ => QueueAction::StartWaiting(fresh_entry()),
    }
}

#[derive(Clone)]
pub struct MatchmakingQueue {
    redis: RedisService,
    serialize_requests: Arc<Mutex<()>>,
}

impl MatchmakingQueue {
    pub fn new(redis: RedisService) -> Self {
        Self { redis, serialize_requests: Arc::new(Mutex::new(())) }
    }

    pub async fn advance(&self, user_id: u64) -> anyhow::Result<QueueEvent> {
        let _serialized = self.serialize_requests.lock().await;

        let matched_game = self
            .redis
            .get(&matched_key(user_id))
            .await?
            .and_then(|game_id| game_id.parse::<u64>().ok());
        let waiting = self
            .redis
            .get(WAITING_KEY)
            .await?
            .and_then(|json| serde_json::from_str::<WaitingEntry>(&json).ok());

        match decide(now_unix_ms(), user_id, matched_game, waiting) {
            QueueAction::HandOutMatch(game_id) => {
                self.redis.del(&matched_key(user_id)).await?;
                Ok(QueueEvent::AlreadyMatched(game_id))
            }
            QueueAction::KeepWaiting(entry) | QueueAction::StartWaiting(entry) => {
                self.store_waiting(&entry).await?;
                Ok(QueueEvent::NowWaiting)
            }
            QueueAction::PairWith(opponent_id) => {
                self.redis.del(WAITING_KEY).await?;
                Ok(QueueEvent::OpponentFound(opponent_id))
            }
            QueueAction::TimeOut => {
                self.redis.del(WAITING_KEY).await?;
                Ok(QueueEvent::NoOpponent)
            }
        }
    }

    pub async fn record_match(&self, waiting_user_id: u64, game_id: u64) -> anyhow::Result<()> {
        self.redis
            .set_with_ttl(&matched_key(waiting_user_id), &game_id.to_string(), MATCH_PICKUP_WINDOW.as_secs())
            .await
    }

    pub async fn leave(&self, user_id: u64) -> anyhow::Result<()> {
        let _serialized = self.serialize_requests.lock().await;

        let waiting = self
            .redis
            .get(WAITING_KEY)
            .await?
            .and_then(|json| serde_json::from_str::<WaitingEntry>(&json).ok());
        if waiting.is_some_and(|entry| entry.user_id == user_id) {
            self.redis.del(WAITING_KEY).await?;
        }
        self.redis.del(&matched_key(user_id)).await?;

        Ok(())
    }

    async fn store_waiting(&self, entry: &WaitingEntry) -> anyhow::Result<()> {
        self.redis
            .set_with_ttl(WAITING_KEY, &serde_json::to_string(entry)?, WAITING_KEY_TTL_SECONDS)
            .await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const USER: u64 = 1;
    const OTHER: u64 = 2;
    const GAME: u64 = 77;
    const NOW_MS: i64 = 1_790_000_000_000;

    fn entry(user_id: u64, joined_ago: Duration, seen_ago: Duration) -> WaitingEntry {
        WaitingEntry {
            user_id,
            joined_at_ms: NOW_MS - joined_ago.as_millis() as i64,
            last_seen_ms: NOW_MS - seen_ago.as_millis() as i64,
        }
    }

    #[test]
    fn a_lone_user_starts_waiting() {
        assert_eq!(
            decide(NOW_MS, USER, None, None),
            QueueAction::StartWaiting(WaitingEntry { user_id: USER, joined_at_ms: NOW_MS, last_seen_ms: NOW_MS })
        );
    }

    #[test]
    fn the_next_user_is_paired_with_the_waiting_one() {
        let waiting = entry(OTHER, Duration::from_secs(3), Duration::from_secs(1));

        assert_eq!(decide(NOW_MS, USER, None, Some(waiting)), QueueAction::PairWith(OTHER));
    }

    #[test]
    fn polling_keeps_the_waiting_user_alive_without_resetting_when_they_joined() {
        let waiting = entry(USER, Duration::from_secs(20), Duration::from_millis(1500));

        let QueueAction::KeepWaiting(kept) = decide(NOW_MS, USER, None, Some(waiting.clone())) else {
            panic!("bekleme sürmeli");
        };

        assert_eq!(kept.joined_at_ms, waiting.joined_at_ms);
        assert_eq!(kept.last_seen_ms, NOW_MS);
    }

    #[test]
    fn after_the_wait_timeout_the_search_ends_with_no_opponent() {
        let waiting = entry(USER, WAIT_TIMEOUT + Duration::from_secs(1), Duration::from_millis(1500));

        assert_eq!(decide(NOW_MS, USER, None, Some(waiting)), QueueAction::TimeOut);
    }

    #[test]
    fn the_wait_lasts_the_full_timeout() {
        let waiting = entry(USER, WAIT_TIMEOUT, Duration::from_millis(1500));

        assert!(matches!(decide(NOW_MS, USER, None, Some(waiting)), QueueAction::KeepWaiting(_)));
    }

    #[test]
    fn a_user_who_left_and_came_back_later_starts_a_fresh_search_instead_of_timing_out() {
        let old_attempt = entry(USER, Duration::from_secs(300), Duration::from_secs(290));

        assert_eq!(
            decide(NOW_MS, USER, None, Some(old_attempt)),
            QueueAction::StartWaiting(WaitingEntry { user_id: USER, joined_at_ms: NOW_MS, last_seen_ms: NOW_MS })
        );
    }

    #[test]
    fn a_waiting_user_who_stopped_polling_is_replaced_by_the_next_user() {
        let ghost = entry(OTHER, Duration::from_secs(30), WAITING_STALE_AFTER + Duration::from_secs(1));

        assert_eq!(
            decide(NOW_MS, USER, None, Some(ghost)),
            QueueAction::StartWaiting(WaitingEntry { user_id: USER, joined_at_ms: NOW_MS, last_seen_ms: NOW_MS })
        );
    }

    #[test]
    fn a_match_made_while_waiting_is_handed_out_before_anything_else() {
        let waiting = entry(OTHER, Duration::from_secs(3), Duration::from_secs(1));

        assert_eq!(decide(NOW_MS, USER, Some(GAME), Some(waiting)), QueueAction::HandOutMatch(GAME));
    }

    #[test]
    fn the_pickup_window_is_as_long_as_the_ghost_timeout() {
        assert_eq!(MATCH_PICKUP_WINDOW, WAITING_STALE_AFTER);
        assert_eq!(WAIT_TIMEOUT, Duration::from_secs(60));
    }

    #[test]
    fn the_queue_keys_are_scoped_per_user() {
        assert_eq!(matched_key(5), "mm:matched:5");
        assert_ne!(matched_key(1), matched_key(11));
    }
}
