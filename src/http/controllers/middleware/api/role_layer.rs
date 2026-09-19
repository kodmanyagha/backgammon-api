use std::{future::Future, pin::Pin};

use crate::service::authentication::jwt::AppRole;

use axum::{
    extract::Request,
    middleware::Next,
    response::{IntoResponse, Response},
};
use reqwest::StatusCode;

use crate::service::authentication::jwt::Claims;

pub fn require_role(
    required: AppRole,
) -> impl Fn(Request, Next) -> Pin<Box<dyn Future<Output = Response> + Send>> + Clone + Send + Sync + 'static
{
    move |req: Request, next: Next| {
        Box::pin(async move {
            let Some(claims) = req.extensions().get::<Claims>() else {
                return StatusCode::UNAUTHORIZED.into_response();
            };

            for claim_role in &claims.roles {
                if *claim_role <= required {
                    return next.run(req).await;
                }
            }

            StatusCode::FORBIDDEN.into_response()
        })
    }
}
