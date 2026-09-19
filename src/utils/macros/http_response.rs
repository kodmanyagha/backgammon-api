/// Generates an error JSON response body.
///
/// # Arguments
/// - `$global`: expression coercible to `String` — the top-level `error._global` message.
/// - (optional) `$key => $value` pairs: additional fields nested inside `error`.
///
/// # Example
/// ```
/// let body = error_response!("Something went wrong");
/// let body = error_response!("Validation failed", "email" => "already taken", "name" => "too short");
/// ```
macro_rules! error_response {
    ($global:expr) => {
        serde_json::json!({
            "status": "error",
            "error": {
                "_global": format!("{}", $global)
            }
        })
    };

    ($global:expr, $($key:expr => $value:expr),+ $(,)?) => {
        {
            let mut __error = serde_json::json!({
                "_global": format!("{}", $global)
            });
            $(
                __error[$key] = serde_json::json!($value);
            )+
            serde_json::json!({ "status": "error", "error": __error })
        }
    };
}

/// Generates a success JSON response body where each variable name becomes a JSON key.
///
/// # Example
/// ```
/// let user = ...;
/// let jwt = "token";
/// let body = success_response!(user, jwt);
/// // => { "status": "success", "data": { "user": <user>, "jwt": "token" } }
/// ```
macro_rules! success_response {
    ($($var:ident),* $(,)?) => {
        serde_json::json!({
            "status": "success",
            "data": {
                $(stringify!($var): $var,)+
            }
        })
    };
}

pub(crate) use error_response;
pub(crate) use success_response;
