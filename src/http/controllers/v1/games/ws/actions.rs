use std::sync::Arc;
use std::time::Instant;

use tavla_core::{Origin, Player};

use super::ai_offer::{accept_ai, decline_ai};
use super::broadcast::{
    broadcast_game_ended, broadcast_state, broadcast_state_after_move, reject_action, send_error,
};
use super::{MatchEnd, NEXT_ROUND_DELAY, NO_LEGAL_MOVES_HOLD, TURN_TIME_LIMIT};
use crate::{
    service::game::{
        live_state::now_unix_ms,
        live_store,
        outbox::OutboxEvent,
        session::{ActionResult, RoundOutcome},
        sessions::ManagedSession,
        ws_protocol::{ClientMessage, LastMoveDto, ServerMessage},
    },
    state::app_state::AppState,
};

pub(super) async fn handle_client_message(
    state: &AppState,
    session: &Arc<ManagedSession>,
    game_id: u64,
    player: Player,
    text: &str,
) {
    let Ok(message) = serde_json::from_str::<ClientMessage>(text) else {
        send_error(session, player, "bad_request").await;
        return;
    };

    let is_turn_action = matches!(
        message,
        ClientMessage::RollDice
            | ClientMessage::MakeMove { .. }
            | ClientMessage::Undo
            | ClientMessage::ConfirmTurn
    );
    if is_turn_action && session.auto_play().await == Some(player) {
        reject_action(session, player, "auto_play_active").await;
        return;
    }

    match message {
        ClientMessage::RollDice => {
            if let Some(wait) = session.roll_hold_remaining().await {
                let state = state.clone();
                let session = session.clone();
                tokio::spawn(async move {
                    tokio::time::sleep(wait).await;
                    roll_dice_and_broadcast(&state, &session, game_id, player).await;
                });
                return;
            }
            roll_dice_and_broadcast(state, session, game_id, player).await;
        }
        ClientMessage::MakeMove { origin, die } => {
            play_move(state, session, game_id, player, origin.into(), die, false).await;
        }
        ClientMessage::Undo => {
            let result = session.game.lock().await.undo(player);
            if matches!(result, ActionResult::Ok) {
                persist(state, game_id, session, &[]).await;
            }
            apply_simple_result(session, player, result).await;
        }
        ClientMessage::ConfirmTurn => {
            confirm_turn(state, session, game_id, player).await;
        }
        ClientMessage::Resign => {
            finish_match(
                state,
                session,
                game_id,
                MatchEnd::Won(player.opponent()),
                Vec::new(),
            )
            .await;
        }
        ClientMessage::Emoji { emoji } => {
            let payload = serde_json::to_string(&ServerMessage::Emoji {
                from: player,
                emoji,
            })
            .unwrap_or_default();
            session.send_to(player.opponent(), &payload).await;
        }
        ClientMessage::AcceptAi => accept_ai(state, session, game_id, player).await,
        ClientMessage::DeclineAi => decline_ai(state, session, game_id, player).await,
    }
}

pub(super) async fn play_move(
    state: &AppState,
    session: &Arc<ManagedSession>,
    game_id: u64,
    player: Player,
    origin: Origin,
    die: u8,
    is_ai: bool,
) {
    let result = session.game.lock().await.make_move(player, origin, die);
    let last_move = LastMoveDto {
        player,
        origin: origin.into(),
        die,
        is_ai,
    };

    match result {
        ActionResult::MoveApplied {
            die,
            origin_point,
            sequence_no,
            round_no,
        } => {
            let event = move_event(
                game_id,
                round_no,
                sequence_no,
                player,
                origin_point,
                die,
                is_ai,
            );
            persist(state, game_id, session, &[event]).await;
            broadcast_state_after_move(session, Some(last_move)).await;
        }
        ActionResult::RoundWon {
            winner,
            die,
            origin_point,
            sequence_no,
            round_no,
        } => {
            let event = move_event(
                game_id,
                round_no,
                sequence_no,
                player,
                origin_point,
                die,
                is_ai,
            );
            finish_round(state, session, game_id, winner, event, last_move).await;
        }
        ActionResult::Err(reason) => reject_action(session, player, reason).await,
        ActionResult::Ok | ActionResult::Rolled { .. } => {}
    }
}

pub(super) async fn confirm_turn(
    state: &AppState,
    session: &Arc<ManagedSession>,
    game_id: u64,
    player: Player,
) {
    let result = session.game.lock().await.confirm_turn(player);
    if matches!(result, ActionResult::Ok) {
        session.clear_turn_deadline().await;
        persist(state, game_id, session, &[]).await;
    }
    apply_simple_result(session, player, result).await;
}

pub(super) async fn roll_dice_and_broadcast(
    state: &AppState,
    session: &Arc<ManagedSession>,
    game_id: u64,
    player: Player,
) {
    let result = session.game.lock().await.roll_dice(player);
    if let ActionResult::Rolled {
        die1,
        die2,
        no_legal_moves,
    } = result
    {
        if no_legal_moves {
            session.hold_next_roll_for(NO_LEGAL_MOVES_HOLD).await;
        } else {
            session
                .set_turn_deadline(Instant::now() + TURN_TIME_LIMIT)
                .await;
        }
        persist(state, game_id, session, &[]).await;
        let payload = serde_json::to_string(&ServerMessage::DiceRolled {
            die1,
            die2,
            no_legal_moves,
        })
        .unwrap_or_default();
        session.send_to_both(&payload, &payload).await;
    }
    apply_simple_result(session, player, result).await;
}

async fn apply_simple_result(session: &Arc<ManagedSession>, player: Player, result: ActionResult) {
    match result {
        ActionResult::Ok | ActionResult::Rolled { .. } => broadcast_state(session).await,
        ActionResult::Err(reason) => reject_action(session, player, reason).await,
        ActionResult::MoveApplied { .. } | ActionResult::RoundWon { .. } => {}
    }
}

fn move_event(
    game_id: u64,
    round_no: u16,
    sequence_no: u32,
    player: Player,
    origin_point: Option<u8>,
    die: u8,
    is_ai: bool,
) -> OutboxEvent {
    OutboxEvent::Move {
        game_id,
        round_no,
        sequence_no,
        player,
        origin_point,
        die,
        is_ai,
        at_ms: now_unix_ms(),
    }
}

pub(super) async fn persist(
    state: &AppState,
    game_id: u64,
    session: &Arc<ManagedSession>,
    events: &[OutboxEvent],
) {
    let _serialized = session.lock_persist().await;
    if !state.game_sessions.contains(game_id).await {
        return;
    }

    let snapshot = session.snapshot().await;
    if let Err(err) =
        live_store::save_snapshot(&state.redis_service, game_id, &snapshot, events).await
    {
        tracing::error!(game_id, error = %err, "live game state could not be saved");
    }
}

async fn finish_round(
    state: &AppState,
    session: &Arc<ManagedSession>,
    game_id: u64,
    winner: Player,
    winning_move: OutboxEvent,
    last_move: LastMoveDto,
) {
    let now_ms = now_unix_ms();
    let (outcome, round_event) = {
        let mut game = session.game.lock().await;
        let round_no = game.round_no;
        let outcome = game.finish_round(winner, now_ms + NEXT_ROUND_DELAY.as_millis() as i64);
        let round_event = OutboxEvent::RoundWon {
            game_id,
            round_no,
            winner_user_id: game.user_id_for(winner),
            white_score: game.white_score,
            black_score: game.black_score,
            at_ms: now_ms,
        };
        (outcome, round_event)
    };
    session.clear_turn_deadline().await;
    let events = vec![winning_move, round_event];

    match outcome {
        RoundOutcome::MatchContinues => {
            persist(state, game_id, session, &events).await;
            broadcast_state_after_move(session, Some(last_move)).await;
            broadcast_game_ended(session).await;
        }
        RoundOutcome::MatchWon(match_winner) => {
            broadcast_state_after_move(session, Some(last_move)).await;
            finish_match(state, session, game_id, MatchEnd::WonLastRound(match_winner), events).await;
        }
    }
}

pub(super) async fn finish_match(
    state: &AppState,
    session: &Arc<ManagedSession>,
    game_id: u64,
    end: MatchEnd,
    mut pending_events: Vec<OutboxEvent>,
) {
    let serialized = session.lock_persist().await;
    if !state.game_sessions.remove(game_id).await {
        return;
    }

    let (white_score, black_score, white_user_id, black_user_id, ranked, last_round_mars) = {
        let game = session.game.lock().await;
        (
            game.white_score,
            game.black_score,
            game.white_user_id,
            game.black_user_id,
            !game.ai_used,
            game.last_round_mars,
        )
    };
    let at_ms = now_unix_ms();

    pending_events.push(match end {
        MatchEnd::Won(winner) | MatchEnd::WonLastRound(winner) => {
            let (winner_user_id, loser_user_id) = match winner {
                Player::White => (white_user_id, black_user_id),
                Player::Black => (black_user_id, white_user_id),
            };
            OutboxEvent::MatchFinished {
                game_id,
                winner_user_id,
                loser_user_id,
                white_score,
                black_score,
                at_ms,
                ranked,
            }
        }
        MatchEnd::Abandoned => OutboxEvent::MatchAbandoned {
            game_id,
            white_score,
            black_score,
            at_ms,
        },
    });

    if let Err(err) = live_store::close_game(&state.redis_service, game_id, &pending_events).await {
        tracing::error!(game_id, error = %err, "finished match could not be queued for the database");
    }
    drop(serialized);

    let announced = match end {
        MatchEnd::Won(winner) => Some((winner, false)),
        MatchEnd::WonLastRound(winner) => Some((winner, last_round_mars)),
        MatchEnd::Abandoned => None,
    };
    if let Some((winner, mars)) = announced {
        let payload = serde_json::to_string(&ServerMessage::GameOver {
            winner,
            white_score,
            black_score,
            mars,
        })
        .unwrap_or_default();
        session.send_to_both(&payload, &payload).await;
    }
}
