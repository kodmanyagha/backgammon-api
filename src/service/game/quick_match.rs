use entity::games::GameStatus;
use serde::Serialize;
use utoipa::ToSchema;

use crate::{service::game::matchmaking_queue::QueueEvent, state::app_state::AppState};

#[derive(Clone, Debug, Serialize, ToSchema)]
#[serde(tag = "status", rename_all = "snake_case")]
pub enum QuickMatchResultDto {
    Waiting,
    Matched { game_id: u64 },
}

pub async fn handle(state: &AppState, user_id: u64) -> anyhow::Result<QuickMatchResultDto> {
    if let Some(existing) = state
        .games_repo
        .get_for_user(user_id, 1)
        .await?
        .into_iter()
        .find(|game| game.status == GameStatus::Active)
    {
        return Ok(QuickMatchResultDto::Matched {
            game_id: existing.id,
        });
    }

    match state.matchmaking.advance(user_id).await {
        QueueEvent::AlreadyMatched(game_id) => Ok(QuickMatchResultDto::Matched { game_id }),
        QueueEvent::NowWaiting => Ok(QuickMatchResultDto::Waiting),
        QueueEvent::OpponentFound(opponent_id) => {
            let game = state.games_repo.create(opponent_id, user_id).await?;
            state.matchmaking.record_match(opponent_id, game.id).await;

            Ok(QuickMatchResultDto::Matched { game_id: game.id })
        }
    }
}

pub async fn cancel(state: &AppState, user_id: u64) {
    state.matchmaking.leave(user_id).await;
}
