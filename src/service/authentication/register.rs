use entity::utils::password_helper::hash_password;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use validator::Validate;

use crate::{
    service::authentication::jwt::{encode_jwt, AppRole, Claims},
    state::app_state::AppState,
    utils::consts::errors,
};

use super::AuthResultDto;

const TOKEN_EXP_HOURS: i64 = 24 * 7;

#[derive(Clone, Debug, Default, Serialize, Deserialize, Validate, ToSchema)]
pub struct RegisterInputDto {
    #[validate(email(message = "error.validation.invalid_email"))]
    pub email: String,
    #[validate(length(min = 6, message = "error.validation.invalid_password_len"))]
    pub password: String,
    pub password_again: String,
    pub username: Option<String>,
}

pub async fn handle(state: &AppState, input: &RegisterInputDto) -> anyhow::Result<AuthResultDto> {
    if input.password != input.password_again {
        return Err(anyhow::anyhow!(errors::PASSWORDS_DO_NOT_MATCH));
    }

    if state.users_repo.get_by_email(&input.email).await.is_some() {
        return Err(anyhow::anyhow!(errors::EMAIL_ALREADY_TAKEN));
    }

    if let Some(username) = input.username.as_deref() {
        if state.users_repo.get_by_username(username).await.is_some() {
            return Err(anyhow::anyhow!(errors::USERNAME_ALREADY_TAKEN));
        }
    }

    let password_hash = hash_password(&input.password)?;
    let user = state
        .users_repo
        .register_user(&input.email, &password_hash, input.username.as_deref())
        .await?;

    state
        .users_entity_service
        .assign_role_by_key(user.id, entity::roles::role_keys::USER)
        .await?;

    let claims = Claims::new()
        .with_sub(user.id)
        .with_roles(vec![AppRole::User])
        .with_exp_hours(TOKEN_EXP_HOURS);
    let token = encode_jwt(claims)?;

    Ok(AuthResultDto {
        token,
        user_id: user.id,
        email: user.email,
        username: user.username,
        is_guest: user.is_guest,
    })
}
