use anyhow::anyhow;
use axum::extract::DefaultBodyLimit;
use axum::http;
use axum::Router;
use std::{fs, path::PathBuf};
use tokio::join;
use tokio::net::{TcpListener, UnixListener};
use tower_http::cors::{Any, CorsLayer};

use crate::http::controllers::middleware::http_log;
use crate::http::routes;
use crate::{state::app_state::AppState, CONFIG};

use rustls::crypto::aws_lc_rs::default_provider;

async fn init_routes(app_state: &AppState) -> Router {
    let cors = CorsLayer::new()
        .allow_origin([CONFIG
            .get_http_allow_origin()
            .parse()
            .expect("HTTP_ALLOW_ORIGIN not valid.")])
        .allow_headers(Any)
        .allow_methods([
            http::Method::GET,
            http::Method::POST,
            http::Method::PUT,
            http::Method::PATCH,
        ]);

    Router::new()
        .merge(routes::all::merge(app_state))
        .layer(cors)
        .layer(DefaultBodyLimit::max(CONFIG.get_max_request_body_bytes()))
        .layer(axum::middleware::from_fn(http_log::handle))
        .layer(CONFIG.get_client_ip_source().clone().into_extension())
}

pub async fn handle() -> anyhow::Result<()> {
    default_provider()
        .install_default()
        .map_err(|_| anyhow::anyhow!("Rustls default provider error"))?;

    let app_state = AppState::try_new(None).await?;

    let app = init_routes(&app_state).await;

    let unix_socket_path = PathBuf::from(CONFIG.get_http_server_socket());
    if unix_socket_path.exists() {
        let _ = fs::remove_file(unix_socket_path.clone());
    }

    let unix_listener =
        UnixListener::bind(unix_socket_path.clone()).map_err(|err| anyhow!("Err: {err}"))?;
    tracing::warn!(
        "Listening on {}",
        unix_socket_path.to_str().unwrap_or_default()
    );

    let tcp_listener = TcpListener::bind(format!(
        "{}:{}",
        CONFIG.get_http_server_ip(),
        CONFIG.get_http_server_port()
    ))
    .await
    .map_err(|err| anyhow!("Err: {err}"))?;
    tracing::warn!("Listening on {}", tcp_listener.local_addr().unwrap());

    let _ = join!(
        axum::serve(unix_listener, app.clone()),
        axum::serve(tcp_listener, app.clone()),
    );

    Ok(())
}
