use std::{
    collections::HashMap,
    sync::Arc,
    time::{Duration, Instant},
};

use tavla_core::Player;
use tokio::sync::{mpsc, Mutex};

use super::session::GameSession;

pub type OutgoingSender = mpsc::UnboundedSender<String>;

#[derive(Default)]
struct SessionConnections {
    black: Option<OutgoingSender>,
    white: Option<OutgoingSender>,
}

/// Bir maçın oyun durumu (`game`) ile o an bağlı WebSocket bağlantılarının
/// (`connections`) birlikte tutulduğu kayıt. İkisi AYRI kilitli — bağlantı
/// açma/kapama, oyun durumunu mutasyona uğratmadan yapılabilsin diye.
pub struct ManagedSession {
    pub game: Mutex<GameSession>,
    connections: Mutex<SessionConnections>,
    /// Sırası gelen oyuncunun hamlesini BİTİRMESİ (`ConfirmTurn`) gereken an —
    /// zar atılınca kurulur (bkz. `ws::TURN_TIME_LIMIT`), tur onaylanınca
    /// temizlenir. Bağlı/kopuk olması ÖNEMLİ DEĞİL: bu süre dolduğunda sırası
    /// gelen oyuncu hükmen kaybeder (bkz. `ws::run_turn_watchdog`) — böylece
    /// hem "bağlantısı koptu" hem "bağlıyken oynamıyor" AYNI mekanizmayla ele alınır.
    turn_deadline: Mutex<Option<Instant>>,
    roll_not_before: Mutex<Option<Instant>>,
}

impl ManagedSession {
    fn new(gold_user_id: u64, purple_user_id: u64) -> Self {
        Self {
            game: Mutex::new(GameSession::new(gold_user_id, purple_user_id)),
            connections: Mutex::new(SessionConnections::default()),
            turn_deadline: Mutex::new(None),
            roll_not_before: Mutex::new(None),
        }
    }

    /// Bundan sonraki zar atışını `hold` kadar geciktirir (bkz. `roll_not_before`).
    pub async fn hold_next_roll_for(&self, hold: Duration) {
        *self.roll_not_before.lock().await = Some(Instant::now() + hold);
    }

    /// Sıradaki zar atışının daha ne kadar bekletilmesi gerektiği; bekleme
    /// hiç kurulmadıysa ya da süresi dolduysa `None`.
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
            Player::Black => conns.black = Some(sender),
            Player::White => conns.white = Some(sender),
        }
    }

    pub async fn clear_sender(&self, player: Player) {
        let mut conns = self.connections.lock().await;
        match player {
            Player::Black => conns.black = None,
            Player::White => conns.white = None,
        }
    }

    pub async fn is_connected(&self, player: Player) -> bool {
        let conns = self.connections.lock().await;
        match player {
            Player::Black => conns.black.is_some(),
            Player::White => conns.white.is_some(),
        }
    }

    pub async fn send_to(&self, player: Player, json: &str) {
        let conns = self.connections.lock().await;
        let sender = match player {
            Player::Black => &conns.black,
            Player::White => &conns.white,
        };
        if let Some(sender) = sender {
            let _ = sender.send(json.to_string());
        }
    }

    pub async fn send_to_both(&self, json_for_black: &str, json_for_white: &str) {
        self.send_to(Player::Black, json_for_black).await;
        self.send_to(Player::White, json_for_white).await;
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

    /// İkinci eleman `true` ise bu çağrı `game_id` için oturumu YENİ oluşturdu —
    /// çağıran (`ws::handle_upgrade`) bunu SADECE bu durumda bir tur-saati
    /// bekçisi (`ws::run_turn_watchdog`) başlatmak için kullanır; aksi halde
    /// (oyun zaten bağlıydı) o oyun için zaten bir bekçi çalışıyordur.
    pub async fn get_or_create(
        &self,
        game_id: u64,
        gold_user_id: u64,
        purple_user_id: u64,
    ) -> (Arc<ManagedSession>, bool) {
        let mut map = self.inner.lock().await;
        if let Some(existing) = map.get(&game_id) {
            return (existing.clone(), false);
        }
        let session = Arc::new(ManagedSession::new(gold_user_id, purple_user_id));
        map.insert(game_id, session.clone());
        (session, true)
    }

    /// `run_turn_watchdog`'un kendini sonlandırma sinyali: oyun (kazanma,
    /// hükmen kaybetme ya da her iki tarafın da kopması yüzünden) haritadan
    /// çoktan silinmişse bekçi döngüsü burada durur.
    pub async fn contains(&self, game_id: u64) -> bool {
        self.inner.lock().await.contains_key(&game_id)
    }

    /// `true` döner ancak ve ancak `game_id` GERÇEKTEN kaldırıldıysa — bu, bir
    /// oyunu bitirme hakkını atomik olarak "kazanmak" için kullanılır: normal
    /// bir kazanma hamlesi ile bir bağlantı kopması aynı anda/arka arkaya
    /// gerçekleşirse (bkz. `ws::finish_game`), sadece İLK çağıran bu haritadan
    /// gerçekten silme işlemini yapar, ikinci çağıran `false` alıp hiçbir şey
    /// yapmaz (skoru/DB satırını iki kez işlememek için).
    pub async fn remove(&self, game_id: u64) -> bool {
        self.inner.lock().await.remove(&game_id).is_some()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Hiç bekleme kurulmadıysa zar hemen atılabilir.
    #[tokio::test]
    async fn no_roll_hold_by_default() {
        let session = ManagedSession::new(1, 2);
        assert_eq!(session.roll_hold_remaining().await, None);
    }

    /// Bekleme kurulunca kalan süre görünür (ve kurulan süreyi aşmaz), süre
    /// dolunca tekrar `None` — sıradaki zar atışı o zaman serbest kalır.
    #[tokio::test]
    async fn roll_hold_counts_down_and_then_releases() {
        let session = ManagedSession::new(1, 2);
        let hold = Duration::from_millis(60);

        session.hold_next_roll_for(hold).await;
        let remaining = session.roll_hold_remaining().await.expect("bekleme kurulmuştu");
        assert!(remaining <= hold, "{remaining:?}");
        assert!(remaining > Duration::ZERO);

        tokio::time::sleep(hold + Duration::from_millis(40)).await;
        assert_eq!(session.roll_hold_remaining().await, None);
    }
}
