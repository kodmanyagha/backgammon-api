use std::{net::IpAddr, time::Duration};

use entity::utils::{
    enums::active_passive_status::ActivePassiveStatus, function_helpers::sha256_hex,
};
use serde::{Deserialize, Serialize};
use thiserror::Error;
use utoipa::ToSchema;
use validator::{Validate, ValidationError};

use crate::{
    service::{
        access_control::ip_rate_limiter::RateLimitScope,
        authentication::jwt::{encode_jwt, Claims},
    },
    state::app_state::AppState,
    utils::consts::errors,
    CONFIG,
};

use super::AuthResultDto;

const TOKEN_EXP_HOURS: i64 = 24;
const GUEST_CREATION_WINDOW: Duration = Duration::from_secs(60 * 60);
const GUEST_UNIQUE_ID_MIN_LEN: usize = 32;
const GUEST_UNIQUE_ID_MAX_LEN: usize = 128;
const GENERATED_USERNAME_ATTEMPTS: usize = 5;

#[derive(Clone, Debug, Serialize, Deserialize, Validate, ToSchema)]
pub struct GuestInputDto {
    pub display_name: Option<String>,
    #[validate(custom(function = "validate_guest_unique_id"))]
    pub guest_unique_id: String,
}

#[derive(Debug, Error)]
pub enum GuestLoginError {
    #[error("{}", errors::RATE_LIMIT_EXCEEDED)]
    RateLimited { retry_after: Duration },
    #[error("{}", errors::GUEST_CREATION_PAUSED)]
    CreationPaused { retry_after: Duration },
    #[error(transparent)]
    Failed(#[from] anyhow::Error),
}

fn validate_guest_unique_id(value: &str) -> Result<(), ValidationError> {
    let has_valid_len = (GUEST_UNIQUE_ID_MIN_LEN..=GUEST_UNIQUE_ID_MAX_LEN).contains(&value.len());
    let has_valid_chars = value
        .chars()
        .all(|character| character.is_ascii_alphanumeric() || character == '-' || character == '_');

    if has_valid_len && has_valid_chars {
        Ok(())
    } else {
        Err(ValidationError::new("invalid_guest_unique_id"))
    }
}

pub async fn handle(
    state: &AppState,
    input: &GuestInputDto,
    client_ip: Option<IpAddr>,
) -> Result<AuthResultDto, GuestLoginError> {
    let guest_unique_id_hash = sha256_hex(&input.guest_unique_id);

    let user = match state
        .users_repo
        .get_by_guest_unique_id_hash(&guest_unique_id_hash)
        .await
    {
        Some(existing) => existing,
        None => create_guest(state, input, &guest_unique_id_hash, client_ip).await?,
    };

    if user.status != ActivePassiveStatus::Active {
        return Err(anyhow::anyhow!(errors::UNAUTHORIZED).into());
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

async fn create_guest(
    state: &AppState,
    input: &GuestInputDto,
    guest_unique_id_hash: &str,
    client_ip: Option<IpAddr>,
) -> Result<entity::users::Model, GuestLoginError> {
    if let Some(ip) = client_ip {
        state
            .ip_rate_limiter
            .check(
                RateLimitScope::GuestCreation,
                ip,
                CONFIG.get_guest_creations_per_hour_per_ip(),
                GUEST_CREATION_WINDOW,
            )
            .map_err(|retry_after| GuestLoginError::RateLimited { retry_after })?;
    }

    state
        .guest_creation_limiter
        .check(
            CONFIG.get_guest_creations_per_hour_global(),
            GUEST_CREATION_WINDOW,
        )
        .map_err(|retry_after| GuestLoginError::CreationPaused { retry_after })?;

    let username = pick_available_username(state, input.display_name.as_deref()).await?;

    let created = match state
        .users_repo
        .create_guest_user(&username, guest_unique_id_hash)
        .await
    {
        Ok(user) => user,
        Err(err) => state
            .users_repo
            .get_by_guest_unique_id_hash(guest_unique_id_hash)
            .await
            .ok_or(err)?,
    };

    state
        .users_entity_service
        .assign_role_by_key(created.id, entity::roles::role_keys::GUEST)
        .await?;

    Ok(created)
}

async fn pick_available_username(
    state: &AppState,
    requested_display_name: Option<&str>,
) -> anyhow::Result<String> {
    let requested = requested_display_name
        .map(str::trim)
        .filter(|name| !name.is_empty());

    if let Some(name) = requested {
        return match state.users_repo.get_by_username(name).await {
            Some(_) => Err(anyhow::anyhow!(errors::USERNAME_ALREADY_TAKEN)),
            None => Ok(name.to_string()),
        };
    }

    for _ in 0..GENERATED_USERNAME_ATTEMPTS {
        let candidate = generate_guest_username();
        if state.users_repo.get_by_username(&candidate).await.is_none() {
            return Ok(candidate);
        }
    }

    Err(anyhow::anyhow!(errors::USERNAME_ALREADY_TAKEN))
}

fn generate_guest_username() -> String {
    let suffix: u32 = rand::random_range(100000..999999);
    format!("Misafir{suffix}")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn input_with_id(guest_unique_id: &str) -> GuestInputDto {
        GuestInputDto {
            display_name: None,
            guest_unique_id: guest_unique_id.to_string(),
        }
    }

    #[test]
    fn accepts_a_64_char_hex_device_secret() {
        assert!(input_with_id(&"a1b2c3d4".repeat(8)).validate().is_ok());
    }

    #[test]
    fn accepts_uuid_like_ids_with_dashes_and_underscores() {
        assert!(input_with_id("123e4567-e89b-12d3-a456-426614174000").validate().is_ok());
        assert!(input_with_id(&"A_b-".repeat(10)).validate().is_ok());
    }

    #[test]
    fn rejects_short_long_and_unsafe_ids() {
        assert!(input_with_id("").validate().is_err());
        assert!(input_with_id(&"a".repeat(GUEST_UNIQUE_ID_MIN_LEN - 1)).validate().is_err());
        assert!(input_with_id(&"a".repeat(GUEST_UNIQUE_ID_MAX_LEN + 1)).validate().is_err());
        assert!(input_with_id(&format!("{}'; DROP TABLE users;--", "a".repeat(32))).validate().is_err());
        assert!(input_with_id(&format!("{} {}", "a".repeat(20), "b".repeat(20))).validate().is_err());
    }

    #[test]
    fn generated_usernames_follow_the_guest_pattern() {
        let username = generate_guest_username();

        assert!(username.starts_with("Misafir"));
        assert_eq!(username.len(), "Misafir".len() + 6);
    }

    #[test]
    fn creation_paused_error_displays_its_own_error_key() {
        let err = GuestLoginError::CreationPaused {
            retry_after: Duration::from_secs(5),
        };

        assert_eq!(err.to_string(), errors::GUEST_CREATION_PAUSED);
    }

    #[test]
    fn rate_limited_error_displays_the_shared_error_key() {
        let err = GuestLoginError::RateLimited {
            retry_after: Duration::from_secs(5),
        };

        assert_eq!(err.to_string(), errors::RATE_LIMIT_EXCEEDED);
    }
}
