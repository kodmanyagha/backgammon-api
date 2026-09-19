use axum::{
    extract::{rejection::FormRejection, Form, FromRequest, Request},
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde::de::DeserializeOwned;
use thiserror::Error;
use validator::Validate;

#[derive(Debug, Clone, Copy, Default)]
pub struct ValidatedForm<T>(pub T);

impl<T, S> FromRequest<S> for ValidatedForm<T>
where
    T: DeserializeOwned + Validate,
    S: Send + Sync,
    Form<T>: FromRequest<S, Rejection = FormRejection>,
{
    type Rejection = ValidatedFormError;

    async fn from_request(req: Request, state: &S) -> Result<Self, Self::Rejection> {
        let Form(value) = Form::<T>::from_request(req, state).await?;
        value.validate()?;
        Ok(ValidatedForm(value))
    }
}

#[derive(Debug, Error)]
pub enum ValidatedFormError {
    #[error(transparent)]
    ValidationError(#[from] validator::ValidationErrors),

    #[error(transparent)]
    AxumFormRejection(#[from] FormRejection),
}

impl IntoResponse for ValidatedFormError {
    fn into_response(self) -> Response {
        match self {
            ValidatedFormError::ValidationError(_) => {
                let message = format!("Input validation error: [{self}]").replace('\n', ", ");
                (StatusCode::BAD_REQUEST, message)
            }
            ValidatedFormError::AxumFormRejection(_) => (StatusCode::BAD_REQUEST, self.to_string()),
        }
        .into_response()
    }
}
