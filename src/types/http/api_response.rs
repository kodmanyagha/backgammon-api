use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use serde_json::Value;
use utoipa::ToSchema;

type ErrorMap = HashMap<String, String>;

#[derive(Clone, Serialize, Deserialize, Debug, Default, ToSchema)]
pub struct ApiResponse {
    error: Option<ErrorMap>,
    data: Option<Value>,
}

impl ApiResponse {
    pub fn new() -> Self {
        ApiResponse {
            error: None,
            data: None,
        }
    }

    pub fn set_data(&mut self, data: Value) {
        self.data = Some(data);
    }

    pub fn set_error(&mut self, error: ErrorMap) {
        self.error = Some(error);
    }

    pub fn with_data(mut self, data: Value) -> Self {
        self.data = Some(data);
        self
    }

    pub fn with_error(mut self, error: ErrorMap) -> Self {
        self.error = Some(error);
        self
    }

    pub fn with_global_error(mut self, error_message: &str) -> Self {
        let mut err_map = self.error.unwrap_or_default();
        err_map.insert("_global".into(), error_message.into());

        self.error = Some(err_map);
        self
    }
}
