use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    Extension, Json,
};
use serde_json::json;

use crate::{
    service::game::get_game, state::app_state::AppState, types::http::api_response::ApiResponse,
    utils::error_response::error_key_response,
};

#[utoipa::path(
    get,
    path = "/v1/games/{id}",
    tag = "Games",
    operation_id = "games_get_show",
    params(("id" = u64, Path, description = "Game id")),
    responses(
        (status = 200, description = "Game details", body = ApiResponse),
        (status = 400, description = "Game not found, or not one of your games", body = ApiResponse),
    ),
    security(("bearer_auth" = [])),
)]
pub async fn get_show(
    State(state): State<AppState>,
    Extension(user): Extension<entity::users::Model>,
    Path(id): Path<u64>,
) -> Response {
    match get_game::handle(&state, id, user.id).await {
        Ok(game) => Json(json!(ApiResponse::new().with_data(json!(game)))).into_response(),
        Err(err) => error_key_response(StatusCode::BAD_REQUEST, err),
    }
}
