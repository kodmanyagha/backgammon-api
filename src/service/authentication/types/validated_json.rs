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
use validator::Validate;

use crate::types::http::api_response::ApiResponse;

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

impl IntoResponse for ValidatedJsonError {
    fn into_response(self) -> Response {
        match self {
            ValidatedJsonError::ValidationError(err) => {
                let response = ApiResponse::new().with_error(
                    err.0
                        .into_iter()
                        .map(|(key, val)| (key.to_string(), format!("{:?}", val)))
                        .collect(),
                );

                (StatusCode::BAD_REQUEST, Json(json!(response)))
            }
            ValidatedJsonError::AxumJsonRejection(err) => (
                StatusCode::BAD_REQUEST,
                Json(json!(
                    ApiResponse::new().with_global_error(&format!("{err}"))
                )),
            ),
        }
        .into_response()
    }
}
