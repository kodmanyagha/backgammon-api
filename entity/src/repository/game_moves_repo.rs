use std::sync::Arc;

use sea_orm::{ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, QueryOrder};

#[derive(Clone)]
pub struct GameMovesRepository {
    db_conn: Arc<DatabaseConnection>,
}

impl GameMovesRepository {
    pub fn new(db_conn: Arc<DatabaseConnection>) -> Self {
        Self { db_conn }
    }

    pub async fn get_for_game(&self, game_id: u64) -> anyhow::Result<Vec<crate::game_moves::Model>> {
        Ok(crate::game_moves::Entity::find()
            .filter(crate::game_moves::Column::GameId.eq(game_id))
            .order_by_asc(crate::game_moves::Column::SequenceNo)
            .all(&*self.db_conn)
            .await?)
    }
}
