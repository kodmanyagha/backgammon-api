use sea_orm::{entity::prelude::*, sqlx::types::chrono};
use serde::{Deserialize, Serialize};

use utoipa::ToSchema;

use crate::utils::enums::active_passive_status::ActivePassiveStatus;

#[derive(
    Clone, Default, Debug, PartialEq, Eq, DeriveEntityModel, Serialize, Deserialize, ToSchema,
)]
#[sea_orm(table_name = "users")]
#[schema(as = entity::users::Model)]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: u64,
    pub parent_id: Option<u64>,
    pub email: Option<String>,

    #[serde(skip_serializing)]
    pub password: Option<String>,

    pub firstname: Option<String>,
    pub lastname: Option<String>,

    pub is_guest: bool,
    pub username: Option<String>,

    #[serde(skip_serializing)]
    pub guest_unique_id: Option<String>,

    pub status: ActivePassiveStatus,

    pub created_at: chrono::NaiveDateTime,
    #[sea_orm(default_expr = "Expr::current_timestamp()")]
    pub updated_at: Option<chrono::NaiveDateTime>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
