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
                    .modify_column(ColumnDef::new(Users::Email).string_len(191).null())
                    .modify_column(ColumnDef::new(Users::Password).string_len(1000).null())
                    .add_column(
                        ColumnDef::new(Users::IsGuest)
                            .boolean()
                            .not_null()
                            .default(false),
                    )
                    .add_column(ColumnDef::new(Users::Username).string_len(191).null())
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                sea_query::Index::create()
                    .if_not_exists()
                    .name("users_username_idx")
                    .table(Users::Table)
                    .col(Users::Username)
                    .unique()
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_index(
                sea_query::Index::drop()
                    .name("users_username_idx")
                    .table(Users::Table)
                    .to_owned(),
            )
            .await?;

        manager
            .alter_table(
                Table::alter()
                    .table(Users::Table)
                    .drop_column(Users::IsGuest)
                    .drop_column(Users::Username)
                    .modify_column(
                        ColumnDef::new(Users::Email)
                            .string_len(191)
                            .not_null(),
                    )
                    .modify_column(ColumnDef::new(Users::Password).string_len(1000).not_null())
                    .to_owned(),
            )
            .await
    }
}

#[derive(DeriveIden)]
enum Users {
    Table,
    Email,
    Password,
    IsGuest,
    Username,
}
