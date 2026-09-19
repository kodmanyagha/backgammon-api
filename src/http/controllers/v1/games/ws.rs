use std::sync::Arc;
use std::time::Duration;

use axum::{
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        Path, State,
    },
    http::StatusCode,
    response::{IntoResponse, Response},
    Extension,
};
use entity::games::GameStatus;
use futures_util::{SinkExt, StreamExt};
use tavla_core::Player;
use tokio::sync::mpsc;
use tokio::time::{interval, Instant, MissedTickBehavior};

use crate::{
    service::game::{
        session::ActionResult,
        sessions::ManagedSession,
        ws_protocol::{ClientMessage, ServerMessage},
    },
    state::app_state::AppState,
};

const PING_INTERVAL: Duration = Duration::from_secs(10);
const PONG_TIMEOUT: Duration = Duration::from_secs(25);

/// Zar atıldıktan sonra sırası gelen oyuncunun hamlesini BİTİRMESİ
/// (`ConfirmTurn`) için tanınan süre — bağlı/kopuk olması ÖNEMLİ DEĞİL, süre
/// dolduğunda o oyuncu hükmen kaybeder (bkz. `run_turn_watchdog`). İstemci
/// tarafında ~2 saniyelik zar animasyonu bu sürenin İÇİNDE sayılır (istemci
/// kullanıcıya efektif ~60sn kalır).
const TURN_TIME_LIMIT: Duration = Duration::from_secs(62);

const NO_LEGAL_MOVES_HOLD: Duration = Duration::from_millis(3300);
const WATCHDOG_TICK: Duration = Duration::from_secs(1);

#[utoipa::path(
    get,
    path = "/v1/games/{id}/ws",
    tag = "Games",
    operation_id = "games_get_ws",
    params(("id" = u64, Path, description = "Game id")),
    responses((status = 101, description = "Switching protocols to WebSocket")),
    security(("bearer_auth" = [])),
)]
pub async fn handle_upgrade(
    State(state): State<AppState>,
    Extension(user): Extension<entity::users::Model>,
    Path(game_id): Path<u64>,
    ws: WebSocketUpgrade,
) -> Response {
    let Some(game) = state.games_repo.get_by_id(game_id).await else {
        return (StatusCode::NOT_FOUND, "game not found").into_response();
    };

    let Some(player) = (if user.id == game.gold_user_id {
        Some(Player::Black)
    } else if user.id == game.purple_user_id {
        Some(Player::White)
    } else {
        None
    }) else {
        return (StatusCode::FORBIDDEN, "not a participant in this game").into_response();
    };

    if game.status != GameStatus::Active {
        return (StatusCode::BAD_REQUEST, "game is not active").into_response();
    }

    let (session, created) = state
        .game_sessions
        .get_or_create(game_id, game.gold_user_id, game.purple_user_id)
        .await;

    if created {
        tokio::spawn(run_turn_watchdog(state.clone(), session.clone(), game_id));
    }

    ws.on_upgrade(move |socket| handle_socket(socket, state, session, game_id, player))
}

#[derive(Debug)]
enum SocketExit {
    ClientClosed,
    ReadError,
    SendFailed,
    PongTimeout,
}

/// Bağlantı boyunca tek bir `select!` döngüsü: giden mesajları (`rx`),
/// istemciden gelenleri (`ws_receiver`) VE düzenli ping/pong nabzını
/// (`ping_ticker`) aynı anda bekler. `PONG_TIMEOUT` içinde bir pong
/// gelmezse (ör. istemci ağı tamamen kesildi, TCP FIN/RST hiç ulaşmadı)
/// bağlantı KOPMUŞ sayılır — sadece sağlıksız bir TCP soketinin sonsuza
/// kadar açık kalmasını beklemek yerine.
///
/// Döngü hangi sebeple biterse bitsin, bu BAŞLI BAŞINA oyunu bitirmez —
/// bkz. `run_turn_watchdog`: kazanan/kaybeden kararı artık SADECE tur
/// saatine bağlı (bağlı olsun ya da olmasın, sırası gelen oyuncu süresi
/// içinde oynamazsa kaybeder). Burada sadece iki istisna var: (1) rakip de
/// zaten bağlı değilse maç kimseyi kazandırmadan terk edilmiş sayılır
/// (`abandon_game`), (2) `Resign` (bkz. `handle_client_message`) hâlâ anında biter.
async fn handle_socket(
    socket: WebSocket,
    state: AppState,
    session: Arc<ManagedSession>,
    game_id: u64,
    player: Player,
) {
    let (mut ws_sender, mut ws_receiver) = socket.split();
    let (tx, mut rx) = mpsc::unbounded_channel::<String>();

    session.set_sender(player, tx).await;

    tracing::info!(game_id, ?player, "ws_connected");
    notify_presence(&session, player, ServerMessage::OpponentConnected).await;
    broadcast_state(&session).await;

    let mut ping_ticker = interval(PING_INTERVAL);
    ping_ticker.set_missed_tick_behavior(MissedTickBehavior::Delay);
    let mut last_pong = Instant::now();

    let exit_reason = loop {
        tokio::select! {
            outgoing = rx.recv() => {
                let Some(text) = outgoing else { break SocketExit::ClientClosed };
                tracing::info!(game_id, ?player, message = %text, "ws_send");
                if ws_sender.send(Message::Text(text.into())).await.is_err() {
                    break SocketExit::SendFailed;
                }
            }
            incoming = ws_receiver.next() => {
                match incoming {
                    Some(Ok(Message::Text(text))) => {
                        tracing::info!(game_id, ?player, message = %text, "ws_recv");
                        handle_client_message(&state, &session, game_id, player, &text).await;
                    }
                    Some(Ok(Message::Pong(_))) => {
                        last_pong = Instant::now();
                    }
                    Some(Ok(Message::Close(_))) => break SocketExit::ClientClosed,
                    Some(Ok(_)) => {}
                    Some(Err(_)) => break SocketExit::ReadError,
                    None => break SocketExit::ClientClosed,
                }
            }
            _ = ping_ticker.tick() => {
                if last_pong.elapsed() > PONG_TIMEOUT {
                    break SocketExit::PongTimeout;
                }
                if ws_sender.send(Message::Ping(Vec::new().into())).await.is_err() {
                    break SocketExit::SendFailed;
                }
            }
        }
    };

    tracing::warn!(game_id, ?player, ?exit_reason, "ws_disconnected");

    session.clear_sender(player).await;
    notify_presence(&session, player, ServerMessage::OpponentDisconnected).await;
    if !session.is_connected(player.opponent()).await {
        abandon_game(&state, game_id).await;
    }
}

/// `game_id` için oturumu YENİ oluşturan bağlantının başlattığı, o oturumun
/// ömrü boyunca yaşayan bekçi görev: her `WATCHDOG_TICK`'te bir tur süresi
/// dolmuş mu diye bakar. Oyun başka bir yoldan zaten bitip haritadan
/// silinmişse (`GameSessions::contains` `false` döner) kendini sonlandırır.
async fn run_turn_watchdog(state: AppState, session: Arc<ManagedSession>, game_id: u64) {
    loop {
        tokio::time::sleep(WATCHDOG_TICK).await;
        if !state.game_sessions.contains(game_id).await {
            break;
        }
        let Some(deadline) = session.turn_deadline().await else { continue };
        if std::time::Instant::now() < deadline {
            continue;
        }
        let loser = session.game.lock().await.current_player;
        finish_game(&state, &session, game_id, loser.opponent()).await;
        break;
    }
}

/// Her iki oyuncu da bağlı değilken tetiklenir (bkz. `handle_socket`'in
/// bağlantı kopma temizliği): kimse kazanmaz, kimseye bildirim gitmez
/// (zaten dinleyen yok) — maç sadece `Abandoned` olarak işaretlenip bellekten
/// silinir. `state.game_sessions.remove` ATOMIK guard'ı burada da geçerli:
/// bu çağrı ile eşzamanlı bir `finish_game` yarışırsa sadece biri ilerler.
async fn abandon_game(state: &AppState, game_id: u64) {
    if !state.game_sessions.remove(game_id).await {
        return;
    }
    let _ = state.games_repo.abandon(game_id).await;
}

async fn handle_client_message(
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

    match message {
        ClientMessage::RollDice => {
            if let Some(wait) = session.roll_hold_remaining().await {
                let session = session.clone();
                tokio::spawn(async move {
                    tokio::time::sleep(wait).await;
                    roll_dice_and_broadcast(&session, player).await;
                });
                return;
            }
            roll_dice_and_broadcast(session, player).await;
        }
        ClientMessage::MakeMove { origin, die } => {
            let result = session
                .game
                .lock()
                .await
                .make_move(player, origin.into(), die);

            match result {
                ActionResult::MoveApplied {
                    die,
                    origin_point,
                    sequence_no,
                } => {
                    persist_move(state, game_id, player, sequence_no, origin_point, die).await;
                    broadcast_state(session).await;
                }
                ActionResult::GameOver {
                    winner,
                    die,
                    origin_point,
                    sequence_no,
                } => {
                    persist_move(state, game_id, player, sequence_no, origin_point, die).await;
                    broadcast_state(session).await;
                    finish_game(state, session, game_id, winner).await;
                }
                ActionResult::Err(reason) => send_error(session, player, reason).await,
                ActionResult::Ok | ActionResult::Rolled { .. } => {}
            }
        }
        ClientMessage::Undo => {
            let result = session.game.lock().await.undo(player);
            apply_simple_result(session, player, result).await;
        }
        ClientMessage::ConfirmTurn => {
            let result = session.game.lock().await.confirm_turn(player);
            if matches!(result, ActionResult::Ok) {
                session.clear_turn_deadline().await;
            }
            apply_simple_result(session, player, result).await;
        }
        ClientMessage::Resign => {
            let winner = player.opponent();
            finish_game(state, session, game_id, winner).await;
        }
        ClientMessage::Emoji { emoji } => {
            let payload = serde_json::to_string(&ServerMessage::Emoji { from: player, emoji })
                .unwrap_or_default();
            session.send_to(player.opponent(), &payload).await;
        }
    }
}

/// Rolls the dice and broadcasts `DiceRolled` followed by the fresh `State` to both players.
///
/// A roll with no legal move hands the turn over immediately, so no turn deadline is set and
/// the opponent's next roll is held back for `NO_LEGAL_MOVES_HOLD`, long enough for both
/// clients to finish the dice animation and read the "no move" notice.
/// The wait itself runs in a spawned task so the socket loop keeps serving pings.
async fn roll_dice_and_broadcast(session: &Arc<ManagedSession>, player: Player) {
    let result = session.game.lock().await.roll_dice(player);
    if let ActionResult::Rolled { die1, die2, no_legal_moves } = result {
        if no_legal_moves {
            session.hold_next_roll_for(NO_LEGAL_MOVES_HOLD).await;
        } else {
            session.set_turn_deadline(std::time::Instant::now() + TURN_TIME_LIMIT).await;
        }
        let payload = serde_json::to_string(&ServerMessage::DiceRolled { die1, die2, no_legal_moves }).unwrap_or_default();
        session.send_to_both(&payload, &payload).await;
    }
    apply_simple_result(session, player, result).await;
}

async fn apply_simple_result(session: &Arc<ManagedSession>, player: Player, result: ActionResult) {
    match result {
        ActionResult::Ok | ActionResult::Rolled { .. } => broadcast_state(session).await,
        ActionResult::Err(reason) => send_error(session, player, reason).await,
        ActionResult::MoveApplied { .. } | ActionResult::GameOver { .. } => {}
    }
}

async fn persist_move(
    state: &AppState,
    game_id: u64,
    player: Player,
    sequence_no: u32,
    origin_point: Option<u8>,
    die: u8,
) {
    // `GameMovePlayer` (DB'deki MySQL ENUM) kasıtlı olarak "gold"/"purple"
    // kalıyor — şema değişikliği/migration riskini önlemek için sadece
    // motor/istemci tarafındaki `tavla_core::Player` yeniden adlandırıldı.
    let move_player = match player {
        Player::Black => entity::game_moves::GameMovePlayer::Gold,
        Player::White => entity::game_moves::GameMovePlayer::Purple,
    };
    let _ = state
        .game_moves_repo
        .append(game_id, sequence_no, move_player, origin_point, die)
        .await;
}

/// `state.game_sessions.remove` bu oyunu bitirme hakkını atomik olarak
/// kazanmak için EN BAŞTA çağrılır: bir kazanma hamlesi ile bir bağlantı
/// kopması (ya da iki oyuncunun bağlantısının neredeyse aynı anda kopması,
/// bkz. `handle_socket`) yarışırsa, sadece haritadan GERÇEKTEN silen çağrı
/// ilerler — diğeri `false` alıp hemen döner, skor/DB satırı iki kez işlenmez.
async fn finish_game(state: &AppState, session: &Arc<ManagedSession>, game_id: u64, winner: Player) {
    if !state.game_sessions.remove(game_id).await {
        return;
    }

    let (winner_user_id, loser_user_id) = {
        let game = session.game.lock().await;
        (
            game.user_id_for(winner),
            game.user_id_for(winner.opponent()),
        )
    };

    let _ = state.games_repo.finish(game_id, winner_user_id).await;
    let _ = state.scores_repo.record_win(winner_user_id).await;
    let _ = state.scores_repo.record_loss(loser_user_id).await;

    let payload = serde_json::to_string(&ServerMessage::GameOver { winner }).unwrap_or_default();
    session.send_to_both(&payload, &payload).await;
}

async fn notify_presence(session: &Arc<ManagedSession>, player: Player, message: ServerMessage) {
    let payload = serde_json::to_string(&message).unwrap_or_default();
    session.send_to(player.opponent(), &payload).await;
}

async fn send_error(session: &Arc<ManagedSession>, player: Player, reason: &str) {
    let payload = serde_json::to_string(&ServerMessage::Error {
        message: reason.to_string(),
    })
    .unwrap_or_default();
    session.send_to(player, &payload).await;
}

async fn broadcast_state(session: &Arc<ManagedSession>) {
    let game = session.game.lock().await;

    let turn_expires_in_ms = session
        .turn_deadline()
        .await
        .map(|deadline| deadline.saturating_duration_since(std::time::Instant::now()).as_millis() as u64);

    let for_black = ServerMessage::State {
        board: game.board.clone(),
        current_player: game.current_player,
        remaining_dice: game.remaining_dice.clone(),
        required_moves: game.required_moves,
        move_history_len: game.move_history.len(),
        your_color: Player::Black,
        turn_expires_in_ms,
    };
    let for_white = ServerMessage::State {
        board: game.board.clone(),
        current_player: game.current_player,
        remaining_dice: game.remaining_dice.clone(),
        required_moves: game.required_moves,
        move_history_len: game.move_history.len(),
        your_color: Player::White,
        turn_expires_in_ms,
    };

    drop(game);

    let json_black = serde_json::to_string(&for_black).unwrap_or_default();
    let json_white = serde_json::to_string(&for_white).unwrap_or_default();
    session.send_to_both(&json_black, &json_white).await;
}
