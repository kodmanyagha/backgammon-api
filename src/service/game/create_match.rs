use crate::{
    service::game::{
        live_state::{now_unix_ms, LiveSnapshot},
        live_store,
        session::{random_first_player, GameSession},
    },
    state::app_state::AppState,
};

pub async fn create(
    state: &AppState,
    white_user_id: u64,
    black_user_id: u64,
) -> anyhow::Result<entity::games::Model> {
    let game = state.games_repo.create(white_user_id, black_user_id).await?;

    let snapshot = LiveSnapshot {
        session: GameSession::new(white_user_id, black_user_id, random_first_player()),
        turn_deadline_at_ms: None,
        saved_at_ms: now_unix_ms(),
    };
    live_store::save_snapshot(&state.redis_service, game.id, &snapshot, &[]).await?;

    Ok(game)
}
