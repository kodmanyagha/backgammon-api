use sea_orm::{entity::prelude::*, sqlx::types::chrono};
use serde::{Deserialize, Serialize};

#[derive(Clone, Default, Debug, PartialEq, Eq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "roles")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: u64,
    pub user_id: Option<u64>,
    pub key: String,
    pub created_at: chrono::NaiveDateTime,
    #[sea_orm(default_expr = "Expr::current_timestamp()")]
    pub updated_at: Option<chrono::NaiveDateTime>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}

pub mod role_keys {
    pub const ADMIN: &str = "admin";
    pub const USER: &str = "user";
    pub const GUEST: &str = "guest";
}
