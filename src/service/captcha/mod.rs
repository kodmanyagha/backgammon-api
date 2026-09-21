use base64::{engine::general_purpose::STANDARD, Engine};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::{state::app_state::AppState, CONFIG};

pub mod challenge;
pub mod problem;
pub mod renderer;
pub mod store;

pub const CAPTCHA_VERIFICATION_HEADER: &str = "x-captcha-verification";
pub const STATUS_VERIFIED: &str = "captcha_verified";
pub const STATUS_INVALID: &str = "captcha_invalid";

#[derive(Clone, Debug, Serialize, ToSchema)]
pub struct CaptchaChallengeDto {
    pub hash: String,
    pub question: String,
    pub answers: Vec<String>,
}

#[derive(Clone, Debug, Deserialize, ToSchema)]
pub struct CaptchaAnswerInputDto {
    pub hash: String,
    pub answer: u8,
}

#[derive(Clone, Debug, Serialize, ToSchema)]
pub struct CaptchaVerifiedDto {
    pub status: String,
    pub hash: String,
    pub expires_at: i64,
}

#[derive(Clone, Debug, Serialize, ToSchema)]
pub struct CaptchaInvalidDto {
    pub status: String,
}

#[derive(Debug)]
pub enum VerificationOutcome {
    Verified(CaptchaVerifiedDto),
    Invalid,
}

pub async fn start_challenge(state: &AppState) -> anyhow::Result<CaptchaChallengeDto> {
    let generated = tokio::task::spawn_blocking(challenge::generate_challenge).await??;

    store::save_challenge(
        &state.redis_service,
        &generated.hash,
        generated.correct_answer,
        CONFIG.get_captcha_challenge_ttl_seconds(),
    )
    .await?;

    Ok(CaptchaChallengeDto {
        hash: generated.hash,
        question: STANDARD.encode(&generated.question_png),
        answers: generated
            .option_pngs
            .iter()
            .map(|png| STANDARD.encode(png))
            .collect(),
    })
}

pub async fn verify_answer(
    state: &AppState,
    input: &CaptchaAnswerInputDto,
) -> anyhow::Result<VerificationOutcome> {
    if !challenge::is_valid_hash_format(&input.hash) {
        return Ok(VerificationOutcome::Invalid);
    }

    let Some(correct_answer) = store::take_challenge_answer(&state.redis_service, &input.hash).await? else {
        return Ok(VerificationOutcome::Invalid);
    };
    if correct_answer != input.answer {
        return Ok(VerificationOutcome::Invalid);
    }

    let verification_ttl_seconds = CONFIG.get_captcha_verification_ttl_seconds();
    let expires_at = Utc::now().timestamp() + verification_ttl_seconds as i64;
    store::save_verification(
        &state.redis_service,
        &input.hash,
        expires_at,
        verification_ttl_seconds,
    )
    .await?;

    Ok(VerificationOutcome::Verified(CaptchaVerifiedDto {
        status: STATUS_VERIFIED.to_string(),
        hash: input.hash.clone(),
        expires_at,
    }))
}

pub async fn is_verification_valid(state: &AppState, hash: &str) -> anyhow::Result<bool> {
    if !challenge::is_valid_hash_format(hash) {
        return Ok(false);
    }

    store::is_verified(&state.redis_service, hash).await
}
