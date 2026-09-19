use std::sync::Arc;

use sea_orm::{
    ActiveModelTrait, ActiveValue::Set, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter,
};

#[derive(Clone)]
pub struct ScoresRepository {
    db_conn: Arc<DatabaseConnection>,
}

impl ScoresRepository {
    pub fn new(db_conn: Arc<DatabaseConnection>) -> Self {
        Self { db_conn }
    }

    pub async fn get_by_user_id(&self, user_id: u64) -> Option<crate::scores::Model> {
        crate::scores::Entity::find()
            .filter(crate::scores::Column::UserId.eq(user_id))
            .one(&*self.db_conn)
            .await
            .ok()?
    }

    pub async fn record_win(&self, user_id: u64) -> anyhow::Result<crate::scores::Model> {
        self.increment(user_id, true).await
    }

    pub async fn record_loss(&self, user_id: u64) -> anyhow::Result<crate::scores::Model> {
        self.increment(user_id, false).await
    }

    async fn increment(&self, user_id: u64, is_win: bool) -> anyhow::Result<crate::scores::Model> {
        match self.get_by_user_id(user_id).await {
            Some(row) => {
                let wins = row.wins;
                let losses = row.losses;
                let mut row: crate::scores::ActiveModel = row.into();
                if is_win {
                    row.wins = Set(wins + 1);
                } else {
                    row.losses = Set(losses + 1);
                }

                Ok(row.update(&*self.db_conn).await?)
            }
            None => {
                let row = crate::scores::ActiveModel {
                    user_id: Set(user_id),
                    wins: Set(u32::from(is_win)),
                    losses: Set(u32::from(!is_win)),
                    ..Default::default()
                };

                Ok(row.insert(&*self.db_conn).await?)
            }
        }
    }
}
