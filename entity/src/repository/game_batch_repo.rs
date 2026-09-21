use std::collections::HashMap;

use chrono::NaiveDateTime;
use sea_orm::{
    sea_query::{Expr, OnConflict},
    ActiveValue::Set,
    ColumnTrait, ConnectionTrait, DatabaseBackend, EntityTrait, Insert, QueryFilter, Statement, Value,
};

use crate::{
    game_moves::{self, GameMovePlayer},
    game_rounds,
    games::{self, GameStatus},
};

const ROWS_PER_STATEMENT: usize = 500;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NewMoveRow {
    pub game_id: u64,
    pub round_no: u16,
    pub sequence_no: u32,
    pub player: GameMovePlayer,
    pub origin_point: Option<u8>,
    pub die: u8,
    pub is_ai: bool,
    pub created_at: NaiveDateTime,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NewRoundRow {
    pub game_id: u64,
    pub round_no: u16,
    pub winner_user_id: u64,
    pub white_score: u16,
    pub black_score: u16,
    pub created_at: NaiveDateTime,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MatchEndRow {
    pub game_id: u64,
    pub status: GameStatus,
    pub winner_user_id: Option<u64>,
    pub white_score: u16,
    pub black_score: u16,
    pub finished_at: NaiveDateTime,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct ScoreDelta {
    pub wins: u32,
    pub losses: u32,
}

fn moves_insert(rows: &[NewMoveRow]) -> Insert<game_moves::ActiveModel> {
    let models = rows.iter().map(|row| game_moves::ActiveModel {
        game_id: Set(row.game_id),
        round_no: Set(row.round_no),
        sequence_no: Set(row.sequence_no),
        player: Set(row.player.clone()),
        origin_point: Set(row.origin_point),
        die: Set(row.die),
        is_ai: Set(row.is_ai),
        created_at: Set(row.created_at),
        ..Default::default()
    });

    game_moves::Entity::insert_many(models).on_conflict(
        OnConflict::columns([game_moves::Column::GameId, game_moves::Column::SequenceNo])
            .update_column(game_moves::Column::GameId)
            .to_owned(),
    )
}

fn rounds_insert(rows: &[NewRoundRow]) -> Insert<game_rounds::ActiveModel> {
    let models = rows.iter().map(|row| game_rounds::ActiveModel {
        game_id: Set(row.game_id),
        round_no: Set(row.round_no),
        winner_user_id: Set(row.winner_user_id),
        white_score: Set(row.white_score),
        black_score: Set(row.black_score),
        created_at: Set(row.created_at),
        ..Default::default()
    });

    game_rounds::Entity::insert_many(models).on_conflict(
        OnConflict::columns([game_rounds::Column::GameId, game_rounds::Column::RoundNo])
            .update_column(game_rounds::Column::GameId)
            .to_owned(),
    )
}

pub async fn insert_moves<C: ConnectionTrait>(conn: &C, rows: Vec<NewMoveRow>) -> anyhow::Result<()> {
    for chunk in rows.chunks(ROWS_PER_STATEMENT) {
        moves_insert(chunk).exec_without_returning(conn).await?;
    }

    Ok(())
}

pub async fn insert_rounds<C: ConnectionTrait>(conn: &C, rows: Vec<NewRoundRow>) -> anyhow::Result<()> {
    for chunk in rows.chunks(ROWS_PER_STATEMENT) {
        rounds_insert(chunk).exec_without_returning(conn).await?;
    }

    Ok(())
}

pub async fn close_match<C: ConnectionTrait>(conn: &C, end: &MatchEndRow) -> anyhow::Result<bool> {
    let result = games::Entity::update_many()
        .col_expr(games::Column::Status, Expr::value(end.status.clone()))
        .col_expr(games::Column::WinnerUserId, Expr::value(end.winner_user_id))
        .col_expr(games::Column::WhiteScore, Expr::value(end.white_score))
        .col_expr(games::Column::BlackScore, Expr::value(end.black_score))
        .col_expr(games::Column::FinishedAt, Expr::value(end.finished_at))
        .filter(games::Column::Id.eq(end.game_id))
        .filter(games::Column::Status.eq(GameStatus::Active))
        .exec(conn)
        .await?;

    Ok(result.rows_affected == 1)
}

pub async fn add_score_deltas<C: ConnectionTrait>(
    conn: &C,
    deltas: &HashMap<u64, ScoreDelta>,
) -> anyhow::Result<()> {
    if deltas.is_empty() {
        return Ok(());
    }

    let placeholders = vec!["(?, ?, ?)"; deltas.len()].join(", ");
    let sql = format!(
        "INSERT INTO scores (user_id, wins, losses) VALUES {placeholders} \
         ON DUPLICATE KEY UPDATE wins = wins + VALUES(wins), losses = losses + VALUES(losses), \
         updated_at = CURRENT_TIMESTAMP"
    );
    let values: Vec<Value> = deltas
        .iter()
        .flat_map(|(user_id, delta)| [Value::from(*user_id), Value::from(delta.wins), Value::from(delta.losses)])
        .collect();

    conn.execute(Statement::from_sql_and_values(DatabaseBackend::MySql, sql, values))
        .await?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use sea_orm::{DbBackend, QueryTrait};

    use super::*;

    fn at() -> NaiveDateTime {
        chrono::DateTime::from_timestamp_millis(1_790_000_000_000)
            .expect("valid timestamp")
            .naive_utc()
    }

    fn sql_of<A: sea_orm::ActiveModelTrait>(insert: Insert<A>) -> String {
        insert.build(DbBackend::MySql).to_string()
    }

    #[test]
    fn move_inserts_are_multi_row_and_ignore_duplicates_with_a_syntax_mysql_accepts() {
        let rows: Vec<NewMoveRow> = (0..3)
            .map(|sequence_no| NewMoveRow {
                game_id: 1,
                round_no: 1,
                sequence_no,
                player: GameMovePlayer::White,
                origin_point: Some(6),
                die: 3,
                is_ai: false,
                created_at: at(),
            })
            .collect();

        let sql = sql_of(moves_insert(&rows));

        assert_eq!(sql.matches("(1, 1, ").count(), 3, "{sql}");
        assert!(sql.contains("ON DUPLICATE KEY UPDATE"), "{sql}");
        assert!(!sql.contains("KEY IGNORE"), "{sql}");
    }

    #[test]
    fn round_inserts_ignore_duplicates_with_a_syntax_mysql_accepts() {
        let rows = vec![NewRoundRow { game_id: 1, round_no: 2, winner_user_id: 9, white_score: 1, black_score: 1, created_at: at() }];

        let sql = sql_of(rounds_insert(&rows));

        assert!(sql.contains("INSERT INTO `game_rounds`"), "{sql}");
        assert!(sql.contains("ON DUPLICATE KEY UPDATE"), "{sql}");
        assert!(!sql.contains("KEY IGNORE"), "{sql}");
    }
}
