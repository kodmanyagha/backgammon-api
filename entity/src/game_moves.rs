use sea_orm::{entity::prelude::*, sqlx::types::chrono};
use serde::{Deserialize, Serialize};

#[derive(
    Clone, Default, Debug, PartialEq, Eq, DeriveEntityModel, Serialize, Deserialize,
)]
#[sea_orm(table_name = "game_moves")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: u64,
    pub game_id: u64,
    pub round_no: u16,
    pub sequence_no: u32,

    pub player: GameMovePlayer,

    pub origin_point: Option<u8>,
    pub die: u8,
    pub is_ai: bool,

    pub created_at: chrono::NaiveDateTime,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}

#[derive(Clone, Debug, PartialEq, Eq, EnumIter, DeriveActiveEnum, Serialize, Deserialize, Default)]
#[sea_orm(rs_type = "String", db_type = "Enum", rename_all = "snake_case")]
#[serde(rename_all = "snake_case")]
pub enum GameMovePlayer {
    #[default]
    #[sea_orm(string_value = "white")]
    White,
    #[sea_orm(string_value = "black")]
    Black,
}
