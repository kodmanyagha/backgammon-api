use std::sync::Arc;

use sea_orm::ColumnTrait;
use sea_orm::DatabaseConnection;
use sea_orm::EntityTrait;
use sea_orm::QueryFilter;

#[derive(Clone)]
pub struct RolesRepository {
    db_conn: Arc<DatabaseConnection>,
}

impl RolesRepository {
    pub fn new(db_conn: Arc<DatabaseConnection>) -> Self {
        Self { db_conn }
    }

    pub async fn get_by_id(&self, id: u64) -> Option<crate::roles::Model> {
        crate::roles::Entity::find_by_id(id)
            .one(&*self.db_conn)
            .await
            .ok()?
    }

    pub async fn get_by_key(&self, role_key: &str) -> Option<crate::roles::Model> {
        crate::roles::Entity::find()
            .filter(crate::roles::Column::Key.eq(role_key))
            .one(&*self.db_conn)
            .await
            .ok()?
    }
}
