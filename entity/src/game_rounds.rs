use sea_orm::{entity::prelude::*, sqlx::types::chrono};
use serde::{Deserialize, Serialize};

#[derive(Clone, Default, Debug, PartialEq, Eq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "game_rounds")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: u64,
    pub game_id: u64,
    pub round_no: u16,
    pub winner_user_id: u64,
    pub white_score: u16,
    pub black_score: u16,

    pub created_at: chrono::NaiveDateTime,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
