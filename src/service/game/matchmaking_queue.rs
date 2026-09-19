use std::{
    collections::HashMap,
    sync::Arc,
    time::{Duration, Instant},
};

use tokio::sync::Mutex;

/// `login_screen.rs::MATCHMAKING_RETRY_SECONDS` (1.5s) ile bir istemcinin
/// gerçekten yaşıyorsa ne sıklıkla tekrar `advance` çağıracağının 4 katı — bir
/// istemci uygulamayı kapatıp `DELETE /v1/games/quick-match` ile sıradan hiç
/// çıkmadan giderse (bildirilen "hiç aktif client yokken anında eşleşiyor"
/// hatası), bu süre boyunca tekrar aranmayan bekleyen kullanıcı hayalet
/// sayılıp bir dahaki `advance` çağrısında atılır.
const WAITING_STALE_AFTER: Duration = Duration::from_secs(6);

#[derive(Clone, Default)]
pub struct MatchmakingQueue {
    inner: Arc<Mutex<MatchmakingQueueInner>>,
}

#[derive(Default)]
struct MatchmakingQueueInner {
    waiting: Option<WaitingEntry>,
    matched: HashMap<u64, u64>,
}

struct WaitingEntry {
    user_id: u64,
    last_seen: Instant,
}

pub enum QueueEvent {
    AlreadyMatched(u64),
    NowWaiting,
    OpponentFound(u64),
}

impl MatchmakingQueue {
    pub fn new() -> Self {
        Self::default()
    }

    pub async fn advance(&self, user_id: u64) -> QueueEvent {
        let mut inner = self.inner.lock().await;

        if let Some(game_id) = inner.matched.remove(&user_id) {
            return QueueEvent::AlreadyMatched(game_id);
        }

        let waiting_state = inner
            .waiting
            .as_ref()
            .map(|entry| (entry.user_id, entry.last_seen.elapsed()));

        match waiting_state {
            Some((waiting_user_id, _)) if waiting_user_id == user_id => {
                inner.waiting = Some(WaitingEntry { user_id, last_seen: Instant::now() });
                QueueEvent::NowWaiting
            }
            Some((opponent_id, elapsed)) if elapsed <= WAITING_STALE_AFTER => {
                inner.waiting = None;
                QueueEvent::OpponentFound(opponent_id)
            }
            _ => {
                inner.waiting = Some(WaitingEntry { user_id, last_seen: Instant::now() });
                QueueEvent::NowWaiting
            }
        }
    }

    pub async fn record_match(&self, opponent_id: u64, game_id: u64) {
        let mut inner = self.inner.lock().await;
        inner.matched.insert(opponent_id, game_id);
    }

    pub async fn leave(&self, user_id: u64) {
        let mut inner = self.inner.lock().await;
        if inner.waiting.as_ref().is_some_and(|entry| entry.user_id == user_id) {
            inner.waiting = None;
        }
        inner.matched.remove(&user_id);
    }
}
