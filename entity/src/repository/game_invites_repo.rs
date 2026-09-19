use std::sync::Arc;

use chrono::{Duration, NaiveDateTime, Utc};
use sea_orm::{
    ActiveModelTrait, ActiveValue::Set, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter,
};

use crate::game_invites::GameInviteStatus;

const INVITE_TTL_HOURS: i64 = 24;

#[derive(Clone)]
pub struct GameInvitesRepository {
    db_conn: Arc<DatabaseConnection>,
}

impl GameInvitesRepository {
    pub fn new(db_conn: Arc<DatabaseConnection>) -> Self {
        Self { db_conn }
    }

    pub async fn create(
        &self,
        token: &str,
        created_by_user_id: u64,
    ) -> anyhow::Result<crate::game_invites::Model> {
        let created_at = now();
        let row = crate::game_invites::ActiveModel {
            token: Set(token.to_string()),
            created_by_user_id: Set(created_by_user_id),
            status: Set(GameInviteStatus::Pending),
            created_at: Set(created_at),
            expires_at: Set(Some(created_at + Duration::hours(INVITE_TTL_HOURS))),
            ..Default::default()
        };

        Ok(row.insert(&*self.db_conn).await?)
    }

    pub async fn get_by_token(&self, token: &str) -> Option<crate::game_invites::Model> {
        crate::game_invites::Entity::find()
            .filter(crate::game_invites::Column::Token.eq(token))
            .one(&*self.db_conn)
            .await
            .ok()?
    }

    pub async fn mark_accepted(
        &self,
        id: u64,
        game_id: u64,
    ) -> anyhow::Result<crate::game_invites::Model> {
        let row = crate::game_invites::Entity::find_by_id(id)
            .one(&*self.db_conn)
            .await?
            .ok_or(anyhow::anyhow!("Not found"))?;
        let mut row: crate::game_invites::ActiveModel = row.into();
        row.status = Set(GameInviteStatus::Accepted);
        row.game_id = Set(Some(game_id));

        Ok(row.update(&*self.db_conn).await?)
    }
}

fn now() -> NaiveDateTime {
    Utc::now().naive_utc()
}
