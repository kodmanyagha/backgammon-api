use std::sync::Arc;

use sea_orm::{
    ActiveModelTrait, ActiveValue::Set, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter,
    QueryOrder,
};

use crate::game_moves::GameMovePlayer;

#[derive(Clone)]
pub struct GameMovesRepository {
    db_conn: Arc<DatabaseConnection>,
}

impl GameMovesRepository {
    pub fn new(db_conn: Arc<DatabaseConnection>) -> Self {
        Self { db_conn }
    }

    pub async fn append(
        &self,
        game_id: u64,
        sequence_no: u32,
        player: GameMovePlayer,
        origin_point: Option<u8>,
        die: u8,
    ) -> anyhow::Result<crate::game_moves::Model> {
        let row = crate::game_moves::ActiveModel {
            game_id: Set(game_id),
            sequence_no: Set(sequence_no),
            player: Set(player),
            origin_point: Set(origin_point),
            die: Set(die),
            ..Default::default()
        };

        Ok(row.insert(&*self.db_conn).await?)
    }

    pub async fn get_for_game(&self, game_id: u64) -> anyhow::Result<Vec<crate::game_moves::Model>> {
        Ok(crate::game_moves::Entity::find()
            .filter(crate::game_moves::Column::GameId.eq(game_id))
            .order_by_asc(crate::game_moves::Column::SequenceNo)
            .all(&*self.db_conn)
            .await?)
    }
}
