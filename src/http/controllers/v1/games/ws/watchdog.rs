use std::sync::Arc;
use std::time::{Duration, Instant};

use tavla_core::Player;

use super::actions::{confirm_turn, finish_match, persist, play_move, roll_dice_and_broadcast};
use super::ai_offer::send_ai_offer;
use super::broadcast::broadcast_state;
use super::{
    MatchEnd, AI_AFTER_ROLL_DELAY, AI_MOVE_DELAY, AI_OFFER_ANSWER_TIMEOUT, AI_ROLL_DELAY,
    TURN_TIME_LIMIT, WATCHDOG_TICK,
};
use crate::{
    service::game::{
        ai_player::{next_ai_action, AiAction},
        expired_turn::{decide_expired_turn, ExpiredTurn},
        live_state::now_unix_ms,
        session::ActionResult,
        sessions::{AiOffer, ManagedSession},
    },
    state::app_state::AppState,
};

pub(super) async fn run_watchdog(state: AppState, session: Arc<ManagedSession>, game_id: u64) {
    loop {
        tokio::time::sleep(next_watchdog_sleep(&session).await).await;
        if !state.game_sessions.contains(game_id).await {
            break;
        }

        let (is_playing, next_round_at_ms, current_player, ai_side) = {
            let game = session.game.lock().await;
            (
                game.is_playing(),
                game.next_round_at_ms(),
                game.current_player,
                game.ai_side,
            )
        };

        if is_playing {
            forget_finished_auto_play(&session, current_player).await;
            let server_plays_this_turn = ai_side == Some(current_player)
                || session.auto_play().await == Some(current_player);

            if server_plays_this_turn {
                play_ai_turn_step(&state, &session, game_id, current_player).await;
            } else {
                watch_turn(&state, &session, game_id, current_player).await;
            }
        } else if next_round_at_ms.is_some_and(|at_ms| now_unix_ms() >= at_ms) {
            start_next_round(&state, &session, game_id).await;
        }
    }
}

async fn forget_finished_auto_play(session: &Arc<ManagedSession>, current_player: Player) {
    if session
        .auto_play()
        .await
        .is_some_and(|auto_player| auto_player != current_player)
    {
        session.set_auto_play(None).await;
    }
}

async fn next_watchdog_sleep(session: &Arc<ManagedSession>) -> Duration {
    let mut sleep_ms = WATCHDOG_TICK.as_millis() as i64;

    let (next_round_at_ms, playing_current_player, ai_side) = {
        let game = session.game.lock().await;
        (
            game.next_round_at_ms(),
            game.is_playing().then_some(game.current_player),
            game.ai_side,
        )
    };
    if let Some(next_round_at_ms) = next_round_at_ms {
        sleep_ms = sleep_ms.min(next_round_at_ms - now_unix_ms());
    }
    if let Some(current_player) = playing_current_player {
        let server_plays_this_turn =
            ai_side == Some(current_player) || session.auto_play().await == Some(current_player);
        if let (true, Some(next_ai_action_at)) =
            (server_plays_this_turn, session.ai_next_action_at().await)
        {
            sleep_ms = sleep_ms.min(
                next_ai_action_at
                    .saturating_duration_since(Instant::now())
                    .as_millis() as i64,
            );
        }
    }

    Duration::from_millis(sleep_ms.max(10) as u64)
}

async fn watch_turn(
    state: &AppState,
    session: &Arc<ManagedSession>,
    game_id: u64,
    current_player: Player,
) {
    let now = Instant::now();

    let deadline = match session.turn_deadline().await {
        Some(deadline) => deadline,
        None => {
            let deadline = now + TURN_TIME_LIMIT;
            session.set_turn_deadline(deadline).await;
            persist(state, game_id, session, &[]).await;
            deadline
        }
    };
    if now < deadline {
        return;
    }

    settle_expired_turn(state, session, game_id, current_player).await;
}

async fn settle_expired_turn(
    state: &AppState,
    session: &Arc<ManagedSession>,
    game_id: u64,
    current_player: Player,
) {
    let opponent = current_player.opponent();
    let current_connected = session.is_connected(current_player).await;
    let opponent_connected = session.is_connected(opponent).await;
    let opponent_is_ai = session.game.lock().await.ai_side == Some(opponent);

    match decide_expired_turn(
        current_player,
        current_connected,
        opponent_connected,
        opponent_is_ai,
    ) {
        ExpiredTurn::AutoPlay => start_auto_play(session, current_player).await,
        ExpiredTurn::AskToContinueWithAi { present, .. } => {
            ask_present_player(state, session, game_id, present).await
        }
        ExpiredTurn::NobodyPresent => {
            let gone_long_enough = |gone_for: Option<Duration>| {
                gone_for.is_some_and(|gone_for| gone_for >= TURN_TIME_LIMIT)
            };
            let current_gone = gone_long_enough(session.disconnected_for(current_player).await);
            let opponent_gone =
                opponent_is_ai || gone_long_enough(session.disconnected_for(opponent).await);
            if current_gone && opponent_gone {
                finish_match(state, session, game_id, MatchEnd::Abandoned, Vec::new()).await;
            }
        }
    }
}

async fn start_auto_play(session: &Arc<ManagedSession>, player: Player) {
    session.set_auto_play(Some(player)).await;
    session.set_ai_next_action_at(None).await;
}

async fn ask_present_player(
    state: &AppState,
    session: &Arc<ManagedSession>,
    game_id: u64,
    present: Player,
) {
    match session.ai_offer().await {
        None => {
            session
                .set_ai_offer(Some(AiOffer {
                    to: present,
                    sent_at: Instant::now(),
                }))
                .await;
            send_ai_offer(session, present).await;
        }
        Some(offer) if offer.sent_at.elapsed() >= AI_OFFER_ANSWER_TIMEOUT => {
            finish_match(state, session, game_id, MatchEnd::Won(present), Vec::new()).await;
        }
        Some(_) => {}
    }
}

async fn start_next_round(state: &AppState, session: &Arc<ManagedSession>, game_id: u64) {
    session.game.lock().await.start_next_round();
    session.clear_turn_deadline().await;
    session.set_ai_next_action_at(None).await;
    persist(state, game_id, session, &[]).await;
    broadcast_state(session).await;
}

async fn play_ai_turn_step(
    state: &AppState,
    session: &Arc<ManagedSession>,
    game_id: u64,
    ai_player: Player,
) {
    let action = next_ai_action(&*session.game.lock().await, ai_player);
    if action == AiAction::Wait {
        return;
    }

    let Some(due_at) = session.ai_next_action_at().await else {
        let think_time = if action == AiAction::Roll {
            AI_ROLL_DELAY
        } else {
            AI_MOVE_DELAY
        };
        session
            .set_ai_next_action_at(Some(Instant::now() + think_time))
            .await;
        return;
    };
    if Instant::now() < due_at {
        return;
    }

    let next_delay = match action {
        AiAction::Roll => {
            if let Some(hold) = session.roll_hold_remaining().await {
                session
                    .set_ai_next_action_at(Some(Instant::now() + hold))
                    .await;
                return;
            }
            roll_dice_and_broadcast(state, session, game_id, ai_player).await;
            let turn_passed_without_a_move = session.game.lock().await.current_player != ai_player;
            (!turn_passed_without_a_move).then_some(AI_AFTER_ROLL_DELAY)
        }
        AiAction::Move(planned) => {
            play_move(
                state,
                session,
                game_id,
                ai_player,
                planned.origin,
                planned.die,
                true,
            )
            .await;
            Some(AI_MOVE_DELAY)
        }
        AiAction::Undo => {
            let result = session.game.lock().await.undo(ai_player);
            if matches!(result, ActionResult::Ok) {
                persist(state, game_id, session, &[]).await;
                broadcast_state(session).await;
            }
            Some(AI_MOVE_DELAY)
        }
        AiAction::Confirm => {
            confirm_turn(state, session, game_id, ai_player).await;
            session.set_auto_play(None).await;
            None
        }
        AiAction::Wait => return,
    };
    session
        .set_ai_next_action_at(next_delay.map(|delay| Instant::now() + delay))
        .await;
}
