use std::env::{self, current_dir};

use app::app_config::{AppConfig, AppConfigBuilder};
use axum_client_ip::ClientIpSource;
use once_cell::sync::Lazy;

pub mod app;
pub mod http;
pub mod service;
pub mod state;
pub mod types;
pub mod utils;

const DEFAULT_MAX_REQUEST_BODY_BYTES: usize = 64 * 1024;
const DEFAULT_AUTH_REQUESTS_PER_MINUTE_PER_IP: u32 = 30;
const DEFAULT_PUBLIC_REQUESTS_PER_MINUTE_PER_IP: u32 = 60;
const DEFAULT_GUEST_CREATIONS_PER_HOUR_PER_IP: u32 = 20;
const DEFAULT_GUEST_CREATIONS_PER_HOUR_GLOBAL: u32 = 200;
const DEFAULT_CAPTCHA_CHALLENGE_TTL_SECONDS: u64 = 60;
const DEFAULT_CAPTCHA_VERIFICATION_TTL_SECONDS: u64 = 7 * 24 * 60 * 60;
const DEFAULT_CAPTCHA_STARTS_PER_MINUTE_PER_IP: u32 = 10;
const DEFAULT_CAPTCHA_VERIFICATIONS_PER_MINUTE_PER_IP: u32 = 30;

fn env_parsed_or<T: std::str::FromStr>(name: &str, default: T) -> T {
    env::var(name)
        .ok()
        .and_then(|value| value.parse::<T>().ok())
        .unwrap_or(default)
}

pub static CONFIG: Lazy<AppConfig> = Lazy::new(|| {
    let current_dir = current_dir().expect("Can not detect current working directory");

    AppConfigBuilder::default()
        .http_server_ip(env::var("HTTP_SERVER_IP").expect("HTTP_SERVER_IP env variable required"))
        .http_server_port(
            env::var("HTTP_SERVER_PORT").expect("HTTP_SERVER_PORT env variable required"),
        )
        .http_server_socket(
            env::var("HTTP_SERVER_SOCKET").expect("HTTP_SERVER_SOCKET env variable required"),
        )
        .http_allow_origin(
            env::var("HTTP_ALLOW_ORIGIN").expect("HTTP_ALLOW_ORIGIN env variable required"),
        )
        .docker_user(env::var("DOCKER_USER").expect("DOCKER_USER env variable required"))
        .app_secret(env::var("APP_SECRET").expect("APP_SECRET env variable required"))
        .default_requests_per_second(
            env::var("DEFAULT_REQUESTS_PER_SECOND")
                .ok()
                .and_then(|value| value.parse::<u64>().ok())
                .unwrap_or(25),
        )
        .max_request_body_bytes(env_parsed_or(
            "MAX_REQUEST_BODY_BYTES",
            DEFAULT_MAX_REQUEST_BODY_BYTES,
        ))
        .client_ip_source(match env::var("CLIENT_IP_SOURCE") {
            Ok(value) => value
                .parse::<ClientIpSource>()
                .expect("CLIENT_IP_SOURCE env variable not valid"),
            Err(_) => ClientIpSource::CfConnectingIp,
        })
        .auth_requests_per_minute_per_ip(env_parsed_or(
            "AUTH_REQUESTS_PER_MINUTE_PER_IP",
            DEFAULT_AUTH_REQUESTS_PER_MINUTE_PER_IP,
        ))
        .public_requests_per_minute_per_ip(env_parsed_or(
            "PUBLIC_REQUESTS_PER_MINUTE_PER_IP",
            DEFAULT_PUBLIC_REQUESTS_PER_MINUTE_PER_IP,
        ))
        .guest_creations_per_hour_per_ip(env_parsed_or(
            "GUEST_CREATIONS_PER_HOUR_PER_IP",
            DEFAULT_GUEST_CREATIONS_PER_HOUR_PER_IP,
        ))
        .guest_creations_per_hour_global(env_parsed_or(
            "GUEST_CREATIONS_PER_HOUR_GLOBAL",
            DEFAULT_GUEST_CREATIONS_PER_HOUR_GLOBAL,
        ))
        .captcha_challenge_ttl_seconds(env_parsed_or(
            "CAPTCHA_CHALLENGE_TTL_SECONDS",
            DEFAULT_CAPTCHA_CHALLENGE_TTL_SECONDS,
        ))
        .captcha_verification_ttl_seconds(env_parsed_or(
            "CAPTCHA_VERIFICATION_TTL_SECONDS",
            DEFAULT_CAPTCHA_VERIFICATION_TTL_SECONDS,
        ))
        .captcha_starts_per_minute_per_ip(env_parsed_or(
            "CAPTCHA_STARTS_PER_MINUTE_PER_IP",
            DEFAULT_CAPTCHA_STARTS_PER_MINUTE_PER_IP,
        ))
        .captcha_verifications_per_minute_per_ip(env_parsed_or(
            "CAPTCHA_VERIFICATIONS_PER_MINUTE_PER_IP",
            DEFAULT_CAPTCHA_VERIFICATIONS_PER_MINUTE_PER_IP,
        ))
        .database_url(env::var("DATABASE_URL").expect("DATABASE_URL env variable required"))
        .redis_url(env::var("REDIS_URL").expect("REDIS_URL env variable required"))
        .current_dir(current_dir)
        .build()
        .expect("Can't build AppConfig")
});
