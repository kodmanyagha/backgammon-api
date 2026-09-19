use axum::{routing::get, Router};

use crate::{http::controllers, state::app_state::AppState, utils::consts::route};

pub fn routes(app_state: &AppState) -> axum::Router<AppState> {
    Router::new()
        .nest(route::AUTH, controllers::v1::auth::routes(app_state))
        .nest(route::GAMES, controllers::v1::games::public_routes(app_state))
        .merge(
            Router::new()
                .nest(
                    route::PROFILE,
                    Router::new()
                        .route(
                            route::SLASH,
                            get(controllers::v1::profile::index::get_index)
                                .patch(controllers::v1::profile::update::patch_update),
                        )
                        .route(route::SCORE, get(controllers::v1::profile::score::get_score))
                        .with_state(app_state.clone()),
                )
                .nest(
                    route::GAMES,
                    controllers::v1::games::protected_routes(app_state),
                )
                .nest(
                    route::SETTINGS,
                    Router::new()
                        .nest(
                            route::PERMISSIONS,
                            Router::new()
                                .route(
                                    route::SLASH,
                                    get(controllers::v1::settings::permissions::get_index),
                                )
                                .with_state(app_state.clone()),
                        )
                        .nest(
                            route::ROLES,
                            Router::new()
                                .route(
                                    route::SLASH,
                                    get(controllers::v1::settings::roles::get_index),
                                )
                                .with_state(app_state.clone()),
                        )
                        .with_state(app_state.clone()),
                )
                .layer(axum::middleware::from_fn_with_state(
                    app_state.clone(),
                    controllers::middleware::api::auth_check::handle,
                ))
                .with_state(app_state.clone()),
        )
        .with_state(app_state.clone())
}
