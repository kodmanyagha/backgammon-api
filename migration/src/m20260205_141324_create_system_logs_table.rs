use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(SystemLogs::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(SystemLogs::Id)
                            .big_unsigned()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(SystemLogs::UserId).big_unsigned())
                    .col(string_len(SystemLogs::IpAddress, 45))
                    .col(string_len(SystemLogs::BrowserUserAgent, 1000).null())
                    .col(string_len(SystemLogs::RelatedTableName, 191).null())
                    .col(
                        ColumnDef::new(SystemLogs::RelatedTableId)
                            .big_integer()
                            .null(),
                    )
                    .col(string_len(SystemLogs::LogType, 100))
                    .col(ColumnDef::new(SystemLogs::LogData).text().null())
                    .col(
                        date_time(SystemLogs::CreatedAt)
                            .default(SimpleExpr::Keyword(Keyword::CurrentTimestamp)),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                sea_query::Index::create()
                    .if_not_exists()
                    .name("system_logs_user_id_idx")
                    .table(SystemLogs::Table)
                    .col(SystemLogs::UserId)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                sea_query::Index::create()
                    .if_not_exists()
                    .name("system_logs_log_type_idx")
                    .table(SystemLogs::Table)
                    .col(SystemLogs::LogType)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                sea_query::Index::create()
                    .if_not_exists()
                    .name("system_logs_created_at_idx")
                    .table(SystemLogs::Table)
                    .col(SystemLogs::CreatedAt)
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(SystemLogs::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum SystemLogs {
    Table,
    Id,
    UserId,

    IpAddress,
    BrowserUserAgent,

    RelatedTableName,
    RelatedTableId,

    LogType,
    LogData,

    CreatedAt,
}
