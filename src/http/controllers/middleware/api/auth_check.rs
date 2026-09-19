use axum::{
    extract::{Request, State},
    http::StatusCode,
    middleware::Next,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;

use crate::{
    service::{access_control::throttle_guard, authentication::jwt::Claims},
    state::app_state::AppState,
    types::http::api_response::ApiResponse,
    utils::consts,
};

pub async fn handle(
    State(state): State<AppState>,
    claims: Claims,
    mut req: Request,
    next: Next,
) -> Response {
    if claims.roles.is_empty() {
        let response = ApiResponse::new().with_global_error(consts::errors::ROLE_MUST_BE_SET);
        return (StatusCode::UNAUTHORIZED, Json(json!(response))).into_response();
    }

    if let Err(response) = throttle_guard::enforce(&state, claims.sub).await {
        return response;
    }

    let Some(user) = state.users_repo.get_by_id_active(claims.sub).await else {
        let response = ApiResponse::new().with_global_error(consts::errors::USER_NOT_FOUND);
        return (StatusCode::UNAUTHORIZED, Json(json!(response))).into_response();
    };

    req.extensions_mut().insert(user);
    req.extensions_mut().insert(claims);

    next.run(req).await
}
