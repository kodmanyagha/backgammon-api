use std::sync::Arc;
use std::time::Duration;

use axum::{
    extract::{ws::WebSocketUpgrade, Path, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    Extension,
};
use entity::games::GameStatus;
use tavla_core::Player;

use self::socket::handle_socket;
use self::watchdog::run_watchdog;
use crate::{
    service::game::{live_store, sessions::ManagedSession},
    state::app_state::AppState,
};

mod actions;
mod ai_offer;
mod broadcast;
mod socket;
mod watchdog;

const PING_INTERVAL: Duration = Duration::from_secs(10);

const PONG_TIMEOUT: Duration = Duration::from_secs(25);

const TURN_TIME_LIMIT: Duration = Duration::from_secs(62);

const NO_LEGAL_MOVES_HOLD: Duration = Duration::from_millis(3300);

const NEXT_ROUND_DELAY: Duration = Duration::from_secs(5);

const WATCHDOG_TICK: Duration = Duration::from_secs(1);

const AI_OFFER_ANSWER_TIMEOUT: Duration = Duration::from_secs(60);

const AI_ROLL_DELAY: Duration = Duration::from_millis(1200);

const AI_AFTER_ROLL_DELAY: Duration = Duration::from_millis(2500);

const AI_MOVE_DELAY: Duration = Duration::from_millis(900);

#[derive(Clone, Copy, Debug)]
enum MatchEnd {
    Won(Player),
    WonLastRound(Player),
    Abandoned,
}

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

    let Some(player) = (if user.id == game.white_user_id {
        Some(Player::White)
    } else if user.id == game.black_user_id {
        Some(Player::Black)
    } else {
        None
    }) else {
        return (StatusCode::FORBIDDEN, "not a participant in this game").into_response();
    };

    if game.status != GameStatus::Active {
        return (StatusCode::BAD_REQUEST, "game is not active").into_response();
    }

    let (session, created) = match open_session(&state, game_id).await {
        Ok(Some(opened)) => opened,
        Ok(None) => return (StatusCode::BAD_REQUEST, "game is not active").into_response(),
        Err(err) => {
            tracing::error!(game_id, error = %err, "live game state could not be loaded");
            return (StatusCode::SERVICE_UNAVAILABLE, "game state unavailable").into_response();
        }
    };

    if created {
        tokio::spawn(run_watchdog(state.clone(), session.clone(), game_id));
    }

    ws.on_upgrade(move |socket| handle_socket(socket, state, session, game_id, player))
}

async fn open_session(
    state: &AppState,
    game_id: u64,
) -> anyhow::Result<Option<(Arc<ManagedSession>, bool)>> {
    if let Some(existing) = state.game_sessions.get(game_id).await {
        return Ok(Some((existing, false)));
    }

    let Some(snapshot) = live_store::load_snapshot(&state.redis_service, game_id).await? else {
        return Ok(None);
    };
    let session = Arc::new(ManagedSession::from_snapshot(snapshot));

    Ok(Some(
        state.game_sessions.insert_if_absent(game_id, session).await,
    ))
}
