use std::path::PathBuf;

use axum_client_ip::ClientIpSource;
use derive_builder::Builder;

#[derive(Clone, Builder)]
pub struct AppConfig {
    http_server_ip: String,
    http_server_port: String,
    http_server_socket: String,
    http_allow_origin: String,

    current_dir: PathBuf,

    docker_user: String,

    database_url: String,
    redis_url: String,
    app_secret: String,
    default_requests_per_second: u64,

    max_request_body_bytes: usize,
    client_ip_source: ClientIpSource,
    auth_requests_per_minute_per_ip: u32,
    public_requests_per_minute_per_ip: u32,
    guest_creations_per_hour_per_ip: u32,
    guest_creations_per_hour_global: u32,
    captcha_challenge_ttl_seconds: u64,
    captcha_verification_ttl_seconds: u64,
    captcha_starts_per_minute_per_ip: u32,
    captcha_verifications_per_minute_per_ip: u32,
}

impl AppConfig {
    pub fn get_database_url(&self) -> &str {
        &self.database_url
    }

    pub fn get_redis_url(&self) -> &str {
        &self.redis_url
    }

    pub fn get_app_secret(&self) -> &str {
        &self.app_secret
    }

    pub fn get_default_requests_per_second(&self) -> u64 {
        self.default_requests_per_second
    }

    pub fn get_max_request_body_bytes(&self) -> usize {
        self.max_request_body_bytes
    }

    pub fn get_client_ip_source(&self) -> &ClientIpSource {
        &self.client_ip_source
    }

    pub fn get_auth_requests_per_minute_per_ip(&self) -> u32 {
        self.auth_requests_per_minute_per_ip
    }

    pub fn get_public_requests_per_minute_per_ip(&self) -> u32 {
        self.public_requests_per_minute_per_ip
    }

    pub fn get_guest_creations_per_hour_per_ip(&self) -> u32 {
        self.guest_creations_per_hour_per_ip
    }

    pub fn get_guest_creations_per_hour_global(&self) -> u32 {
        self.guest_creations_per_hour_global
    }

    pub fn get_captcha_challenge_ttl_seconds(&self) -> u64 {
        self.captcha_challenge_ttl_seconds
    }

    pub fn get_captcha_verification_ttl_seconds(&self) -> u64 {
        self.captcha_verification_ttl_seconds
    }

    pub fn get_captcha_starts_per_minute_per_ip(&self) -> u32 {
        self.captcha_starts_per_minute_per_ip
    }

    pub fn get_captcha_verifications_per_minute_per_ip(&self) -> u32 {
        self.captcha_verifications_per_minute_per_ip
    }

    pub fn get_http_allow_origin(&self) -> &str {
        &self.http_allow_origin
    }

    pub fn get_http_server_ip(&self) -> &str {
        &self.http_server_ip
    }

    pub fn get_http_server_port(&self) -> &str {
        &self.http_server_port
    }

    pub fn get_http_server_socket(&self) -> &str {
        &self.http_server_socket
    }

    pub fn get_current_dir(&self) -> &PathBuf {
        &self.current_dir
    }

    pub fn get_docker_user(&self) -> &str {
        &self.docker_user
    }
}
