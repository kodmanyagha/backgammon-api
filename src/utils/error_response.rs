use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;

use crate::{types::http::api_response::ApiResponse, utils::consts::errors};

pub fn error_key_response(status: StatusCode, err: impl std::fmt::Display) -> Response {
    let key = err.to_string();
    let safe_key = if key.starts_with("error.") {
        key
    } else {
        tracing::error!(error = %key, "unexpected internal error");
        errors::INTERNAL_ERROR.to_string()
    };

    (
        status,
        Json(json!(ApiResponse::new().with_global_error(&safe_key))),
    )
        .into_response()
}
