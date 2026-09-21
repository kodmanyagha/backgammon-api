use axum::{
    extract::{Request, State},
    http::{header::CACHE_CONTROL, HeaderValue, StatusCode},
    middleware::Next,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;

use crate::{
    service::captcha::{
        is_verification_valid, CaptchaInvalidDto, CAPTCHA_VERIFICATION_HEADER, STATUS_INVALID,
    },
    state::app_state::AppState,
    types::http::api_response::ApiResponse,
    utils::consts::errors,
};

pub async fn require_verification(
    State(state): State<AppState>,
    req: Request,
    next: Next,
) -> Response {
    let Some(hash) = req
        .headers()
        .get(CAPTCHA_VERIFICATION_HEADER)
        .and_then(|value| value.to_str().ok())
    else {
        return captcha_invalid_response();
    };

    match is_verification_valid(&state, hash).await {
        Ok(true) => next.run(req).await,
        Ok(false) => captcha_invalid_response(),
        Err(err) => {
            tracing::error!(error = %err, "captcha verification lookup failed");
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

pub fn captcha_invalid_response() -> Response {
    let mut response = (
        StatusCode::BAD_REQUEST,
        Json(CaptchaInvalidDto {
            status: STATUS_INVALID.to_string(),
        }),
    )
        .into_response();
    response
        .headers_mut()
        .insert(CACHE_CONTROL, HeaderValue::from_static("no-store"));
    response
}

#[cfg(test)]
mod tests {
    use axum::body::to_bytes;

    use super::*;

    #[tokio::test]
    async fn invalid_response_is_a_400_with_only_the_status_field() {
        let response = captcha_invalid_response();

        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
        assert_eq!(response.headers().get(CACHE_CONTROL).unwrap(), "no-store");
        let body = to_bytes(response.into_body(), 1024).await.unwrap();
        assert_eq!(body.as_ref(), br#"{"status":"captcha_invalid"}"#);
    }
}
