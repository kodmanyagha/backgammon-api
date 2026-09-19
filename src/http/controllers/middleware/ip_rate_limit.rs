use std::{
    sync::atomic::{AtomicBool, Ordering},
    time::Duration,
};

use axum::{
    extract::{Request, State},
    http::{header::RETRY_AFTER, HeaderValue, StatusCode},
    middleware::Next,
    response::{IntoResponse, Response},
    Json,
};
use axum_client_ip::{ClientIp, Rejection};
use serde_json::json;

use crate::{
    service::access_control::ip_rate_limiter::RateLimitScope, state::app_state::AppState,
    types::http::api_response::ApiResponse, utils::consts::errors, CONFIG,
};

const REQUEST_WINDOW: Duration = Duration::from_secs(60);

static UNRESOLVED_CLIENT_IP_WARNED: AtomicBool = AtomicBool::new(false);

/// Limits how often one network may call the unauthenticated `/v1/auth` endpoints.
pub async fn limit_auth_requests(
    State(state): State<AppState>,
    client_ip: Result<ClientIp, Rejection>,
    req: Request,
    next: Next,
) -> Response {
    enforce(
        &state,
        RateLimitScope::AuthRequests,
        CONFIG.get_auth_requests_per_minute_per_ip(),
        client_ip,
        req,
        next,
    )
    .await
}

/// Limits how often one network may call the unauthenticated public game endpoints.
pub async fn limit_public_requests(
    State(state): State<AppState>,
    client_ip: Result<ClientIp, Rejection>,
    req: Request,
    next: Next,
) -> Response {
    enforce(
        &state,
        RateLimitScope::PublicRequests,
        CONFIG.get_public_requests_per_minute_per_ip(),
        client_ip,
        req,
        next,
    )
    .await
}

async fn enforce(
    state: &AppState,
    scope: RateLimitScope,
    limit: u32,
    client_ip: Result<ClientIp, Rejection>,
    req: Request,
    next: Next,
) -> Response {
    let Ok(ClientIp(ip)) = client_ip else {
        warn_once_about_unresolved_client_ip();
        return next.run(req).await;
    };

    match state.ip_rate_limiter.check(scope, ip, limit, REQUEST_WINDOW) {
        Ok(()) => next.run(req).await,
        Err(retry_after) => too_many_requests_response(retry_after),
    }
}

fn warn_once_about_unresolved_client_ip() {
    if !UNRESOLVED_CLIENT_IP_WARNED.swap(true, Ordering::Relaxed) {
        tracing::warn!(
            source = %CONFIG.get_client_ip_source(),
            "client ip could not be resolved, per-ip rate limits are skipped; check CLIENT_IP_SOURCE"
        );
    }
}

/// Builds the `429` response asking the client to retry once `retry_after` has passed.
pub fn too_many_requests_response(retry_after: Duration) -> Response {
    let retry_after_seconds = retry_after.as_secs().max(1);
    let mut response = (
        StatusCode::TOO_MANY_REQUESTS,
        Json(json!(
            ApiResponse::new().with_global_error(errors::RATE_LIMIT_EXCEEDED)
        )),
    )
        .into_response();
    response
        .headers_mut()
        .insert(RETRY_AFTER, HeaderValue::from(retry_after_seconds));
    response
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn too_many_requests_response_carries_a_retry_after_of_at_least_one_second() {
        let response = too_many_requests_response(Duration::from_millis(200));

        assert_eq!(response.status(), StatusCode::TOO_MANY_REQUESTS);
        assert_eq!(response.headers().get(RETRY_AFTER).unwrap(), "1");
    }

    #[test]
    fn too_many_requests_response_rounds_down_whole_seconds() {
        let response = too_many_requests_response(Duration::from_secs(42));

        assert_eq!(response.headers().get(RETRY_AFTER).unwrap(), "42");
    }
}
