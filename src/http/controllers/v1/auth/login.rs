use axum::{
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;

pub use crate::service::authentication::login::LoginInputDto;
use crate::{
    service::authentication::{login::handle, types::validated_json::ValidatedJson},
    state::app_state::AppState,
    types::http::api_response::ApiResponse,
    utils::error_response::error_key_response,
};

#[utoipa::path(
    post,
    path = "/v1/auth/login",
    tag = "Auth",
    operation_id = "auth_login",
    request_body = LoginInputDto,
    responses(
        (status = 200, description = "Logged in, returns a bearer JWT", body = ApiResponse),
        (status = 400, description = "Invalid credentials", body = ApiResponse),
    ),
)]
pub async fn post_login(
    State(state): State<AppState>,
    ValidatedJson(input): ValidatedJson<LoginInputDto>,
) -> Response {
    match handle(&state, &input).await {
        Ok(result) => Json(json!(ApiResponse::new().with_data(json!(result)))).into_response(),
        Err(err) => error_key_response(StatusCode::BAD_REQUEST, err),
    }
}
