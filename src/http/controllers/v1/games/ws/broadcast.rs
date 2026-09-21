use std::sync::Arc;
use std::time::Instant;

use tavla_core::Player;

use super::NEXT_ROUND_DELAY;
use crate::service::game::{
    live_state::now_unix_ms,
    session::GameSession,
    sessions::ManagedSession,
    ws_protocol::{LastMoveDto, ServerMessage},
};

pub(super) async fn notify_presence(
    session: &Arc<ManagedSession>,
    player: Player,
    message: ServerMessage,
) {
    let payload = serde_json::to_string(&message).unwrap_or_default();
    session.send_to(player.opponent(), &payload).await;
}

pub(super) async fn send_error(session: &Arc<ManagedSession>, player: Player, reason: &str) {
    let payload = serde_json::to_string(&ServerMessage::Error {
        message: reason.to_string(),
    })
    .unwrap_or_default();
    session.send_to(player, &payload).await;
}

fn game_ended_message(game: &GameSession, next_round_in_ms: u64) -> Option<ServerMessage> {
    Some(ServerMessage::GameEnded {
        winner: game.last_round_winner?,
        white_score: game.white_score,
        black_score: game.black_score,
        next_round_in_ms,
        mars: game.last_round_mars,
    })
}

pub(super) async fn broadcast_game_ended(session: &Arc<ManagedSession>) {
    let message = game_ended_message(
        &*session.game.lock().await,
        NEXT_ROUND_DELAY.as_millis() as u64,
    );
    let Some(message) = message else { return };

    let payload = serde_json::to_string(&message).unwrap_or_default();
    session.send_to_both(&payload, &payload).await;
}

pub(super) async fn notify_round_countdown(session: &Arc<ManagedSession>, player: Player) {
    let message = {
        let game = session.game.lock().await;
        let Some(next_round_at_ms) = game.next_round_at_ms() else {
            return;
        };
        game_ended_message(&game, (next_round_at_ms - now_unix_ms()).max(0) as u64)
    };
    let Some(message) = message else { return };

    let payload = serde_json::to_string(&message).unwrap_or_default();
    session.send_to(player, &payload).await;
}

pub(super) async fn broadcast_state(session: &Arc<ManagedSession>) {
    broadcast_state_after_move(session, None).await;
}

pub(super) async fn reject_action(session: &Arc<ManagedSession>, player: Player, reason: &str) {
    send_error(session, player, reason).await;
    send_state_to(session, player).await;
}

pub(super) async fn send_state_to(session: &Arc<ManagedSession>, player: Player) {
    let (json_white, json_black) = state_payloads(session, None).await;
    let payload = match player {
        Player::White => json_white,
        Player::Black => json_black,
    };
    session.send_to(player, &payload).await;
}

pub(super) async fn broadcast_state_after_move(
    session: &Arc<ManagedSession>,
    last_move: Option<LastMoveDto>,
) {
    let (json_white, json_black) = state_payloads(session, last_move).await;
    session.send_to_both(&json_white, &json_black).await;
}

async fn state_payloads(
    session: &Arc<ManagedSession>,
    last_move: Option<LastMoveDto>,
) -> (String, String) {
    let white_connected = session.is_connected(Player::White).await;
    let black_connected = session.is_connected(Player::Black).await;
    let game = session.game.lock().await;

    let turn_expires_in_ms = session.turn_deadline().await.map(|deadline| {
        deadline
            .saturating_duration_since(Instant::now())
            .as_millis() as u64
    });

    let state_for = |your_color: Player| ServerMessage::State {
        board: game.board.clone(),
        current_player: game.current_player,
        remaining_dice: game.remaining_dice.clone(),
        dice: game.last_roll,
        required_moves: game.required_moves,
        move_history_len: game.move_history.len(),
        your_color,
        turn_expires_in_ms,
        white_score: game.white_score,
        black_score: game.black_score,
        round_no: game.round_no,
        last_move,
        opponent_connected: match your_color {
            Player::White => black_connected,
            Player::Black => white_connected,
        },
    };
    let json_white = serde_json::to_string(&state_for(Player::White)).unwrap_or_default();
    let json_black = serde_json::to_string(&state_for(Player::Black)).unwrap_or_default();

    (json_white, json_black)
}

#[cfg(test)]
mod tests {
    use tokio::sync::mpsc;

    use super::*;

    async fn session_with_both_players() -> (
        Arc<ManagedSession>,
        mpsc::UnboundedReceiver<String>,
        mpsc::UnboundedReceiver<String>,
    ) {
        let session = Arc::new(ManagedSession::new(GameSession::new(1, 2, Player::White)));
        let (white_tx, white_rx) = mpsc::unbounded_channel();
        let (black_tx, black_rx) = mpsc::unbounded_channel();
        session.set_sender(Player::White, white_tx).await;
        session.set_sender(Player::Black, black_tx).await;
        (session, white_rx, black_rx)
    }

    fn message_type(payload: &str) -> String {
        let json: serde_json::Value = serde_json::from_str(payload).unwrap();
        json["type"].as_str().unwrap().to_string()
    }

    #[tokio::test]
    async fn a_rejected_action_gets_an_error_followed_by_the_real_state_for_that_player_only() {
        let (session, mut white_rx, mut black_rx) = session_with_both_players().await;

        reject_action(&session, Player::Black, "not_your_turn").await;

        let error = black_rx.try_recv().unwrap();
        let state = black_rx.try_recv().unwrap();
        assert_eq!(message_type(&error), "error");
        assert_eq!(message_type(&state), "state");
        let state: serde_json::Value = serde_json::from_str(&state).unwrap();
        assert_eq!(state["your_color"], "black");
        assert!(black_rx.try_recv().is_err());
        assert!(white_rx.try_recv().is_err());
    }
}
