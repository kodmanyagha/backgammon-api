use entity::games::GameStatus;
use serde::Serialize;
use serde_json::{json, Value};
use utoipa::ToSchema;

use crate::state::app_state::AppState;

#[derive(Clone, Debug, Serialize, ToSchema)]
pub struct GameHistoryItemDto {
    pub game_id: u64,
    pub opponent_id: u64,
    pub opponent_username: Option<String>,
    pub your_color: &'static str,
    pub status: GameStatus,
    pub result: &'static str,
    pub created_at: chrono::NaiveDateTime,
    pub finished_at: Option<chrono::NaiveDateTime>,
}

pub async fn to_dto(state: &AppState, user_id: u64, game: &entity::games::Model) -> Value {
    let (your_color, opponent_id) = if game.gold_user_id == user_id {
        ("gold", game.purple_user_id)
    } else {
        ("purple", game.gold_user_id)
    };

    let opponent_username = state
        .users_repo
        .get_by_id(opponent_id)
        .await
        .and_then(|opponent| opponent.username);

    let result = match &game.status {
        GameStatus::Finished if game.winner_user_id == Some(user_id) => "win",
        GameStatus::Finished => "loss",
        GameStatus::Abandoned => "abandoned",
        GameStatus::Waiting | GameStatus::Active => "ongoing",
    };

    json!(GameHistoryItemDto {
        game_id: game.id,
        opponent_id,
        opponent_username,
        your_color,
        status: game.status.clone(),
        result,
        created_at: game.created_at,
        finished_at: game.finished_at,
    })
}
