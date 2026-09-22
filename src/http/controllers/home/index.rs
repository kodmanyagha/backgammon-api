use axum::{
    extract::State,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;

use crate::state::app_state::AppState;

#[utoipa::path(
    get,
    path = "/",
    tag = "Home",
    operation_id = "home_get_index",
    responses(
        (status = 200, description = "Server info and current time"),
    ),
)]
pub async fn get_index(State(state): State<AppState>) -> Result<Response, Response> {
    let datetime = chrono::Local::now();

    let ret_data = json!({
        "api": "backgammon-api",
        "datetime": datetime,
        "datetime_utc": datetime.naive_utc(),
    });

    Ok(Json(json!(ret_data)).into_response())
}
