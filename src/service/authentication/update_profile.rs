use entity::utils::password_helper::hash_password;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use validator::Validate;

use crate::{state::app_state::AppState, utils::consts::errors};

#[derive(Clone, Debug, Default, Serialize, Deserialize, Validate, ToSchema)]
pub struct UpdateProfileInputDto {
    pub username: Option<String>,
    #[validate(length(min = 6, message = "Password must be at least 6 characters"))]
    pub password: Option<String>,
    pub password_again: Option<String>,
}

pub async fn handle(
    state: &AppState,
    user: &entity::users::Model,
    input: &UpdateProfileInputDto,
) -> anyhow::Result<entity::users::Model> {
    if let Some(username) = input.username.as_deref() {
        if let Some(existing) = state.users_repo.get_by_username(username).await {
            if existing.id != user.id {
                return Err(anyhow::anyhow!(errors::USERNAME_ALREADY_TAKEN));
            }
        }

        state.users_repo.update_username(user.id, username).await?;
    }

    if let Some(password) = input.password.as_deref() {
        if input.password_again.as_deref() != Some(password) {
            return Err(anyhow::anyhow!(errors::PASSWORDS_DO_NOT_MATCH));
        }

        let password_hash = hash_password(password)?;
        state
            .users_repo
            .update_user(user.id, None, Some(&password_hash), None, None, None)
            .await?;
    }

    state
        .users_repo
        .get_by_id(user.id)
        .await
        .ok_or_else(|| anyhow::anyhow!(errors::USER_NOT_FOUND))
}
