use serde::Serialize;
use utoipa::ToSchema;

use crate::state::app_state::AppState;

#[derive(Clone, Debug, Serialize, ToSchema)]
pub struct ScoreDto {
    pub wins: u32,
    pub losses: u32,
}

pub async fn handle(state: &AppState, user_id: u64) -> ScoreDto {
    match state.scores_repo.get_by_user_id(user_id).await {
        Some(row) => ScoreDto {
            wins: row.wins,
            losses: row.losses,
        },
        None => ScoreDto { wins: 0, losses: 0 },
    }
}
