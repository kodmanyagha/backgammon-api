use tower::Layer;

use crate::service::authentication::jwt::{AppPerm, PermOperation};
use std::{
    future::Future,
    pin::Pin,
    task::{Context, Poll},
};

use axum::{
    extract::Request,
    response::{IntoResponse, Response},
};
use reqwest::StatusCode;
use tower::Service;

use crate::service::authentication::jwt::Claims;

#[derive(Clone)]
pub struct PermissionLayer {
    perm: AppPerm,
    operation: PermOperation,
}

impl PermissionLayer {
    pub fn new(perm: AppPerm, operation: PermOperation) -> Self {
        Self { perm, operation }
    }
}

impl<S> Layer<S> for PermissionLayer {
    type Service = PermissionLayerService<S>;

    fn layer(&self, inner: S) -> Self::Service {
        PermissionLayerService {
            inner,
            perm: self.perm,
            operation: self.operation,
        }
    }
}

#[derive(Clone)]
pub struct PermissionLayerService<S> {
    pub inner: S,
    perm: AppPerm,
    operation: PermOperation,
}

impl<S, B> Service<Request<B>> for PermissionLayerService<S>
where
    S: Service<Request<B>, Response = Response> + Clone + Send + 'static,
    S::Future: Send + 'static,
    B: Send + 'static,
{
    type Response = Response;
    type Error = S::Error;
    type Future = Pin<Box<dyn Future<Output = Result<Response, S::Error>> + Send>>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, req: Request<B>) -> Self::Future {
        let mut inner = self.inner.clone();
        let perm = self.perm;
        let operation = self.operation;
        let required_perm_str = format!("{operation}:{perm}");
        tracing::debug!(?required_perm_str, "required_perm_str");

        Box::pin(async move {
            let Some(claims) = req.extensions().get::<Claims>() else {
                return Ok(axum::response::IntoResponse::into_response(
                    StatusCode::UNAUTHORIZED,
                ));
            };

            for claim_perms in &claims.perms {
                if claim_perms.eq(&required_perm_str) {
                    return inner.call(req).await;
                }
            }

            Ok(StatusCode::FORBIDDEN.into_response())
        })
    }
}
