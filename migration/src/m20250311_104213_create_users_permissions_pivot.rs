use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(UsersPermissionsPivot::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(UsersPermissionsPivot::Id)
                            .big_unsigned()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(UsersPermissionsPivot::UserId).big_unsigned())
                    .col(ColumnDef::new(UsersPermissionsPivot::PermissionId).big_unsigned())
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                sea_query::Index::create()
                    .if_not_exists()
                    .name("users_permissions_pivot_user_id_idx")
                    .table(UsersPermissionsPivot::Table)
                    .col(UsersPermissionsPivot::UserId)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                sea_query::Index::create()
                    .if_not_exists()
                    .name("users_permissions_pivot_permission_id_idx")
                    .table(UsersPermissionsPivot::Table)
                    .col(UsersPermissionsPivot::PermissionId)
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(UsersPermissionsPivot::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum UsersPermissionsPivot {
    Table,
    Id,
    UserId,
    PermissionId,
}
