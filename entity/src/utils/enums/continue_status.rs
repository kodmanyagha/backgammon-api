use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

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
)]
#[strum(serialize_all = "snake_case")]
#[sea_orm(rs_type = "String", db_type = "Enum", rename_all = "snake_case")]
#[serde(rename_all = "snake_case")]
pub enum ContinueStatus {
    #[default]
    #[sea_orm(string_value = "continue")]
    Continue,

    #[sea_orm(string_value = "finish")]
    Finish,
}
