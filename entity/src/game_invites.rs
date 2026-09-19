use sea_orm::{entity::prelude::*, sqlx::types::chrono};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(
    Clone, Default, Debug, PartialEq, Eq, DeriveEntityModel, Serialize, Deserialize,
)]
#[sea_orm(table_name = "game_invites")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: u64,
    pub token: String,
    pub created_by_user_id: u64,
    pub game_id: Option<u64>,

    pub status: GameInviteStatus,

    pub created_at: chrono::NaiveDateTime,
    pub expires_at: Option<chrono::NaiveDateTime>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}

#[derive(Clone, Debug, PartialEq, Eq, EnumIter, DeriveActiveEnum, Serialize, Deserialize, Default, ToSchema)]
#[sea_orm(rs_type = "String", db_type = "Enum", rename_all = "snake_case")]
#[serde(rename_all = "snake_case")]
pub enum GameInviteStatus {
    #[default]
    #[sea_orm(string_value = "pending")]
    Pending,
    #[sea_orm(string_value = "accepted")]
    Accepted,
    #[sea_orm(string_value = "expired")]
    Expired,
    #[sea_orm(string_value = "cancelled")]
    Cancelled,
}
