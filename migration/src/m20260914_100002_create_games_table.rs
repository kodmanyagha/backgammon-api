use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Games::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(Games::Id)
                            .big_unsigned()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(Games::GoldUserId).big_unsigned().not_null())
                    .col(
                        ColumnDef::new(Games::PurpleUserId)
                            .big_unsigned()
                            .not_null(),
                    )
                    .col(ColumnDef::new(Games::WinnerUserId).big_unsigned().null())
                    .col(
                        ColumnDef::new(Games::Status)
                            .enumeration(
                                Alias::new("games_status"),
                                [
                                    Alias::new("waiting"),
                                    Alias::new("active"),
                                    Alias::new("finished"),
                                    Alias::new("abandoned"),
                                ],
                            )
                            .default("waiting")
                            .not_null(),
                    )
                    .col(
                        date_time(Games::CreatedAt)
                            .default(SimpleExpr::Keyword(Keyword::CurrentTimestamp)),
                    )
                    .col(date_time_null(Games::StartedAt))
                    .col(date_time_null(Games::FinishedAt))
                    .to_owned(),
            )
            .await?;

        for (name, col) in [
            ("games_gold_user_id_idx", Games::GoldUserId),
            ("games_purple_user_id_idx", Games::PurpleUserId),
            ("games_winner_user_id_idx", Games::WinnerUserId),
            ("games_status_idx", Games::Status),
        ] {
            manager
                .create_index(
                    sea_query::Index::create()
                        .if_not_exists()
                        .name(name)
                        .table(Games::Table)
                        .col(col)
                        .to_owned(),
                )
                .await?;
        }

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Games::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum Games {
    Table,
    Id,
    GoldUserId,
    PurpleUserId,
    WinnerUserId,
    Status,
    CreatedAt,
    StartedAt,
    FinishedAt,
}
