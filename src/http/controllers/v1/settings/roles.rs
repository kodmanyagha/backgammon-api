use std::collections::HashMap;

use axum::{
    extract::{Query, State},
    response::{IntoResponse, Response},
    Extension, Json,
};
use sea_orm::{entity::*, Order, PaginatorTrait, QueryFilter, QueryOrder, QuerySelect, QueryTrait};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use utoipa::{IntoParams, ToSchema};

use crate::{
    state::app_state::AppState,
    utils::{datatable::Datatable, merge_json::merge_struct},
};

#[derive(Serialize, Deserialize, ToSchema, IntoParams)]
struct HandleGetQueryInput {
    #[serde(rename = "search[value]", default)]
    pub search_value: Option<String>,

    #[serde(rename = "search[user]", default)]
    pub user: Option<String>,

    #[serde(rename = "search[domain]", default)]
    pub domain: Option<String>,

    #[serde(rename = "search[title]", default)]
    pub title: Option<String>,

    #[serde(rename = "order[0][dir]", default)]
    pub order_dir: String,
    #[serde(rename = "order[0][name]", default)]
    pub order_column: String,

    #[serde(deserialize_with = "serde_aux::field_attributes::deserialize_number_from_string")]
    pub start: u64,
    #[serde(deserialize_with = "serde_aux::field_attributes::deserialize_number_from_string")]
    pub length: u64,
}

#[utoipa::path(
    get,
    path = "/v1/settings/roles",
    tag = "Settings",
    operation_id = "settings_roles_get_index",
    params(HandleGetQueryInput),
    responses(
        (status = 200, description = "Paginated list of roles", body = Datatable<Value>),
        (status = 400, description = "Query error"),
    ),
    security(("bearer_auth" = [])),
)]
pub async fn get_index(
    Extension(_user): Extension<entity::users::Model>,
    Query(query_input): Query<Value>,
    State(state): State<AppState>,
) -> Result<Response, Response> {
    tracing::info!("Input: {:?}", query_input);

    let query_input: HandleGetQueryInput =
        serde_json::from_value(query_input).map_err(|err| format!("Err: {err}").into_response())?;

    let filter = entity::roles::Entity::find()
        .apply_if(query_input.title.clone(), |query, value| {
            query.filter(entity::roles::Column::Key.like(format!("%{}%", value)))
        });

    let mut order_columns = HashMap::new();

    order_columns.insert("id", entity::roles::Column::Id);
    order_columns.insert("key", entity::roles::Column::Key);

    let dt = {
        let count = filter
            .clone()
            .count(&*state.db_conn)
            .await
            .map_err(|err| format!("Query error: {}", err).into_response())?;

        let order_dir = match query_input.order_dir.clone().as_str() {
            "asc" => Order::Asc,
            _ => Order::Desc,
        };

        let order_column = if order_columns.contains_key(query_input.order_column.as_str()) {
            *order_columns
                .get(&query_input.order_column.as_str())
                .unwrap()
        } else {
            entity::roles::Column::Id
        };

        let rows = filter
            .clone()
            .order_by(order_column, order_dir)
            .offset(Some(query_input.start))
            .limit(query_input.length)
            .all(&*state.db_conn)
            .await
            .map_err(|err| format!("Query error: {}", err).into_response())?;

        crate::utils::datatable::Datatable::new(count, rows)
    };

    let dt = Datatable::new(
        dt.records_filtered,
        dt.data
            .iter()
            .map(|row| {
                merge_struct(
                    &json!(row),
                    &json!({
                        "user_email": "foo",
                        "doamin": "foo"
                    }),
                )
                .unwrap_or(json!({}))
            })
            .collect::<Value>(),
    );

    Ok(Json(json!(dt)).into_response())
}
