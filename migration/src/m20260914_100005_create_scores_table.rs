use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Scores::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(Scores::Id)
                            .big_unsigned()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(Scores::UserId).big_unsigned().not_null())
                    .col(
                        ColumnDef::new(Scores::Wins)
                            .unsigned()
                            .not_null()
                            .default(0),
                    )
                    .col(
                        ColumnDef::new(Scores::Losses)
                            .unsigned()
                            .not_null()
                            .default(0),
                    )
                    .col(
                        ColumnDef::new(Scores::UpdatedAt)
                            .date_time()
                            .null()
                            .extra("ON UPDATE CURRENT_TIMESTAMP".to_string())
                            .default(SimpleExpr::Keyword(Keyword::CurrentTimestamp)),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                sea_query::Index::create()
                    .if_not_exists()
                    .name("scores_user_id_idx")
                    .table(Scores::Table)
                    .col(Scores::UserId)
                    .unique()
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Scores::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum Scores {
    Table,
    Id,
    UserId,
    Wins,
    Losses,
    UpdatedAt,
}
