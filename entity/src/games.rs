use sea_orm::{entity::prelude::*, sqlx::types::chrono};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(
    Clone, Default, Debug, PartialEq, Eq, DeriveEntityModel, Serialize, Deserialize, ToSchema,
)]
#[sea_orm(table_name = "games")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: u64,
    pub gold_user_id: u64,
    pub purple_user_id: u64,
    pub winner_user_id: Option<u64>,

    pub status: GameStatus,

    pub created_at: chrono::NaiveDateTime,
    pub started_at: Option<chrono::NaiveDateTime>,
    pub finished_at: Option<chrono::NaiveDateTime>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}

#[derive(Clone, Debug, PartialEq, Eq, EnumIter, DeriveActiveEnum, Serialize, Deserialize, Default, ToSchema)]
#[sea_orm(rs_type = "String", db_type = "Enum", rename_all = "snake_case")]
#[serde(rename_all = "snake_case")]
pub enum GameStatus {
    #[default]
    #[sea_orm(string_value = "waiting")]
    Waiting,
    #[sea_orm(string_value = "active")]
    Active,
    #[sea_orm(string_value = "finished")]
    Finished,
    #[sea_orm(string_value = "abandoned")]
    Abandoned,
}
