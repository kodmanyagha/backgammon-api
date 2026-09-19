use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Roles::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(Roles::Id)
                            .big_unsigned()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(Roles::UserId).big_unsigned().null())
                    .col(ColumnDef::new(Roles::Key).string_len(250))
                    .col(ColumnDef::new(Roles::Title).string_len(250))
                    .col(
                        date_time(Roles::CreatedAt)
                            .default(SimpleExpr::Keyword(Keyword::CurrentTimestamp)),
                    )
                    .col(
                        ColumnDef::new(Roles::UpdatedAt)
                            .date_time()
                            .null()
                            .extra("ON UPDATE CURRENT_TIMESTAMP".to_string()),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                sea_query::Index::create()
                    .if_not_exists()
                    .name("roles_user_id_idx")
                    .table(Roles::Table)
                    .col(Roles::UserId)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                sea_query::Index::create()
                    .if_not_exists()
                    .name("roles_key_idx")
                    .table(Roles::Table)
                    .col(Roles::Key)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                sea_query::Index::create()
                    .if_not_exists()
                    .name("roles_created_at_idx")
                    .table(Roles::Table)
                    .col(Roles::CreatedAt)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                sea_query::Index::create()
                    .if_not_exists()
                    .name("roles_updated_at_idx")
                    .table(Roles::Table)
                    .col(Roles::UpdatedAt)
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Roles::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum Roles {
    Table,
    Id,
    UserId,
    Key,
    Title,
    CreatedAt,
    UpdatedAt,
}
