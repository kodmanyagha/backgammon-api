use sea_orm::{entity::prelude::*, sqlx::types::chrono};
use serde::{Deserialize, Serialize};

#[derive(Clone, Default, Debug, PartialEq, Eq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "system_logs")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: u64,
    pub user_id: u64,
    pub ip_address: String,
    pub browser_user_agent: String,
    pub related_table_name: String,
    pub related_table_id: u64,
    pub log_type: String,
    pub log_data: String,
    pub created_at: chrono::NaiveDateTime,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
