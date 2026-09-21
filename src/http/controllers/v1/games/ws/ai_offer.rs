use std::sync::Arc;
use std::time::Instant;

use tavla_core::Player;

use super::actions::{finish_match, persist};
use super::broadcast::{broadcast_state, send_error};
use super::{MatchEnd, TURN_TIME_LIMIT};
use crate::{
    service::game::{
        sessions::{AiOffer, ManagedSession},
        ws_protocol::ServerMessage,
    },
    state::app_state::AppState,
};

pub(super) async fn send_ai_offer(session: &Arc<ManagedSession>, player: Player) {
    let payload = serde_json::to_string(&ServerMessage::AiOffer).unwrap_or_default();
    session.send_to(player, &payload).await;
}

pub(super) async fn withdraw_ai_offer_after_return(
    state: &AppState,
    session: &Arc<ManagedSession>,
    game_id: u64,
    player: Player,
) {
    let offer_was_open_for_the_opponent = session
        .ai_offer()
        .await
        .is_some_and(|offer| offer.to == player.opponent());
    if !offer_was_open_for_the_opponent {
        return;
    }

    session.set_ai_offer(None).await;
    session
        .set_turn_deadline(Instant::now() + TURN_TIME_LIMIT)
        .await;
    persist(state, game_id, session, &[]).await;
}

pub(super) async fn resend_pending_ai_offer(session: &Arc<ManagedSession>, player: Player) {
    let Some(offer) = session.ai_offer().await else {
        return;
    };

    if offer.to == player {
        session
            .set_ai_offer(Some(AiOffer {
                sent_at: Instant::now(),
                ..offer
            }))
            .await;
        send_ai_offer(session, player).await;
    }
}

pub(super) async fn take_back_seat_from_ai(
    state: &AppState,
    session: &Arc<ManagedSession>,
    game_id: u64,
    player: Player,
) {
    let seat_was_played_by_ai = {
        let mut game = session.game.lock().await;
        let was_ai = game.ai_side == Some(player);
        if was_ai {
            game.ai_side = None;
        }
        was_ai
    };

    if seat_was_played_by_ai {
        session.set_ai_next_action_at(None).await;
        persist(state, game_id, session, &[]).await;
    }
}

pub(super) async fn accept_ai(
    state: &AppState,
    session: &Arc<ManagedSession>,
    game_id: u64,
    player: Player,
) {
    if !has_open_ai_offer(session, player).await {
        send_error(session, player, "no_ai_offer").await;
        return;
    }
    if session.is_connected(player.opponent()).await {
        send_error(session, player, "opponent_present").await;
        return;
    }

    let ai_took_over_the_clock = {
        let mut game = session.game.lock().await;
        if game.ai_side.is_some() {
            return;
        }
        game.hand_side_to_ai(player.opponent());
        game.current_player == player.opponent()
    };

    session.set_ai_offer(None).await;
    session.set_ai_next_action_at(None).await;
    if ai_took_over_the_clock {
        session.clear_turn_deadline().await;
    } else {
        session
            .set_turn_deadline(Instant::now() + TURN_TIME_LIMIT)
            .await;
    }
    persist(state, game_id, session, &[]).await;
    broadcast_state(session).await;
}

pub(super) async fn decline_ai(
    state: &AppState,
    session: &Arc<ManagedSession>,
    game_id: u64,
    player: Player,
) {
    if !has_open_ai_offer(session, player).await {
        send_error(session, player, "no_ai_offer").await;
        return;
    }
    if session.is_connected(player.opponent()).await {
        send_error(session, player, "opponent_present").await;
        return;
    }

    finish_match(state, session, game_id, MatchEnd::Won(player), Vec::new()).await;
}

async fn has_open_ai_offer(session: &Arc<ManagedSession>, player: Player) -> bool {
    session
        .ai_offer()
        .await
        .is_some_and(|offer| offer.to == player)
}
