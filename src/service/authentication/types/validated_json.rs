use axum::{
    extract::{
        rejection::{FormRejection, JsonRejection},
        Form, FromRequest, Request,
    },
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde::de::DeserializeOwned;
use serde_json::json;
use thiserror::Error;
use validator::{Validate, ValidationErrorsKind};

use crate::{types::http::api_response::ApiResponse, utils::consts::errors};

#[derive(Debug, Clone, Copy, Default)]
pub struct ValidatedJson<T>(pub T);

impl<T, S> FromRequest<S> for ValidatedJson<T>
where
    T: DeserializeOwned + Validate,
    S: Send + Sync,
    Form<T>: FromRequest<S, Rejection = FormRejection>,
{
    type Rejection = ValidatedJsonError;

    async fn from_request(req: Request, state: &S) -> Result<Self, Self::Rejection> {
        let Json(value) = Json::<T>::from_request(req, state).await?;
        value.validate()?;
        Ok(ValidatedJson(value))
    }
}

#[derive(Debug, Error)]
pub enum ValidatedJsonError {
    #[error(transparent)]
    ValidationError(#[from] validator::ValidationErrors),

    #[error(transparent)]
    AxumJsonRejection(#[from] JsonRejection),
}

fn error_key_for(kind: &ValidationErrorsKind) -> String {
    match kind {
        ValidationErrorsKind::Field(field_errors) => field_errors
            .first()
            .and_then(|error| error.message.as_deref())
            .unwrap_or(errors::validation::REQUIRED)
            .to_string(),
        ValidationErrorsKind::Struct(_) | ValidationErrorsKind::List(_) => {
            errors::validation::REQUIRED.to_string()
        }
    }
}

impl IntoResponse for ValidatedJsonError {
    fn into_response(self) -> Response {
        match self {
            ValidatedJsonError::ValidationError(err) => {
                let response = ApiResponse::new().with_error(
                    err.0
                        .iter()
                        .map(|(key, val)| (key.to_string(), error_key_for(val)))
                        .collect(),
                );

                (StatusCode::BAD_REQUEST, Json(json!(response)))
            }
            ValidatedJsonError::AxumJsonRejection(err) => {
                tracing::warn!(error = %err, "malformed request body");
                (
                    StatusCode::BAD_REQUEST,
                    Json(json!(
                        ApiResponse::new().with_global_error(errors::INVALID_REQUEST_BODY)
                    )),
                )
            }
        }
        .into_response()
    }
}
