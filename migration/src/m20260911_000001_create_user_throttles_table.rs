use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(UserThrottles::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(UserThrottles::Id)
                            .big_unsigned()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(
                        ColumnDef::new(UserThrottles::UserId)
                            .big_unsigned()
                            .not_null()
                            .unique_key(),
                    )
                    .col(
                        ColumnDef::new(UserThrottles::RequestsPerSecond)
                            .big_unsigned()
                            .not_null(),
                    )
                    .col(
                        date_time(UserThrottles::CreatedAt)
                            .default(SimpleExpr::Keyword(Keyword::CurrentTimestamp)),
                    )
                    .col(
                        ColumnDef::new(UserThrottles::UpdatedAt)
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
                    .name("user_throttles_user_id_idx")
                    .table(UserThrottles::Table)
                    .col(UserThrottles::UserId)
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(UserThrottles::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum UserThrottles {
    Table,
    Id,
    UserId,
    RequestsPerSecond,
    CreatedAt,
    UpdatedAt,
}
