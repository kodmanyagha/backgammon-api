use std::sync::Arc;

use chrono::{NaiveDateTime, Utc};
use sea_orm::{
    ActiveModelTrait, ActiveValue::Set, Condition, ColumnTrait, DatabaseConnection, EntityTrait,
    QueryFilter, QueryOrder, QuerySelect,
};

use crate::games::GameStatus;

#[derive(Clone)]
pub struct GamesRepository {
    db_conn: Arc<DatabaseConnection>,
}

impl GamesRepository {
    pub fn new(db_conn: Arc<DatabaseConnection>) -> Self {
        Self { db_conn }
    }

    pub async fn create(
        &self,
        gold_user_id: u64,
        purple_user_id: u64,
    ) -> anyhow::Result<crate::games::Model> {
        let created_at = now();
        let row = crate::games::ActiveModel {
            gold_user_id: Set(gold_user_id),
            purple_user_id: Set(purple_user_id),
            status: Set(GameStatus::Active),
            created_at: Set(created_at),
            started_at: Set(Some(created_at)),
            ..Default::default()
        };

        Ok(row.insert(&*self.db_conn).await?)
    }

    pub async fn get_by_id(&self, id: u64) -> Option<crate::games::Model> {
        crate::games::Entity::find_by_id(id)
            .one(&*self.db_conn)
            .await
            .ok()?
    }

    pub async fn get_for_user(&self, user_id: u64, limit: u64) -> anyhow::Result<Vec<crate::games::Model>> {
        Ok(crate::games::Entity::find()
            .filter(
                Condition::any()
                    .add(crate::games::Column::GoldUserId.eq(user_id))
                    .add(crate::games::Column::PurpleUserId.eq(user_id)),
            )
            .order_by_desc(crate::games::Column::CreatedAt)
            .limit(limit)
            .all(&*self.db_conn)
            .await?)
    }

    pub async fn finish(&self, id: u64, winner_user_id: u64) -> anyhow::Result<crate::games::Model> {
        let row = self
            .get_by_id(id)
            .await
            .ok_or(anyhow::anyhow!("Not found"))?;
        let mut row: crate::games::ActiveModel = row.into();
        row.status = Set(GameStatus::Finished);
        row.winner_user_id = Set(Some(winner_user_id));
        row.finished_at = Set(Some(now()));

        Ok(row.update(&*self.db_conn).await?)
    }

    pub async fn abandon(&self, id: u64) -> anyhow::Result<crate::games::Model> {
        let row = self
            .get_by_id(id)
            .await
            .ok_or(anyhow::anyhow!("Not found"))?;
        let mut row: crate::games::ActiveModel = row.into();
        row.status = Set(GameStatus::Abandoned);
        row.finished_at = Set(Some(now()));

        Ok(row.update(&*self.db_conn).await?)
    }
}

fn now() -> NaiveDateTime {
    Utc::now().naive_utc()
}
