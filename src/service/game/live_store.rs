use crate::service::{
    game::{live_state::LiveSnapshot, outbox::OutboxEvent},
    redis_service::redis_service::RedisService,
};

pub const OUTBOX_KEY: &str = "db:outbox";

const STATE_TTL_SECONDS: u64 = 24 * 60 * 60;

fn state_key(game_id: u64) -> String {
    format!("game:{game_id}:state")
}

fn serialize_events(events: &[OutboxEvent]) -> anyhow::Result<Vec<String>> {
    events
        .iter()
        .map(|event| serde_json::to_string(event).map_err(anyhow::Error::from))
        .collect()
}

pub async fn save_snapshot(
    redis: &RedisService,
    game_id: u64,
    snapshot: &LiveSnapshot,
    events: &[OutboxEvent],
) -> anyhow::Result<()> {
    let mut pipeline = redis::pipe();
    pipeline
        .atomic()
        .cmd("SET")
        .arg(state_key(game_id))
        .arg(serde_json::to_string(snapshot)?)
        .arg("EX")
        .arg(STATE_TTL_SECONDS)
        .ignore();
    for event in serialize_events(events)? {
        pipeline.cmd("RPUSH").arg(OUTBOX_KEY).arg(event).ignore();
    }

    let mut conn = redis.conn().await?;
    pipeline.query_async::<()>(&mut *conn).await?;
    Ok(())
}

pub async fn load_snapshot(redis: &RedisService, game_id: u64) -> anyhow::Result<Option<LiveSnapshot>> {
    let Some(json) = redis.get(&state_key(game_id)).await? else {
        return Ok(None);
    };

    Ok(Some(serde_json::from_str(&json)?))
}

pub async fn close_game(redis: &RedisService, game_id: u64, events: &[OutboxEvent]) -> anyhow::Result<()> {
    let mut pipeline = redis::pipe();
    pipeline.atomic().cmd("DEL").arg(state_key(game_id)).ignore();
    for event in serialize_events(events)? {
        pipeline.cmd("RPUSH").arg(OUTBOX_KEY).arg(event).ignore();
    }

    let mut conn = redis.conn().await?;
    pipeline.query_async::<()>(&mut *conn).await?;
    Ok(())
}

pub async fn peek_outbox(redis: &RedisService, limit: usize) -> anyhow::Result<Vec<String>> {
    let mut conn = redis.conn().await?;
    let raw: Vec<String> = redis::cmd("LRANGE")
        .arg(OUTBOX_KEY)
        .arg(0)
        .arg(limit as isize - 1)
        .query_async(&mut *conn)
        .await?;
    Ok(raw)
}

pub async fn drop_from_outbox(redis: &RedisService, count: usize) -> anyhow::Result<()> {
    let mut conn = redis.conn().await?;
    redis::cmd("LTRIM")
        .arg(OUTBOX_KEY)
        .arg(count)
        .arg(-1)
        .query_async::<()>(&mut *conn)
        .await?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_state_key_is_scoped_per_game() {
        assert_eq!(state_key(42), "game:42:state");
        assert_ne!(state_key(1), state_key(11));
    }
}
