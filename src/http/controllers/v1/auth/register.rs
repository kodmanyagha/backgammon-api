use axum::{
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;

pub use crate::service::authentication::register::RegisterInputDto;
use crate::{
    service::authentication::{register::handle, types::validated_json::ValidatedJson},
    state::app_state::AppState,
    types::http::api_response::ApiResponse,
};

#[utoipa::path(
    post,
    path = "/v1/auth/register",
    tag = "Auth",
    operation_id = "auth_register",
    request_body = RegisterInputDto,
    responses(
        (status = 200, description = "Registered, returns a bearer JWT", body = ApiResponse),
        (status = 400, description = "Validation error, or email/username already taken", body = ApiResponse),
    ),
)]
pub async fn post_register(
    State(state): State<AppState>,
    ValidatedJson(input): ValidatedJson<RegisterInputDto>,
) -> Response {
    match handle(&state, &input).await {
        Ok(result) => Json(json!(ApiResponse::new().with_data(json!(result)))).into_response(),
        Err(err) => (
            StatusCode::BAD_REQUEST,
            Json(json!(ApiResponse::new().with_global_error(&err.to_string()))),
        )
            .into_response(),
    }
}
