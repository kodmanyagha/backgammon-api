use axum::{
    extract::{rejection::JsonRejection, State},
    http::{header::CACHE_CONTROL, HeaderValue, StatusCode},
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;

use crate::{
    http::controllers::middleware::captcha_verification::captcha_invalid_response,
    service::captcha::{
        verify_answer, CaptchaAnswerInputDto, CaptchaInvalidDto, CaptchaVerifiedDto,
        VerificationOutcome,
    },
    state::app_state::AppState,
    types::http::api_response::ApiResponse,
    utils::consts::errors,
};

#[utoipa::path(
    post,
    path = "/v1/captcha/verify",
    tag = "Captcha",
    operation_id = "captcha_verify",
    request_body = CaptchaAnswerInputDto,
    responses(
        (status = 200, description = "Solved. Send `hash` as the `x-captcha-verification` header of every later request until `expires_at` (unix seconds, UTC)", body = CaptchaVerifiedDto),
        (status = 400, description = "Wrong, unknown, expired or malformed answer. The challenge is consumed, start a new one", body = CaptchaInvalidDto),
        (status = 429, description = "Too many answers sent from this network", body = ApiResponse),
    ),
)]
pub async fn post_verify(
    State(state): State<AppState>,
    input: Result<Json<CaptchaAnswerInputDto>, JsonRejection>,
) -> Response {
    let Ok(Json(input)) = input else {
        return captcha_invalid_response();
    };

    match verify_answer(&state, &input).await {
        Ok(VerificationOutcome::Verified(verified)) => {
            let mut response = Json(verified).into_response();
            response
                .headers_mut()
                .insert(CACHE_CONTROL, HeaderValue::from_static("no-store"));
            response
        }
        Ok(VerificationOutcome::Invalid) => captcha_invalid_response(),
        Err(err) => {
            tracing::error!(error = %err, "captcha answer could not be verified");
            (
                StatusCode::SERVICE_UNAVAILABLE,
                Json(json!(
                    ApiResponse::new().with_global_error(errors::CAPTCHA_UNAVAILABLE)
                )),
            )
                .into_response()
        }
    }
}
