use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(RolesPermissionsPivot::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(RolesPermissionsPivot::Id)
                            .big_unsigned()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(RolesPermissionsPivot::RoleId).big_unsigned())
                    .col(ColumnDef::new(RolesPermissionsPivot::PermissionId).big_unsigned())
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                sea_query::Index::create()
                    .if_not_exists()
                    .name("roles_permissions_pivot_role_id_idx")
                    .table(RolesPermissionsPivot::Table)
                    .col(RolesPermissionsPivot::RoleId)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                sea_query::Index::create()
                    .if_not_exists()
                    .name("roles_permissions_pivot_permission_id_idx")
                    .table(RolesPermissionsPivot::Table)
                    .col(RolesPermissionsPivot::PermissionId)
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(RolesPermissionsPivot::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum RolesPermissionsPivot {
    Table,
    Id,
    RoleId,
    PermissionId,
}
