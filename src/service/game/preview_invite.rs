use entity::game_invites::GameInviteStatus;
use serde::Serialize;
use utoipa::ToSchema;

use crate::{state::app_state::AppState, utils::consts::errors};

#[derive(Clone, Debug, Serialize, ToSchema)]
pub struct InvitePreviewDto {
    pub status: GameInviteStatus,
    pub inviter_username: Option<String>,
    pub expires_at: Option<chrono::NaiveDateTime>,
}

pub async fn handle(state: &AppState, token: &str) -> anyhow::Result<InvitePreviewDto> {
    let invite = state
        .game_invites_repo
        .get_by_token(token)
        .await
        .ok_or_else(|| anyhow::anyhow!(errors::INVITE_NOT_FOUND))?;

    let inviter = state.users_repo.get_by_id(invite.created_by_user_id).await;

    Ok(InvitePreviewDto {
        status: invite.status,
        inviter_username: inviter.and_then(|user| user.username),
        expires_at: invite.expires_at,
    })
}
