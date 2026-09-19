use serde::Serialize;
use utoipa::ToSchema;
use uuid::Uuid;

use crate::state::app_state::AppState;

#[derive(Clone, Debug, Serialize, ToSchema)]
pub struct InviteResultDto {
    pub token: String,
    pub expires_at: Option<chrono::NaiveDateTime>,
}

pub async fn handle(state: &AppState, created_by_user_id: u64) -> anyhow::Result<InviteResultDto> {
    let token = Uuid::new_v4().simple().to_string();
    let invite = state
        .game_invites_repo
        .create(&token, created_by_user_id)
        .await?;

    Ok(InviteResultDto {
        token: invite.token,
        expires_at: invite.expires_at,
    })
}
