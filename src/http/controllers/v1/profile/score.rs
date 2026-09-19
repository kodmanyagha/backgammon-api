use axum::{extract::State, response::IntoResponse, Extension, Json};
use serde_json::json;

use crate::{
    service::game::get_score, state::app_state::AppState, types::http::api_response::ApiResponse,
};

#[utoipa::path(
    get,
    path = "/v1/profile/score",
    tag = "Profile",
    operation_id = "profile_get_score",
    responses(
        (status = 200, description = "Caller's win/loss totals", body = ApiResponse),
    ),
    security(("bearer_auth" = [])),
)]
pub async fn get_score(
    State(state): State<AppState>,
    Extension(user): Extension<entity::users::Model>,
) -> impl IntoResponse {
    let score = get_score::handle(&state, user.id).await;

    Json(json!(ApiResponse::new().with_data(json!(score))))
}
