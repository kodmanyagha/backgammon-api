use entity::{
    roles::role_keys,
    utils::{enums::active_passive_status::ActivePassiveStatus, password_helper::hash_password},
};
use sea_orm::{ActiveModelTrait, ActiveValue::Set, ColumnTrait, EntityTrait, QueryFilter};
use sea_orm_migration::SchemaManagerConnection;

pub async fn seed(db_conn: &SchemaManagerConnection<'_>) -> anyhow::Result<()> {
    let emails_with_roles: Vec<(&str, &[&str])> =
        vec![("admin@example.com", &[role_keys::ADMIN, role_keys::USER])];

    for (email, role_keys) in emails_with_roles {
        let existing_user_row = entity::users::Entity::find()
            .filter(entity::users::Column::Email.eq(email))
            .one(db_conn)
            .await?;

        let user_row = match existing_user_row {
            Some(user_row) => user_row,
            None => {
                let user_row = entity::users::ActiveModel {
                    email: Set(Some(email.to_owned())),
                    password: Set(Some(hash_password("1q2w3e4Q!")?)),
                    firstname: Set(Some(email.into())),
                    lastname: Set(Some(email.into())),
                    status: Set(ActivePassiveStatus::Active),
                    ..Default::default()
                };

                let user_row = user_row.insert(db_conn).await?;
                tracing::info!("User {} created, id: {}", email, user_row.id);

                user_row
            }
        };

        for role_key in role_keys {
            assign_role_to_user(db_conn, user_row.id, role_key).await?;
        }
    }

    Ok(())
}

async fn assign_role_to_user(
    db_conn: &SchemaManagerConnection<'_>,
    user_id: u64,
    role_key: &str,
) -> anyhow::Result<()> {
    let Some(role) = entity::roles::Entity::find()
        .filter(entity::roles::Column::Key.eq(role_key))
        .one(db_conn)
        .await?
    else {
        return Err(anyhow::anyhow!("error.role_not_found"));
    };

    let existing_pivot_row = entity::users_roles_pivot::Entity::find()
        .filter(entity::users_roles_pivot::Column::UserId.eq(user_id))
        .filter(entity::users_roles_pivot::Column::RoleId.eq(role.id))
        .one(db_conn)
        .await?;

    if existing_pivot_row.is_some() {
        return Ok(());
    }

    entity::users_roles_pivot::ActiveModel {
        role_id: Set(role.id),
        user_id: Set(user_id),
        ..Default::default()
    }
    .insert(db_conn)
    .await?;

    Ok(())
}
