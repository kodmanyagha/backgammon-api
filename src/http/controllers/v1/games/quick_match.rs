use axum::{
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Response},
    Extension, Json,
};
use serde_json::json;

use crate::{
    service::game::quick_match::{cancel, handle},
    state::app_state::AppState,
    types::http::api_response::ApiResponse,
    utils::error_response::error_key_response,
};

#[utoipa::path(
    post,
    path = "/v1/games/quick-match",
    tag = "Games",
    operation_id = "games_quick_match_post",
    responses(
        (status = 200, description = "Waiting for an opponent, matched into a game, or `no_opponent` after the 60 second wait", body = ApiResponse),
    ),
    security(("bearer_auth" = [])),
)]
pub async fn post_join(
    State(state): State<AppState>,
    Extension(user): Extension<entity::users::Model>,
) -> Response {
    match handle(&state, user.id).await {
        Ok(result) => Json(json!(ApiResponse::new().with_data(json!(result)))).into_response(),
        Err(err) => error_key_response(StatusCode::BAD_REQUEST, err),
    }
}

#[utoipa::path(
    delete,
    path = "/v1/games/quick-match",
    tag = "Games",
    operation_id = "games_quick_match_delete",
    responses(
        (status = 200, description = "Left the matchmaking queue", body = ApiResponse),
    ),
    security(("bearer_auth" = [])),
)]
pub async fn delete_leave(
    State(state): State<AppState>,
    Extension(user): Extension<entity::users::Model>,
) -> Response {
    match cancel(&state, user.id).await {
        Ok(()) => Json(json!(ApiResponse::new().with_data(json!({ "left": true })))).into_response(),
        Err(err) => error_key_response(StatusCode::BAD_REQUEST, err),
    }
}
