use entity::utils::{enums::active_passive_status::ActivePassiveStatus, password_helper::verify_password};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use validator::Validate;

use crate::{
    service::authentication::jwt::{encode_jwt, Claims},
    state::app_state::AppState,
    utils::consts::errors,
};

use super::AuthResultDto;

const TOKEN_EXP_HOURS: i64 = 24 * 7;

#[derive(Clone, Debug, Default, Serialize, Deserialize, Validate, ToSchema)]
pub struct LoginInputDto {
    #[validate(email(message = "error.validation.invalid_email"))]
    pub email: String,
    #[validate(length(min = 1, message = "error.validation.required"))]
    pub password: String,
}

pub async fn handle(state: &AppState, input: &LoginInputDto) -> anyhow::Result<AuthResultDto> {
    let user = state
        .users_repo
        .get_by_email(&input.email)
        .await
        .ok_or_else(|| anyhow::anyhow!(errors::INVALID_CREDENTIALS))?;

    let password_hash = user
        .password
        .as_deref()
        .ok_or_else(|| anyhow::anyhow!(errors::INVALID_CREDENTIALS))?;

    if !verify_password(&input.password, password_hash) {
        return Err(anyhow::anyhow!(errors::INVALID_CREDENTIALS));
    }

    if user.status != ActivePassiveStatus::Active {
        return Err(anyhow::anyhow!(errors::UNAUTHORIZED));
    }

    let roles = state.users_entity_service.get_user_roles_enum(user.id).await;

    let claims = Claims::new()
        .with_sub(user.id)
        .with_roles(roles)
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
