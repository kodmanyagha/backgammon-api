use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(GameInvites::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(GameInvites::Id)
                            .big_unsigned()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(
                        ColumnDef::new(GameInvites::Token)
                            .string_len(64)
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(GameInvites::CreatedByUserId)
                            .big_unsigned()
                            .not_null(),
                    )
                    .col(ColumnDef::new(GameInvites::GameId).big_unsigned().null())
                    .col(
                        ColumnDef::new(GameInvites::Status)
                            .enumeration(
                                Alias::new("game_invites_status"),
                                [
                                    Alias::new("pending"),
                                    Alias::new("accepted"),
                                    Alias::new("expired"),
                                    Alias::new("cancelled"),
                                ],
                            )
                            .default("pending")
                            .not_null(),
                    )
                    .col(
                        date_time(GameInvites::CreatedAt)
                            .default(SimpleExpr::Keyword(Keyword::CurrentTimestamp)),
                    )
                    .col(date_time_null(GameInvites::ExpiresAt))
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                sea_query::Index::create()
                    .if_not_exists()
                    .name("game_invites_token_idx")
                    .table(GameInvites::Table)
                    .col(GameInvites::Token)
                    .unique()
                    .to_owned(),
            )
            .await?;

        for (name, col) in [
            (
                "game_invites_created_by_user_id_idx",
                GameInvites::CreatedByUserId,
            ),
            ("game_invites_status_idx", GameInvites::Status),
        ] {
            manager
                .create_index(
                    sea_query::Index::create()
                        .if_not_exists()
                        .name(name)
                        .table(GameInvites::Table)
                        .col(col)
                        .to_owned(),
                )
                .await?;
        }

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(GameInvites::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum GameInvites {
    Table,
    Id,
    Token,
    CreatedByUserId,
    GameId,
    Status,
    CreatedAt,
    ExpiresAt,
}
