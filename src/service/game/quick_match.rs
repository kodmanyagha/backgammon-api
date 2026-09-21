use serde::Serialize;
use utoipa::ToSchema;

use crate::{
    service::game::{create_match, matchmaking_queue::QueueEvent},
    state::app_state::AppState,
};

#[derive(Clone, Debug, Serialize, ToSchema)]
#[serde(tag = "status", rename_all = "snake_case")]
pub enum QuickMatchResultDto {
    Waiting,
    Matched { game_id: u64, opponent_name: String },
    NoOpponent,
}

const UNKNOWN_OPPONENT_NAME: &str = "Rakip";

async fn username_of(state: &AppState, user_id: u64) -> String {
    state
        .users_repo
        .get_by_id(user_id)
        .await
        .and_then(|user| user.username)
        .unwrap_or_else(|| UNKNOWN_OPPONENT_NAME.to_string())
}

async fn opponent_name_in_game(state: &AppState, game_id: u64, user_id: u64) -> String {
    let Some(game) = state.games_repo.get_by_id(game_id).await else {
        return UNKNOWN_OPPONENT_NAME.to_string();
    };
    let opponent_id = if game.white_user_id == user_id {
        game.black_user_id
    } else {
        game.white_user_id
    };
    username_of(state, opponent_id).await
}

pub async fn handle(state: &AppState, user_id: u64) -> anyhow::Result<QuickMatchResultDto> {
    match state.matchmaking.advance(user_id).await? {
        QueueEvent::AlreadyMatched(game_id) => Ok(QuickMatchResultDto::Matched {
            game_id,
            opponent_name: opponent_name_in_game(state, game_id, user_id).await,
        }),
        QueueEvent::NowWaiting => Ok(QuickMatchResultDto::Waiting),
        QueueEvent::NoOpponent => Ok(QuickMatchResultDto::NoOpponent),
        QueueEvent::OpponentFound(opponent_id) => {
            let game = create_match::create(state, opponent_id, user_id).await?;
            state.matchmaking.record_match(opponent_id, game.id).await?;

            Ok(QuickMatchResultDto::Matched {
                game_id: game.id,
                opponent_name: username_of(state, opponent_id).await,
            })
        }
    }
}

pub async fn cancel(state: &AppState, user_id: u64) -> anyhow::Result<()> {
    state.matchmaking.leave(user_id).await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_match_names_the_opponent() {
        let matched = QuickMatchResultDto::Matched {
            game_id: 7,
            opponent_name: "Misafir123456".to_string(),
        };

        assert_eq!(
            serde_json::to_value(matched).unwrap(),
            serde_json::json!({"status": "matched", "game_id": 7, "opponent_name": "Misafir123456"})
        );
    }
}
