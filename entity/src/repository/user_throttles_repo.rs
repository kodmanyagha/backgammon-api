use std::sync::Arc;

use sea_orm::{
    ActiveModelTrait, ActiveValue::Set, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter,
};

#[derive(Clone)]
pub struct UserThrottlesRepository {
    db_conn: Arc<DatabaseConnection>,
}

impl UserThrottlesRepository {
    pub fn new(db_conn: Arc<DatabaseConnection>) -> Self {
        Self { db_conn }
    }

    pub async fn get_all(&self) -> anyhow::Result<Vec<crate::user_throttles::Model>> {
        Ok(crate::user_throttles::Entity::find()
            .all(&*self.db_conn)
            .await?)
    }

    pub async fn get_by_user_id(&self, user_id: u64) -> Option<crate::user_throttles::Model> {
        crate::user_throttles::Entity::find()
            .filter(crate::user_throttles::Column::UserId.eq(user_id))
            .one(&*self.db_conn)
            .await
            .ok()?
    }

    pub async fn set_requests_per_second(
        &self,
        user_id: u64,
        requests_per_second: u64,
    ) -> anyhow::Result<crate::user_throttles::Model> {
        match self.get_by_user_id(user_id).await {
            Some(row) => {
                let mut row: crate::user_throttles::ActiveModel = row.into();
                row.requests_per_second = Set(requests_per_second);
                Ok(row.update(&*self.db_conn).await?)
            }
            None => {
                let row = crate::user_throttles::ActiveModel {
                    user_id: Set(user_id),
                    requests_per_second: Set(requests_per_second),
                    ..Default::default()
                };
                Ok(row.insert(&*self.db_conn).await?)
            }
        }
    }
}
