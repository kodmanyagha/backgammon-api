use std::fmt::Debug;

use axum::{http::StatusCode, response::IntoResponse, Json};
use serde::{Deserialize, Serialize};
use serde_json::json;
use utoipa::ToSchema;

#[derive(Clone, Serialize, Deserialize, Debug, ToSchema)]
pub struct Datatable<T>
where
    T: Serialize + Debug + Clone,
{
    #[serde(rename = "recordsTotal")]
    pub records_total: u64,

    #[serde(rename = "recordsFiltered")]
    pub records_filtered: u64,

    pub data: T,
}

impl<T> Datatable<T>
where
    T: Serialize + Debug + Clone,
{
    pub fn new(total: u64, data: T) -> Self {
        Self {
            records_total: total,
            records_filtered: total,
            data,
        }
    }
}

impl<T> IntoResponse for Datatable<T>
where
    T: Serialize + Debug + Clone,
{
    fn into_response(self) -> axum::response::Response {
        (StatusCode::OK, Json(json!(self))).into_response()
    }
}

#[macro_export]
macro_rules! datatable {
    ($query_input:expr, $filter: expr, $order_columns: expr, $default_col: expr, $state: expr) => {{
        use sea_orm::{PaginatorTrait, QueryOrder, QuerySelect};

        let input_start = $query_input["start"]
            .as_str()
            .unwrap_or("0")
            .parse::<u64>()
            .unwrap_or(0);

        let input_length = $query_input["length"]
            .as_str()
            .unwrap_or("5")
            .parse::<u64>()
            .unwrap_or(5)
            .clamp(5, 1000);

        let count = $filter
            .clone()
            .count(&*$state.db_conn)
            .await
            .map_err(|err| format!("Query error: {}", err).into_response())?;

        let order_dir = match $query_input["order[0][dir]"].as_str().unwrap_or("asc") {
            "asc" => sea_orm::Order::Asc,
            _ => sea_orm::Order::Desc,
        };

        let input_order_col = $query_input["order[0][name]"].as_str().unwrap_or_default();
        let order_column = if $order_columns.get(input_order_col).is_some() {
            $order_columns
                .get(input_order_col)
                .ok_or("Unknown order colur specified".into_response())?
                .clone()
        } else {
            $default_col
        };

        let rows = $filter
            .clone()
            .order_by(order_column, order_dir)
            .offset(Some(input_start))
            .limit(input_length)
            .all(&*$state.db_conn)
            .await
            .map_err(|err| format!("Query error: {}", err).into_response())?;

        Ok($crate::utils::datatable::Datatable::new(count, rows))
    }};
}

#[macro_export]
macro_rules! map_dt_data_async {
    ($dt:expr, $mapper: expr) => {{
        use serde_json::Value;

        let mut data = Vec::<Value>::new();
        for item in &$dt.data {
            data.push($mapper(item).await);
        }

        let dt = $crate::utils::datatable::Datatable {
            records_filtered: $dt.records_filtered,
            records_total: $dt.records_total,
            data,
        };
        dt
    }};
}
