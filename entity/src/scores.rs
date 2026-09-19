use sea_orm::{entity::prelude::*, sqlx::types::chrono};
use serde::{Deserialize, Serialize};

#[derive(
    Clone, Default, Debug, PartialEq, Eq, DeriveEntityModel, Serialize, Deserialize,
)]
#[sea_orm(table_name = "scores")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: u64,
    pub user_id: u64,
    pub wins: u32,
    pub losses: u32,
    #[sea_orm(default_expr = "Expr::current_timestamp()")]
    pub updated_at: Option<chrono::NaiveDateTime>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
