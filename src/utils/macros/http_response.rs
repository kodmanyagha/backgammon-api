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
