use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(
    Clone, Default, Debug, PartialEq, Eq, DeriveEntityModel, Serialize, Deserialize, ToSchema,
)]
#[sea_orm(table_name = "permissions")]
#[schema(as = entity::permissions::Model)]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: u64,
    pub key: String,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}

pub mod perm_keys {
    pub const USER: &str = "user";
    pub const SYSTEM_MANAGER: &str = "system_manager";
}
