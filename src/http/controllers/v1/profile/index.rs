use axum::{extract::State, response::IntoResponse, Extension, Json};
use serde_json::json;

use crate::{state::app_state::AppState, types::http::api_response::ApiResponse};

#[utoipa::path(
    get,
    path = "/v1/profile",
    tag = "Profile",
    operation_id = "profile_get_index",
    responses(
        (status = 200, description = "Current user's profile, roles and permissions", body = ApiResponse),
    ),
    security(("bearer_auth" = [])),
)]
pub async fn get_index(
    Extension(user): Extension<entity::users::Model>,
    State(state): State<AppState>,
) -> impl IntoResponse {
    let mut response = ApiResponse::new();

    let roles_str = state
        .users_entity_service
        .get_user_roles(user.id)
        .await
        .iter()
        .map(|item| &item.key)
        .cloned()
        .collect::<Vec<String>>();

    response.set_data(json!({
       "user":  {
        "id": user.id,
        "firstname": user.firstname,
        "lastname": user.lastname,
        "email": user.email,
        "username": user.username,
        "is_guest": user.is_guest,
        "created_at": user.created_at,
        "updated_at": user.updated_at,
       },
       "roles": roles_str,
       "perms": Vec::<&str>::new(),
    }));

    Json(json!(response)).into_response()
}
