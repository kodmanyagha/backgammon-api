use std::time::Duration;

use chrono::Utc;
use sea_orm::{DatabaseConnection, TransactionTrait};
use tavla_core::Player;

use crate::{
    service::game::{
        live_state::now_unix_ms,
        live_store,
        outbox::{plan_batch, score_deltas, BatchPlan, OutboxEvent},
    },
    state::app_state::AppState,
};

pub const FLUSH_INTERVAL: Duration = Duration::from_millis(500);

const FLUSH_BATCH_LIMIT: usize = 2000;
const RETRY_BACKOFF: Duration = Duration::from_secs(2);
const STALE_SWEEP_INTERVAL: Duration = Duration::from_secs(60);
const STALE_MATCH_AFTER: Duration = Duration::from_secs(10 * 60);
const STALE_SWEEP_LIMIT: u64 = 500;

pub async fn run(state: AppState) {
    let mut next_sweep = tokio::time::Instant::now() + STALE_SWEEP_INTERVAL;

    loop {
        tokio::time::sleep(FLUSH_INTERVAL).await;

        loop {
            match flush_once(&state).await {
                Ok(flushed) if flushed < FLUSH_BATCH_LIMIT => break,
                Ok(_) => continue,
                Err(err) => {
                    tracing::error!(error = %err, "game event flush failed, retrying");
                    tokio::time::sleep(RETRY_BACKOFF).await;
                    break;
                }
            }
        }

        if tokio::time::Instant::now() >= next_sweep {
            next_sweep = tokio::time::Instant::now() + STALE_SWEEP_INTERVAL;
            if let Err(err) = close_stale_matches(&state).await {
                tracing::error!(error = %err, "stale match sweep failed");
            }
        }
    }
}

pub async fn flush_once(state: &AppState) -> anyhow::Result<usize> {
    let raw_events = live_store::peek_outbox(&state.redis_service, FLUSH_BATCH_LIMIT).await?;
    if raw_events.is_empty() {
        return Ok(0);
    }

    let events: Vec<OutboxEvent> = raw_events
        .iter()
        .filter_map(|raw| match serde_json::from_str(raw) {
            Ok(event) => Some(event),
            Err(err) => {
                tracing::error!(error = %err, "dropping an unreadable game event");
                None
            }
        })
        .collect();

    apply_plan(&state.db_conn, plan_batch(&events)).await?;
    live_store::drop_from_outbox(&state.redis_service, raw_events.len()).await?;

    Ok(raw_events.len())
}

async fn apply_plan(db: &DatabaseConnection, plan: BatchPlan) -> anyhow::Result<()> {
    use entity::repository::game_batch_repo::{add_score_deltas, close_match, insert_moves, insert_rounds};

    let BatchPlan { moves, rounds, match_ends } = plan;

    let transaction = db.begin().await?;
    insert_moves(&transaction, moves).await?;
    insert_rounds(&transaction, rounds).await?;

    let mut applied = Vec::with_capacity(match_ends.len());
    for match_end in &match_ends {
        applied.push(close_match(&transaction, &match_end.row).await?);
    }
    add_score_deltas(&transaction, &score_deltas(&match_ends, &applied)).await?;

    transaction.commit().await?;
    Ok(())
}

pub async fn close_stale_matches(state: &AppState) -> anyhow::Result<()> {
    let cutoff = Utc::now().naive_utc() - chrono::Duration::from_std(STALE_MATCH_AFTER)?;
    let cutoff_ms = now_unix_ms() - STALE_MATCH_AFTER.as_millis() as i64;

    for game in state.games_repo.active_started_before(cutoff, STALE_SWEEP_LIMIT).await? {
        if state.game_sessions.contains(game.id).await {
            continue;
        }

        let snapshot = match live_store::load_snapshot(&state.redis_service, game.id).await {
            Ok(snapshot) => snapshot,
            Err(err) => {
                tracing::warn!(game_id = game.id, error = %err, "stale match check skipped");
                continue;
            }
        };
        if snapshot.as_ref().is_some_and(|snapshot| snapshot.saved_at_ms > cutoff_ms) {
            continue;
        }

        let (white_score, black_score) = snapshot
            .map(|snapshot| (snapshot.session.score_for(Player::White), snapshot.session.score_for(Player::Black)))
            .unwrap_or_default();
        let event = OutboxEvent::MatchAbandoned { game_id: game.id, white_score, black_score, at_ms: now_unix_ms() };
        live_store::close_game(&state.redis_service, game.id, &[event]).await?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_flush_interval_is_half_a_second() {
        assert_eq!(FLUSH_INTERVAL, Duration::from_millis(500));
    }
}
