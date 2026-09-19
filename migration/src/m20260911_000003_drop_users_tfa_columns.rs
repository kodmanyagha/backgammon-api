use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table(Users::Table)
                    .drop_column(Users::TfaMethod)
                    .drop_column(Users::TfaKey)
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table(Users::Table)
                    .add_column(
                        ColumnDef::new(Users::TfaMethod)
                            .enumeration(
                                Alias::new("users_tfa_method"),
                                [
                                    Alias::new("none"),
                                    Alias::new("google_auth"),
                                    Alias::new("mail"),
                                    Alias::new("sms"),
                                    Alias::new("whatsapp"),
                                    Alias::new("telegram"),
                                ],
                            )
                            .default("none")
                            .not_null(),
                    )
                    .add_column(ColumnDef::new(Users::TfaKey).string_len(256).null())
                    .to_owned(),
            )
            .await
    }
}

#[derive(DeriveIden)]
enum Users {
    Table,
    TfaMethod,
    TfaKey,
}
