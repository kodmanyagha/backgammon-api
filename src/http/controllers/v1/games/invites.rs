use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    Extension, Json,
};
use serde_json::json;

use crate::{
    service::game::{accept_invite, create_invite, preview_invite},
    state::app_state::AppState,
    types::http::api_response::ApiResponse,
};

#[utoipa::path(
    post,
    path = "/v1/games/invites",
    tag = "Games",
    operation_id = "games_invites_post_create",
    responses(
        (status = 200, description = "Invite link token created", body = ApiResponse),
    ),
    security(("bearer_auth" = [])),
)]
pub async fn post_create(
    State(state): State<AppState>,
    Extension(user): Extension<entity::users::Model>,
) -> Response {
    match create_invite::handle(&state, user.id).await {
        Ok(result) => Json(json!(ApiResponse::new().with_data(json!(result)))).into_response(),
        Err(err) => (
            StatusCode::BAD_REQUEST,
            Json(json!(ApiResponse::new().with_global_error(&err.to_string()))),
        )
            .into_response(),
    }
}

#[utoipa::path(
    get,
    path = "/v1/games/invites/{token}",
    tag = "Games",
    operation_id = "games_invites_get_preview",
    params(("token" = String, Path, description = "Invite token")),
    responses(
        (status = 200, description = "Invite preview (inviter, status, expiry)", body = ApiResponse),
        (status = 400, description = "Invite not found", body = ApiResponse),
    ),
    security(("bearer_auth" = [])),
)]
pub async fn get_preview(State(state): State<AppState>, Path(token): Path<String>) -> Response {
    match preview_invite::handle(&state, &token).await {
        Ok(result) => Json(json!(ApiResponse::new().with_data(json!(result)))).into_response(),
        Err(err) => (
            StatusCode::BAD_REQUEST,
            Json(json!(ApiResponse::new().with_global_error(&err.to_string()))),
        )
            .into_response(),
    }
}

#[utoipa::path(
    post,
    path = "/v1/games/invites/{token}/accept",
    tag = "Games",
    operation_id = "games_invites_post_accept",
    params(("token" = String, Path, description = "Invite token")),
    responses(
        (status = 200, description = "Invite accepted, game created", body = ApiResponse),
        (status = 400, description = "Invite not found, expired, already used, or own invite", body = ApiResponse),
    ),
    security(("bearer_auth" = [])),
)]
pub async fn post_accept(
    State(state): State<AppState>,
    Extension(user): Extension<entity::users::Model>,
    Path(token): Path<String>,
) -> Response {
    match accept_invite::handle(&state, &token, user.id).await {
        Ok(game_id) => {
            Json(json!(ApiResponse::new().with_data(json!({ "game_id": game_id })))).into_response()
        }
        Err(err) => (
            StatusCode::BAD_REQUEST,
            Json(json!(ApiResponse::new().with_global_error(&err.to_string()))),
        )
            .into_response(),
    }
}
