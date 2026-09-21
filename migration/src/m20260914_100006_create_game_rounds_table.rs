use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(GameRounds::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(GameRounds::Id)
                            .big_unsigned()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(GameRounds::GameId).big_unsigned().not_null())
                    .col(ColumnDef::new(GameRounds::RoundNo).small_unsigned().not_null())
                    .col(ColumnDef::new(GameRounds::WinnerUserId).big_unsigned().not_null())
                    .col(ColumnDef::new(GameRounds::WhiteScore).small_unsigned().not_null())
                    .col(ColumnDef::new(GameRounds::BlackScore).small_unsigned().not_null())
                    .col(
                        ColumnDef::new(GameRounds::CreatedAt)
                            .date_time()
                            .not_null()
                            .default(SimpleExpr::Keyword(Keyword::CurrentTimestamp)),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                sea_query::Index::create()
                    .if_not_exists()
                    .name("game_rounds_game_id_round_no_idx")
                    .table(GameRounds::Table)
                    .col(GameRounds::GameId)
                    .col(GameRounds::RoundNo)
                    .unique()
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(GameRounds::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum GameRounds {
    Table,
    Id,
    GameId,
    RoundNo,
    WinnerUserId,
    WhiteScore,
    BlackScore,
    CreatedAt,
}
