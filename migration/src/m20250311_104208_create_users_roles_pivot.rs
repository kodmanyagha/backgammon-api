use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(UsersRolesPivot::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(UsersRolesPivot::Id)
                            .big_unsigned()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(UsersRolesPivot::UserId).big_unsigned())
                    .col(ColumnDef::new(UsersRolesPivot::RoleId).big_unsigned())
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                sea_query::Index::create()
                    .if_not_exists()
                    .name("users_roles_pivot_user_id_idx")
                    .table(UsersRolesPivot::Table)
                    .col(UsersRolesPivot::UserId)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                sea_query::Index::create()
                    .if_not_exists()
                    .name("users_roles_pivot_role_id_idx")
                    .table(UsersRolesPivot::Table)
                    .col(UsersRolesPivot::RoleId)
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(UsersRolesPivot::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum UsersRolesPivot {
    Table,
    Id,
    UserId,
    RoleId,
}
