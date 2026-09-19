use entity::permissions;
use sea_orm::{ActiveModelTrait, ActiveValue::Set, ColumnTrait, EntityTrait, QueryFilter};
use sea_orm_migration::SchemaManagerConnection;

pub async fn seed(db_conn: &SchemaManagerConnection<'_>) -> anyhow::Result<()> {
    let perm_keys = vec![
        permissions::perm_keys::SYSTEM_MANAGER,
        permissions::perm_keys::USER,
    ];

    for perm_key in perm_keys {
        let perm_row = entity::permissions::Entity::find()
            .filter(entity::permissions::Column::Key.eq(perm_key))
            .one(db_conn)
            .await?;

        if perm_row.is_some() {
            continue;
        }

        let perm_row = entity::permissions::ActiveModel {
            key: Set(perm_key.to_owned()),
            ..Default::default()
        };

        let perm_row = perm_row.insert(db_conn).await?;

        tracing::info!("Permission {} created, id: {}", perm_key, perm_row.id);
    }

    Ok(())
}
