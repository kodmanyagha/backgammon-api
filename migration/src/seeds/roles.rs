use entity::roles;
use sea_orm::{ActiveModelTrait, ActiveValue::Set, ColumnTrait, EntityTrait, QueryFilter};
use sea_orm_migration::SchemaManagerConnection;

pub async fn seed(db_conn: &SchemaManagerConnection<'_>) -> anyhow::Result<()> {
    let role_keys = vec![
        roles::role_keys::ADMIN,
        roles::role_keys::USER,
        roles::role_keys::GUEST,
    ];

    for role_key in role_keys {
        let perm_row = entity::roles::Entity::find()
            .filter(entity::roles::Column::Key.eq(role_key))
            .one(db_conn)
            .await?;

        if perm_row.is_some() {
            continue;
        }

        let perm_row = entity::roles::ActiveModel {
            key: Set(role_key.to_owned()),
            ..Default::default()
        };
        let perm_row = perm_row.insert(db_conn).await?;
        tracing::info!("Permission {} created, id: {}", role_key, perm_row.id);
    }

    Ok(())
}
