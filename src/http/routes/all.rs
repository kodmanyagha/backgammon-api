use axum::{routing::get, Router};
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

use crate::{
    http::{controllers, openapi::ApiDoc},
    state::app_state::AppState,
    utils::consts::route,
};

pub fn merge(app_state: &AppState) -> Router {
    Router::new()
        .route(route::SLASH, get(controllers::home::index::get_index))
        .merge(Router::new().nest(route::V1, controllers::v1::routes::routes(app_state)))
        .with_state(app_state.clone())
        .merge(SwaggerUi::new("/swagger-ui").url("/api-docs/openapi.json", ApiDoc::openapi()))
}
