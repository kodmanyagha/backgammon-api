use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(GameMoves::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(GameMoves::Id)
                            .big_unsigned()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(GameMoves::GameId).big_unsigned().not_null())
                    .col(ColumnDef::new(GameMoves::RoundNo).small_unsigned().not_null().default(1))
                    .col(
                        ColumnDef::new(GameMoves::SequenceNo)
                            .unsigned()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(GameMoves::Player)
                            .enumeration(
                                Alias::new("game_moves_player"),
                                [Alias::new("white"), Alias::new("black")],
                            )
                            .not_null(),
                    )
                    .col(ColumnDef::new(GameMoves::OriginPoint).tiny_unsigned().null())
                    .col(ColumnDef::new(GameMoves::Die).tiny_unsigned().not_null())
                    .col(ColumnDef::new(GameMoves::IsAi).boolean().not_null().default(false))
                    .col(
                        date_time(GameMoves::CreatedAt)
                            .default(SimpleExpr::Keyword(Keyword::CurrentTimestamp)),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                sea_query::Index::create()
                    .if_not_exists()
                    .name("game_moves_game_id_sequence_no_idx")
                    .table(GameMoves::Table)
                    .col(GameMoves::GameId)
                    .col(GameMoves::SequenceNo)
                    .unique()
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(GameMoves::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum GameMoves {
    Table,
    Id,
    GameId,
    RoundNo,
    SequenceNo,
    Player,
    OriginPoint,
    Die,
    IsAi,
    CreatedAt,
}
