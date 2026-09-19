use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Users::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(Users::Id)
                            .big_unsigned()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(Users::ParentId).big_unsigned().null())
                    .col(
                        ColumnDef::new(Users::ReferencedUserId)
                            .big_unsigned()
                            .null(),
                    )
                    .col(
                        ColumnDef::new(Users::Email)
                            .string_len(191)
                            .unique_key()
                            .not_null(),
                    )
                    .col(ColumnDef::new(Users::Password).string_len(1000))
                    .col(ColumnDef::new(Users::Firstname).string_len(1000).null())
                    .col(ColumnDef::new(Users::Lastname).string_len(1000).null())
                    .col(
                        ColumnDef::new(Users::TfaMethod)
                            .enumeration(
                                Alias::new("users_tfa_method"),
                                [
                                    Alias::new("none"),
                                    Alias::new("google_auth"),
                                    Alias::new("mail"),
                                    Alias::new("sms"),
                                    Alias::new("whatsapp"),
                                    Alias::new("telegram"),
                                ],
                            )
                            .default("none")
                            .not_null(),
                    )
                    .col(ColumnDef::new(Users::TfaKey).string_len(256).null())
                    .col(
                        ColumnDef::new(Users::Status)
                            .enumeration(
                                Alias::new("users_status"),
                                [Alias::new("active"), Alias::new("passive")],
                            )
                            .default("active")
                            .not_null(),
                    )
                    .col(
                        date_time(Users::CreatedAt)
                            .default(SimpleExpr::Keyword(Keyword::CurrentTimestamp)),
                    )
                    .col(
                        ColumnDef::new(Users::UpdatedAt)
                            .date_time()
                            .null()
                            .extra("ON UPDATE CURRENT_TIMESTAMP".to_string()),
                    )
                    .col(date_time_null(Users::DeletedAt))
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                sea_query::Index::create()
                    .if_not_exists()
                    .name("users_parent_id_idx")
                    .table(Users::Table)
                    .col(Users::ParentId)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                sea_query::Index::create()
                    .if_not_exists()
                    .name("users_referenced_user_id_idx")
                    .table(Users::Table)
                    .col(Users::ReferencedUserId)
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Users::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum Users {
    Table,
    Id,
    ParentId,
    ReferencedUserId,
    Email,
    Password,
    Firstname,
    Lastname,

    TfaMethod,
    TfaKey,

    Status,

    CreatedAt,
    UpdatedAt,
    DeletedAt,
}
