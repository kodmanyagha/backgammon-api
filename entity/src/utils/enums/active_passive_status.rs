use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(
    Default,
    Debug,
    Clone,
    PartialEq,
    Eq,
    EnumIter,
    DeriveActiveEnum,
    Serialize,
    Deserialize,
    strum::EnumString,
    strum::IntoStaticStr,
    ToSchema,
)]
#[strum(serialize_all = "snake_case")]
#[sea_orm(rs_type = "String", db_type = "Enum", rename_all = "snake_case")]
#[serde(rename_all = "snake_case")]
pub enum ActivePassiveStatus {
    #[default]
    #[sea_orm(string_value = "active")]
    Active,

    #[sea_orm(string_value = "passive")]
    Passive,
}
