use sea_orm::{entity::prelude::*, sqlx::types::chrono};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(
    Clone, Default, Debug, PartialEq, Eq, DeriveEntityModel, Serialize, Deserialize, ToSchema,
)]
#[sea_orm(table_name = "user_throttles")]
#[schema(as = entity::user_throttles::Model)]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: u64,
    pub user_id: u64,
    pub requests_per_second: u64,

    pub created_at: chrono::NaiveDateTime,
    #[sea_orm(default_expr = "Expr::current_timestamp()")]
    pub updated_at: Option<chrono::NaiveDateTime>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
