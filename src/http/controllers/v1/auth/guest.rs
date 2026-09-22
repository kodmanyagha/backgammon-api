use axum::{
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use axum_client_ip::{ClientIp, Rejection};
use serde_json::json;

pub use crate::service::authentication::guest::GuestInputDto;
use crate::{
    http::controllers::middleware::ip_rate_limit::{rate_limited_response, too_many_requests_response},
    service::authentication::{
        guest::{handle, GuestLoginError},
        types::validated_json::ValidatedJson,
    },
    state::app_state::AppState,
    types::http::api_response::ApiResponse,
    utils::{consts::errors, error_response::error_key_response},
};

#[utoipa::path(
    post,
    path = "/v1/auth/guest",
    tag = "Auth",
    operation_id = "auth_guest",
    request_body = GuestInputDto,
    responses(
        (status = 200, description = "Guest signed in, returns a bearer JWT. The same guest_unique_id always maps to the same user", body = ApiResponse),
        (status = 400, description = "Invalid guest_unique_id or display name already taken", body = ApiResponse),
        (status = 429, description = "Too many guest accounts created from this network, or new guest sign-ups are temporarily paused globally", body = ApiResponse),
    ),
)]
pub async fn post_guest(
    State(state): State<AppState>,
    client_ip: Result<ClientIp, Rejection>,
    ValidatedJson(input): ValidatedJson<GuestInputDto>,
) -> Response {
    let client_ip = client_ip.ok().map(|ClientIp(ip)| ip);

    match handle(&state, &input, client_ip).await {
        Ok(result) => Json(json!(ApiResponse::new().with_data(json!(result)))).into_response(),
        Err(GuestLoginError::RateLimited { retry_after }) => too_many_requests_response(retry_after),
        Err(GuestLoginError::CreationPaused { retry_after }) => {
            rate_limited_response(retry_after, errors::GUEST_CREATION_PAUSED)
        }
        Err(err) => error_key_response(StatusCode::BAD_REQUEST, err),
    }
}
