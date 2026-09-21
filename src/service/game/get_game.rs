use crate::{state::app_state::AppState, utils::consts::errors};

pub async fn handle(
    state: &AppState,
    game_id: u64,
    requester_id: u64,
) -> anyhow::Result<entity::games::Model> {
    let game = state
        .games_repo
        .get_by_id(game_id)
        .await
        .ok_or_else(|| anyhow::anyhow!(errors::GAME_NOT_FOUND))?;

    if game.white_user_id != requester_id && game.black_user_id != requester_id {
        return Err(anyhow::anyhow!(errors::UNAUTHORIZED));
    }

    Ok(game)
}
