use sea_orm::DeriveActiveEnum;
use sea_orm_migration::sea_orm::EnumIter;

#[derive(EnumIter, DeriveActiveEnum)]
#[sea_orm(
    rs_type = "String",
    db_type = "Enum",
    rename_all = "snake_case",
    enum_name = "status"
)]
pub enum ActivePassiveStatus {
    #[sea_orm(string_value = "active")]
    Active,

    #[sea_orm(string_value = "passive")]
    Passive,
}
