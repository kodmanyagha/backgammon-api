use std::{
    collections::HashMap,
    sync::Arc,
    time::{Duration, Instant},
};

use tavla_core::Player;
use tokio::sync::{mpsc, Mutex};

use super::{
    live_state::{deadline_from_unix_ms, deadline_to_unix_ms, now_unix_ms, LiveSnapshot},
    session::GameSession,
};

pub type OutgoingSender = mpsc::UnboundedSender<String>;

#[derive(Clone, Copy, Debug)]
pub struct AiOffer {
    pub to: Player,
    pub sent_at: Instant,
}

struct SessionConnections {
    white: Option<OutgoingSender>,
    black: Option<OutgoingSender>,
    white_gone_since: Option<Instant>,
    black_gone_since: Option<Instant>,
}

impl SessionConnections {
    fn new() -> Self {
        let created_at = Instant::now();
        Self { white: None, black: None, white_gone_since: Some(created_at), black_gone_since: Some(created_at) }
    }
}

pub struct ManagedSession {
    pub game: Mutex<GameSession>,
    connections: Mutex<SessionConnections>,
    turn_deadline: Mutex<Option<Instant>>,
    roll_not_before: Mutex<Option<Instant>>,
    ai_offer: Mutex<Option<AiOffer>>,
    auto_play: Mutex<Option<Player>>,
    ai_next_action_at: Mutex<Option<Instant>>,
    persist_lock: Mutex<()>,
}

impl ManagedSession {
    pub fn new(game: GameSession) -> Self {
        Self::build(game, None)
    }

    fn build(game: GameSession, turn_deadline: Option<Instant>) -> Self {
        Self {
            game: Mutex::new(game),
            connections: Mutex::new(SessionConnections::new()),
            turn_deadline: Mutex::new(turn_deadline),
            roll_not_before: Mutex::new(None),
            ai_offer: Mutex::new(None),
            auto_play: Mutex::new(None),
            ai_next_action_at: Mutex::new(None),
            persist_lock: Mutex::new(()),
        }
    }

    pub fn from_snapshot(snapshot: LiveSnapshot) -> Self {
        let restored_deadline = snapshot
            .turn_deadline_at_ms
            .map(|deadline_at_ms| deadline_from_unix_ms(deadline_at_ms, Instant::now(), now_unix_ms()));

        Self::build(snapshot.session, restored_deadline)
    }

    pub async fn snapshot(&self) -> LiveSnapshot {
        let game = self.game.lock().await.clone();
        let turn_deadline_at_ms = self
            .turn_deadline()
            .await
            .map(|deadline| deadline_to_unix_ms(deadline, Instant::now(), now_unix_ms()));

        LiveSnapshot { session: game, turn_deadline_at_ms, saved_at_ms: now_unix_ms() }
    }

    pub async fn lock_persist(&self) -> tokio::sync::MutexGuard<'_, ()> {
        self.persist_lock.lock().await
    }

    pub async fn hold_next_roll_for(&self, hold: Duration) {
        *self.roll_not_before.lock().await = Some(Instant::now() + hold);
    }

    pub async fn roll_hold_remaining(&self) -> Option<Duration> {
        let not_before = (*self.roll_not_before.lock().await)?;
        let remaining = not_before.saturating_duration_since(Instant::now());
        (!remaining.is_zero()).then_some(remaining)
    }

    pub async fn set_turn_deadline(&self, deadline: Instant) {
        *self.turn_deadline.lock().await = Some(deadline);
    }

    pub async fn clear_turn_deadline(&self) {
        *self.turn_deadline.lock().await = None;
    }

    pub async fn turn_deadline(&self) -> Option<Instant> {
        *self.turn_deadline.lock().await
    }

    pub async fn set_sender(&self, player: Player, sender: OutgoingSender) {
        let mut conns = self.connections.lock().await;
        match player {
            Player::White => {
                conns.white = Some(sender);
                conns.white_gone_since = None;
            }
            Player::Black => {
                conns.black = Some(sender);
                conns.black_gone_since = None;
            }
        }
    }

    pub async fn clear_sender(&self, player: Player) {
        let mut conns = self.connections.lock().await;
        match player {
            Player::White => {
                conns.white = None;
                conns.white_gone_since = Some(Instant::now());
            }
            Player::Black => {
                conns.black = None;
                conns.black_gone_since = Some(Instant::now());
            }
        }
    }

    pub async fn disconnected_for(&self, player: Player) -> Option<Duration> {
        let conns = self.connections.lock().await;
        let gone_since = match player {
            Player::White => conns.white_gone_since,
            Player::Black => conns.black_gone_since,
        };
        gone_since.map(|since| since.elapsed())
    }

    pub async fn ai_offer(&self) -> Option<AiOffer> {
        *self.ai_offer.lock().await
    }

    pub async fn set_ai_offer(&self, offer: Option<AiOffer>) {
        *self.ai_offer.lock().await = offer;
    }

    pub async fn auto_play(&self) -> Option<Player> {
        *self.auto_play.lock().await
    }

    pub async fn set_auto_play(&self, player: Option<Player>) {
        *self.auto_play.lock().await = player;
    }

    pub async fn ai_next_action_at(&self) -> Option<Instant> {
        *self.ai_next_action_at.lock().await
    }

    pub async fn set_ai_next_action_at(&self, at: Option<Instant>) {
        *self.ai_next_action_at.lock().await = at;
    }

    pub async fn is_connected(&self, player: Player) -> bool {
        let conns = self.connections.lock().await;
        match player {
            Player::White => conns.white.is_some(),
            Player::Black => conns.black.is_some(),
        }
    }

    pub async fn send_to(&self, player: Player, json: &str) {
        let conns = self.connections.lock().await;
        let sender = match player {
            Player::White => &conns.white,
            Player::Black => &conns.black,
        };
        if let Some(sender) = sender {
            let _ = sender.send(json.to_string());
        }
    }

    pub async fn send_to_both(&self, json_for_white: &str, json_for_black: &str) {
        self.send_to(Player::White, json_for_white).await;
        self.send_to(Player::Black, json_for_black).await;
    }
}

#[derive(Clone, Default)]
pub struct GameSessions {
    inner: Arc<Mutex<HashMap<u64, Arc<ManagedSession>>>>,
}

impl GameSessions {
    pub fn new() -> Self {
        Self::default()
    }

    pub async fn get(&self, game_id: u64) -> Option<Arc<ManagedSession>> {
        self.inner.lock().await.get(&game_id).cloned()
    }

    pub async fn insert_if_absent(
        &self,
        game_id: u64,
        session: Arc<ManagedSession>,
    ) -> (Arc<ManagedSession>, bool) {
        let mut map = self.inner.lock().await;
        if let Some(existing) = map.get(&game_id) {
            return (existing.clone(), false);
        }
        map.insert(game_id, session.clone());
        (session, true)
    }

    pub async fn contains(&self, game_id: u64) -> bool {
        self.inner.lock().await.contains_key(&game_id)
    }

    pub async fn remove(&self, game_id: u64) -> bool {
        self.inner.lock().await.remove(&game_id).is_some()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn disconnection_time_is_tracked_per_side() {
        let session = ManagedSession::new(GameSession::new(1, 2, Player::White));
        assert!(session.disconnected_for(Player::White).await.is_some());
        assert!(session.disconnected_for(Player::Black).await.is_some());

        let (sender, _receiver) = mpsc::unbounded_channel();
        session.set_sender(Player::White, sender).await;
        assert!(session.is_connected(Player::White).await);
        assert_eq!(session.disconnected_for(Player::White).await, None);
        assert!(session.disconnected_for(Player::Black).await.is_some());

        session.clear_sender(Player::White).await;
        assert!(session.disconnected_for(Player::White).await.is_some());
    }

    #[tokio::test]
    async fn the_ai_offer_and_schedule_start_empty_and_can_be_set_and_cleared() {
        let session = ManagedSession::new(GameSession::new(1, 2, Player::White));
        assert!(session.ai_offer().await.is_none());
        assert_eq!(session.ai_next_action_at().await, None);

        session.set_ai_offer(Some(AiOffer { to: Player::Black, sent_at: Instant::now() })).await;
        session.set_ai_next_action_at(Some(Instant::now())).await;
        assert_eq!(session.ai_offer().await.map(|offer| offer.to), Some(Player::Black));
        assert!(session.ai_next_action_at().await.is_some());

        session.set_ai_offer(None).await;
        session.set_ai_next_action_at(None).await;
        assert!(session.ai_offer().await.is_none());
        assert_eq!(session.ai_next_action_at().await, None);
    }

    #[tokio::test]
    async fn auto_play_starts_off_and_can_be_switched_on_and_off() {
        let session = ManagedSession::new(GameSession::new(1, 2, Player::White));
        assert_eq!(session.auto_play().await, None);

        session.set_auto_play(Some(Player::Black)).await;
        assert_eq!(session.auto_play().await, Some(Player::Black));

        session.set_auto_play(None).await;
        assert_eq!(session.auto_play().await, None);
    }

    #[tokio::test]
    async fn no_roll_hold_by_default() {
        let session = ManagedSession::new(GameSession::new(1, 2, Player::White));
        assert_eq!(session.roll_hold_remaining().await, None);
    }

    #[tokio::test]
    async fn roll_hold_counts_down_and_then_releases() {
        let session = ManagedSession::new(GameSession::new(1, 2, Player::White));
        let hold = Duration::from_millis(60);

        session.hold_next_roll_for(hold).await;
        let remaining = session.roll_hold_remaining().await.expect("bekleme kurulmuştu");
        assert!(remaining <= hold, "{remaining:?}");
        assert!(remaining > Duration::ZERO);

        tokio::time::sleep(hold + Duration::from_millis(40)).await;
        assert_eq!(session.roll_hold_remaining().await, None);
    }
}
