use axum::{
    body::{to_bytes, Body, HttpBody},
    extract::Request,
    http::{
        header::{AUTHORIZATION, CONTENT_TYPE, COOKIE, SET_COOKIE, UPGRADE},
        HeaderMap, Method, StatusCode, Uri,
    },
    middleware::Next,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::{json, Value};

use crate::{types::http::api_response::ApiResponse, utils::consts::errors, CONFIG};

const MAX_LOGGED_RESPONSE_BYTES: usize = 256 * 1024;
const MAX_LOGGED_BODY_CHARS: usize = 2048;
const REDACTED_VALUE: &str = "<redacted>";
const SENSITIVE_KEY_FRAGMENTS: [&str; 6] = [
    "password",
    "token",
    "secret",
    "guest_unique_id",
    "authorization",
    "verifier",
];

/// Logs every request and JSON response with secrets masked and rejects request bodies
/// larger than `MAX_REQUEST_BODY_BYTES` with `413 Payload Too Large`.
pub async fn handle(req: Request, next: Next) -> Response {
    handle_with_body_limit(CONFIG.get_max_request_body_bytes(), req, next).await
}

async fn handle_with_body_limit(max_request_body_bytes: usize, req: Request, next: Next) -> Response {
    let method = req.method().clone();
    let uri = req.uri().clone();
    let headers = describe_headers(req.headers());

    if is_upgrade_request(&req) {
        tracing::info!(%method, %uri, headers = %headers, "http_request (websocket upgrade, body skipped)");
        let response = next.run(req).await;
        tracing::info!(%method, %uri, status = %response.status(), "http_response (websocket upgrade, body skipped)");
        return response;
    }

    let (parts, body) = req.into_parts();
    let Ok(body_bytes) = to_bytes(body, max_request_body_bytes).await else {
        tracing::warn!(%method, %uri, headers = %headers, "http_request rejected, body too large or unreadable");
        return payload_too_large_response();
    };
    tracing::info!(%method, %uri, headers = %headers, body = %describe_body(&body_bytes), "http_request");

    let response = next.run(Request::from_parts(parts, Body::from(body_bytes))).await;
    log_response(&method, &uri, response).await
}

async fn log_response(method: &Method, uri: &Uri, response: Response) -> Response {
    let status = response.status();
    if !is_small_json_response(&response) {
        tracing::info!(%method, %uri, %status, "http_response (body not logged)");
        return response;
    }

    let (parts, body) = response.into_parts();
    let Ok(body_bytes) = to_bytes(body, MAX_LOGGED_RESPONSE_BYTES).await else {
        tracing::error!(%method, %uri, %status, "http_response body could not be read for logging");
        return StatusCode::INTERNAL_SERVER_ERROR.into_response();
    };
    tracing::info!(%method, %uri, %status, body = %describe_body(&body_bytes), "http_response");

    Response::from_parts(parts, Body::from(body_bytes))
}

fn is_small_json_response(response: &Response) -> bool {
    let is_json = response
        .headers()
        .get(CONTENT_TYPE)
        .and_then(|value| value.to_str().ok())
        .is_some_and(|value| value.starts_with("application/json"));
    let fits_in_log_limit = response
        .body()
        .size_hint()
        .upper()
        .is_some_and(|upper| upper <= MAX_LOGGED_RESPONSE_BYTES as u64);

    is_json && fits_in_log_limit
}

fn payload_too_large_response() -> Response {
    (
        StatusCode::PAYLOAD_TOO_LARGE,
        Json(json!(
            ApiResponse::new().with_global_error(errors::PAYLOAD_TOO_LARGE)
        )),
    )
        .into_response()
}

fn is_upgrade_request(req: &Request) -> bool {
    req.headers()
        .get(UPGRADE)
        .and_then(|value| value.to_str().ok())
        .is_some_and(|value| value.eq_ignore_ascii_case("websocket"))
}

/// Renders `headers` for logging with credentials (`authorization`, cookies) fully masked.
fn describe_headers(headers: &HeaderMap) -> String {
    headers
        .iter()
        .map(|(name, value)| {
            let is_credential = name == AUTHORIZATION || name == COOKIE || name == SET_COOKIE;
            if is_credential {
                format!("{name}: {REDACTED_VALUE}")
            } else {
                format!("{name}: {}", value.to_str().unwrap_or("<non-utf8>"))
            }
        })
        .collect::<Vec<_>>()
        .join(", ")
}

/// Renders a body for logging: JSON is printed with sensitive values masked and truncated,
/// anything else is reduced to its size so unknown payloads never reach the log.
fn describe_body(body: &[u8]) -> String {
    if body.is_empty() {
        return String::new();
    }

    match serde_json::from_slice::<Value>(body) {
        Ok(mut value) => {
            redact_sensitive_values(&mut value);
            truncate_for_log(value.to_string())
        }
        Err(_) => format!("<{} bytes, not json>", body.len()),
    }
}

fn redact_sensitive_values(value: &mut Value) {
    match value {
        Value::Object(fields) => {
            for (key, inner) in fields.iter_mut() {
                if is_sensitive_key(key) {
                    *inner = Value::String(REDACTED_VALUE.to_string());
                } else {
                    redact_sensitive_values(inner);
                }
            }
        }
        Value::Array(items) => items.iter_mut().for_each(redact_sensitive_values),
        _ => {}
    }
}

fn is_sensitive_key(key: &str) -> bool {
    let lowered = key.to_ascii_lowercase();
    SENSITIVE_KEY_FRAGMENTS
        .iter()
        .any(|fragment| lowered.contains(fragment))
}

fn truncate_for_log(text: String) -> String {
    if text.chars().count() <= MAX_LOGGED_BODY_CHARS {
        return text;
    }
    let truncated: String = text.chars().take(MAX_LOGGED_BODY_CHARS).collect();
    format!("{truncated}...")
}

#[cfg(test)]
mod tests {
    use axum::{
        body::Body,
        http::{header::HeaderValue, Request as HttpRequest},
        middleware::from_fn,
        routing::post,
        Router,
    };
    use tower::ServiceExt;

    use super::*;

    #[test]
    fn json_bodies_are_logged_with_secrets_masked_at_any_depth() {
        let body = br#"{"email":"a@b.c","password":"hunter2","data":{"token":"jwt.value.here","list":[{"refresh_token":"r"}]},"guest_unique_id":"abc"}"#;

        let logged = describe_body(body);

        assert!(!logged.contains("hunter2"));
        assert!(!logged.contains("jwt.value.here"));
        assert!(!logged.contains("\"r\""));
        assert!(!logged.contains("abc"));
        assert!(logged.contains("a@b.c"));
        assert_eq!(logged.matches(REDACTED_VALUE).count(), 4);
    }

    #[test]
    fn non_json_bodies_are_reduced_to_their_size() {
        assert_eq!(describe_body(b"password=hunter2"), "<16 bytes, not json>");
        assert_eq!(describe_body(b""), "");
    }

    #[test]
    fn long_bodies_are_truncated() {
        let big = format!(r#"{{"note":"{}"}}"#, "x".repeat(5000));

        let logged = describe_body(big.as_bytes());

        assert!(logged.ends_with("..."));
        assert_eq!(logged.chars().count(), MAX_LOGGED_BODY_CHARS + 3);
    }

    #[test]
    fn credential_headers_are_fully_masked() {
        let mut headers = HeaderMap::new();
        headers.insert(AUTHORIZATION, HeaderValue::from_static("Bearer eyJsecret.jwt.value"));
        headers.insert(COOKIE, HeaderValue::from_static("session=abc123"));
        headers.insert("x-app-version", HeaderValue::from_static("1.2.3"));

        let described = describe_headers(&headers);

        assert!(!described.contains("eyJsecret"));
        assert!(!described.contains("abc123"));
        assert!(described.contains("x-app-version: 1.2.3"));
        assert!(described.contains("authorization: <redacted>"));
    }

    fn echo_app(max_request_body_bytes: usize) -> Router {
        Router::new()
            .route("/echo", post(|body: String| async move { body }))
            .layer(from_fn(move |req, next| {
                handle_with_body_limit(max_request_body_bytes, req, next)
            }))
    }

    #[tokio::test]
    async fn oversized_request_bodies_are_rejected_with_413() {
        let request = HttpRequest::post("/echo")
            .body(Body::from("x".repeat(2000)))
            .unwrap();

        let response = echo_app(1024).oneshot(request).await.unwrap();

        assert_eq!(response.status(), StatusCode::PAYLOAD_TOO_LARGE);
    }

    #[tokio::test]
    async fn requests_within_the_limit_reach_the_handler_unchanged() {
        let request = HttpRequest::post("/echo")
            .body(Body::from("hello"))
            .unwrap();

        let response = echo_app(1024).oneshot(request).await.unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        let bytes = to_bytes(response.into_body(), 1024).await.unwrap();
        assert_eq!(&bytes[..], b"hello");
    }
}
