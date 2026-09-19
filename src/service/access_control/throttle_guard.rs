use axum::{http::StatusCode, response::IntoResponse, response::Response, Json};
use serde_json::json;

use crate::{
    state::app_state::AppState, types::http::api_response::ApiResponse, utils::consts, CONFIG,
};

pub async fn enforce(state: &AppState, user_id: u64) -> Result<(), Response> {
    let requests_per_second = state
        .throttle_cache
        .limit_for(user_id)
        .await
        .unwrap_or_else(|| CONFIG.get_default_requests_per_second());

    if state.rate_limiter.allow(user_id, requests_per_second) {
        return Ok(());
    }

    let response = ApiResponse::new().with_global_error(consts::errors::RATE_LIMIT_EXCEEDED);
    Err((StatusCode::TOO_MANY_REQUESTS, Json(json!(response))).into_response())
}
