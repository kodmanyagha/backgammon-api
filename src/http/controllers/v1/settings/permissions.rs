use std::collections::HashMap;

use axum::{
    extract::{Query, State},
    response::{IntoResponse, Response},
    Extension, Json,
};
use sea_orm::{entity::*, Order, PaginatorTrait, QueryFilter, QueryOrder, QuerySelect, QueryTrait};
use serde_json::{json, Value};

use crate::{datatable, state::app_state::AppState, utils::datatable::Datatable};

#[utoipa::path(
    get,
    path = "/v1/settings/permissions",
    tag = "Settings",
    operation_id = "settings_permissions_get_index",
    params(
        ("start" = Option<String>, Query, description = "DataTables pagination offset"),
        ("length" = Option<String>, Query, description = "DataTables page length"),
        ("order[0][dir]" = Option<String>, Query, description = "Sort direction: asc/desc"),
        ("order[0][name]" = Option<String>, Query, description = "Column to sort by"),
        ("search[key]" = Option<String>, Query, description = "Filter by permission key"),
    ),
    responses(
        (status = 200, description = "Paginated list of permissions", body = Datatable<Vec<entity::permissions::Model>>),
    ),
    security(("bearer_auth" = [])),
)]
pub async fn get_index(
    Extension(user): Extension<entity::users::Model>,
    Query(query_input): Query<Value>,
    State(state): State<AppState>,
) -> Result<Response, Response> {
    let filter = entity::permissions::Entity::find().apply_if(
        query_input.get("search[key]"),
        |mut query, value| {
            query.filter(
                entity::permissions::Column::Key
                    .like(format!("%{}%", value.as_str().unwrap_or_default())),
            )
        },
    );

    let mut order_columns = HashMap::new();
    order_columns.insert("id", entity::permissions::Column::Id);
    order_columns.insert("key", entity::permissions::Column::Key);

    let dt = datatable!(
        query_input,
        filter,
        order_columns,
        entity::permissions::Column::Id,
        state
    )?;

    Ok(Json(json!(dt)).into_response())
}
