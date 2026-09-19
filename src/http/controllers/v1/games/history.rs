use std::collections::HashMap;

use axum::{
    extract::{Query, State},
    response::{IntoResponse, Response},
    Extension, Json,
};
use sea_orm::{entity::*, Condition, Order, PaginatorTrait, QueryFilter, QueryOrder, QuerySelect};
use serde_json::{json, Value};

use crate::{
    datatable, map_dt_data_async,
    service::game::get_history,
    state::app_state::AppState,
    utils::datatable::Datatable,
};

#[utoipa::path(
    get,
    path = "/v1/games/history",
    tag = "Games",
    operation_id = "games_get_history",
    params(
        ("start" = Option<String>, Query, description = "DataTables pagination offset"),
        ("length" = Option<String>, Query, description = "DataTables page length"),
        ("order[0][dir]" = Option<String>, Query, description = "Sort direction: asc/desc"),
        ("order[0][name]" = Option<String>, Query, description = "Column to sort by (id, created_at, status)"),
    ),
    responses(
        (status = 200, description = "Caller's past and ongoing games, most recent first", body = Datatable<Vec<get_history::GameHistoryItemDto>>),
        (status = 400, description = "Query error"),
    ),
    security(("bearer_auth" = [])),
)]
pub async fn get_history(
    State(state): State<AppState>,
    Extension(user): Extension<entity::users::Model>,
    Query(query_input): Query<Value>,
) -> Result<Response, Response> {
    let filter = entity::games::Entity::find().filter(
        Condition::any()
            .add(entity::games::Column::GoldUserId.eq(user.id))
            .add(entity::games::Column::PurpleUserId.eq(user.id)),
    );

    let mut order_columns = HashMap::new();
    order_columns.insert("id", entity::games::Column::Id);
    order_columns.insert("created_at", entity::games::Column::CreatedAt);
    order_columns.insert("status", entity::games::Column::Status);

    let dt = datatable!(
        query_input,
        filter,
        order_columns,
        entity::games::Column::CreatedAt,
        state
    )?;

    let mapper = |game: &entity::games::Model| {
        let game = game.clone();
        let state = state.clone();
        let user_id = user.id;
        async move { get_history::to_dto(&state, user_id, &game).await }
    };
    let dt = map_dt_data_async!(dt, mapper);

    Ok(Json(json!(dt)).into_response())
}
