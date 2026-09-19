use axum::{
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Response},
    Extension, Json,
};
use serde_json::json;

pub use crate::service::authentication::update_profile::UpdateProfileInputDto;
use crate::{
    service::authentication::{types::validated_json::ValidatedJson, update_profile::handle},
    state::app_state::AppState,
    types::http::api_response::ApiResponse,
};

#[utoipa::path(
    patch,
    path = "/v1/profile",
    tag = "Profile",
    operation_id = "profile_patch_update",
    request_body = UpdateProfileInputDto,
    responses(
        (status = 200, description = "Updated profile", body = ApiResponse),
        (status = 400, description = "Validation error, or username already taken", body = ApiResponse),
    ),
    security(("bearer_auth" = [])),
)]
pub async fn patch_update(
    State(state): State<AppState>,
    Extension(user): Extension<entity::users::Model>,
    ValidatedJson(input): ValidatedJson<UpdateProfileInputDto>,
) -> Response {
    match handle(&state, &user, &input).await {
        Ok(updated) => Json(json!(ApiResponse::new().with_data(json!({
            "id": updated.id,
            "email": updated.email,
            "username": updated.username,
            "is_guest": updated.is_guest,
        }))))
        .into_response(),
        Err(err) => (
            StatusCode::BAD_REQUEST,
            Json(json!(ApiResponse::new().with_global_error(&err.to_string()))),
        )
            .into_response(),
    }
}
