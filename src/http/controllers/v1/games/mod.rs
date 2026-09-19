use axum::{
    routing::{get, post},
    Router,
};

use crate::{
    http::controllers::{self, middleware::ip_rate_limit},
    state::app_state::AppState,
    utils::consts::route,
};

pub mod history;
pub mod invites;
pub mod quick_match;
pub mod show;
pub mod ws;

/// Kimlik doğrulaması GEREKTİRMEYEN uçlar — bir davet linkine tıklayan kişi
/// henüz giriş yapmamış/hesabı olmayabilir, davetin kimden geldiğini
/// (`preview`) görebilmesi login/misafir-girişinden ÖNCE gerekiyor.
pub fn public_routes(app_state: &AppState) -> Router<AppState> {
    Router::new()
        .nest(
            route::INVITES,
            Router::new()
                .route(
                    route::TOKEN,
                    get(controllers::v1::games::invites::get_preview),
                )
                .with_state(app_state.clone()),
        )
        .layer(axum::middleware::from_fn_with_state(
            app_state.clone(),
            ip_rate_limit::limit_public_requests,
        ))
        .with_state(app_state.clone())
}

/// Kimlik doğrulaması GEREKTİREN uçlar — `v1/routes.rs`'te `auth_check`
/// middleware'inin ARKASINA eklenir.
pub fn protected_routes(app_state: &AppState) -> Router<AppState> {
    Router::new()
        .route(
            route::QUICK_MATCH,
            post(controllers::v1::games::quick_match::post_join)
                .delete(controllers::v1::games::quick_match::delete_leave),
        )
        .nest(
            route::INVITES,
            Router::new()
                .route(
                    route::SLASH,
                    post(controllers::v1::games::invites::post_create),
                )
                .route(
                    &format!("{}{}", route::TOKEN, route::ACCEPT),
                    post(controllers::v1::games::invites::post_accept),
                )
                .with_state(app_state.clone()),
        )
        .route(
            route::HISTORY,
            get(controllers::v1::games::history::get_history),
        )
        .route(route::ID, get(controllers::v1::games::show::get_show))
        .route(
            &format!("{}{}", route::ID, route::WS),
            get(controllers::v1::games::ws::handle_upgrade),
        )
        .with_state(app_state.clone())
}
