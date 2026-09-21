use axum::{
    extract::State,
    http::{header::CACHE_CONTROL, HeaderValue, StatusCode},
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;

use crate::{
    service::captcha::{start_challenge, CaptchaChallengeDto},
    state::app_state::AppState,
    types::http::api_response::ApiResponse,
    utils::consts::errors,
};

#[utoipa::path(
    post,
    path = "/v1/captcha/start",
    tag = "Captcha",
    operation_id = "captcha_start",
    responses(
        (status = 200, description = "A new challenge: `question` and the four `answers` are base64 encoded PNG images, `hash` identifies the challenge for one minute", body = CaptchaChallengeDto),
        (status = 429, description = "Too many challenges requested from this network", body = ApiResponse),
    ),
)]
pub async fn post_start(State(state): State<AppState>) -> Response {
    match start_challenge(&state).await {
        Ok(challenge) => {
            let mut response = Json(challenge).into_response();
            response
                .headers_mut()
                .insert(CACHE_CONTROL, HeaderValue::from_static("no-store"));
            response
        }
        Err(err) => {
            tracing::error!(error = %err, "captcha challenge could not be created");
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
