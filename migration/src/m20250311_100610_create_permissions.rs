use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Permissions::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(Permissions::Id)
                            .big_unsigned()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(
                        ColumnDef::new(Permissions::Key)
                            .string_len(250)
                            .unique_key(),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                sea_query::Index::create()
                    .if_not_exists()
                    .name("permissions_key_idx")
                    .table(Permissions::Table)
                    .col(Permissions::Key)
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Permissions::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum Permissions {
    Table,
    Id,
    Key,
}
