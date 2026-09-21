use axum::{routing::post, Router};

use crate::{
    http::controllers::middleware::ip_rate_limit, state::app_state::AppState, utils::consts::route,
};

pub mod start;
pub mod verify;

pub fn routes(app_state: &AppState) -> Router<AppState> {
    Router::new()
        .route(
            route::START,
            post(start::post_start).layer(axum::middleware::from_fn_with_state(
                app_state.clone(),
                ip_rate_limit::limit_captcha_start_requests,
            )),
        )
        .route(
            route::VERIFY,
            post(verify::post_verify).layer(axum::middleware::from_fn_with_state(
                app_state.clone(),
                ip_rate_limit::limit_captcha_verify_requests,
            )),
        )
        .with_state(app_state.clone())
}
