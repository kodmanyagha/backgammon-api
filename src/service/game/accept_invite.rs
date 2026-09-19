use chrono::Utc;
use entity::game_invites::GameInviteStatus;

use crate::{state::app_state::AppState, utils::consts::errors};

pub async fn handle(
    state: &AppState,
    token: &str,
    accepting_user_id: u64,
) -> anyhow::Result<u64> {
    let invite = state
        .game_invites_repo
        .get_by_token(token)
        .await
        .ok_or_else(|| anyhow::anyhow!(errors::INVITE_NOT_FOUND))?;

    if invite.status != GameInviteStatus::Pending {
        return Err(anyhow::anyhow!(errors::INVITE_NOT_AVAILABLE));
    }

    if let Some(expires_at) = invite.expires_at {
        if expires_at < Utc::now().naive_utc() {
            return Err(anyhow::anyhow!(errors::INVITE_EXPIRED));
        }
    }

    if invite.created_by_user_id == accepting_user_id {
        return Err(anyhow::anyhow!(errors::CANNOT_ACCEPT_OWN_INVITE));
    }

    let game = state
        .games_repo
        .create(invite.created_by_user_id, accepting_user_id)
        .await?;

    state
        .game_invites_repo
        .mark_accepted(invite.id, game.id)
        .await?;

    Ok(game.id)
}
