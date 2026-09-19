use axum::{routing::post, Router};

use crate::{
    http::controllers::{self, middleware::ip_rate_limit},
    state::app_state::AppState,
    utils::consts::route,
};

pub mod guest;
pub mod login;
pub mod register;

pub fn routes(app_state: &AppState) -> Router<AppState> {
    Router::new()
        .route(
            route::REGISTER,
            post(controllers::v1::auth::register::post_register),
        )
        .route(route::LOGIN, post(controllers::v1::auth::login::post_login))
        .route(route::GUEST, post(controllers::v1::auth::guest::post_guest))
        .layer(axum::middleware::from_fn_with_state(
            app_state.clone(),
            ip_rate_limit::limit_auth_requests,
        ))
        .with_state(app_state.clone())
}
