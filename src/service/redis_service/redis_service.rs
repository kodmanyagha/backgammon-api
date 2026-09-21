use std::collections::HashSet;

use bb8_redis::{bb8, RedisConnectionManager};
use redis::{
    aio::MultiplexedConnection, AsyncTypedCommands, Client, LposOptions, SetOptions,
    SortedSetAddOptions,
};

use crate::{
    service::redis_service::{redis_manager::RedisManager, sscan_stream::SScanStream},
    CONFIG,
};

#[derive(Debug, Clone)]
pub struct RedisService {
    client: Client,
    r2d2_pool: r2d2::Pool<RedisManager>,
    bb8_pool: bb8::Pool<RedisConnectionManager>,
}

impl RedisService {
    pub async fn new() -> anyhow::Result<Self> {
        let manager = RedisConnectionManager::new(CONFIG.get_redis_url())?;
        let bb8_pool = bb8::Pool::builder()
            .min_idle(8)
            .max_size(64)
            .connection_timeout(std::time::Duration::from_secs(30))
            .build(manager)
            .await?;

        let manager = RedisManager::new(CONFIG.get_redis_url())
            .map_err(|err| anyhow::anyhow!("Error creating Redis manager: {err}"))?;

        let r2d2_pool = r2d2::Pool::builder()
            .max_size(64)
            .build(manager)
            .map_err(|err| anyhow::anyhow!("Error creating pool: {err}"))?;

        let client = redis::Client::open(CONFIG.get_redis_url())
            .map_err(|err| anyhow::anyhow!("Error occured when creating pool: {err}"))?;

        Ok(RedisService {
            client,
            r2d2_pool,
            bb8_pool,
        })
    }

    pub async fn new_client(&self) -> Client {
        self.client.clone()
    }

    pub async fn get(&self, key: &str) -> anyhow::Result<Option<String>> {
        let mut conn = self.conn().await?;

        conn.get(key)
            .await
            .map_err(|err| anyhow::anyhow!("Err: {err}"))
    }

    pub async fn set(&self, key: &str, value: &str) -> anyhow::Result<Option<String>> {
        self.set_with_expiration(key, value, None).await
    }

    pub async fn set_with_expiration(
        &self,
        key: &str,
        value: &str,
        expiration: Option<time::Duration>,
    ) -> anyhow::Result<Option<String>> {
        let mut conn = self.client.get_multiplexed_async_connection().await?;

        let mut opts = SetOptions::default().get(true);

        if let Some(expiration) = expiration {
            opts = opts.with_expiration(redis::SetExpiry::EX(expiration.as_seconds_f64() as u64));
        }

        conn.set_options(key, value, opts)
            .await
            .map_err(|err| anyhow::anyhow!("Err: {err}"))
    }

    pub async fn conn_multiplexed_async(&self) -> anyhow::Result<MultiplexedConnection> {
        self.client
            .get_multiplexed_async_connection()
            .await
            .map_err(|err| anyhow::anyhow!("Err: {err}"))
    }

    pub async fn conn(&self) -> anyhow::Result<bb8::PooledConnection<'_, RedisConnectionManager>> {
        self.bb8_pool
            .get()
            .await
            .map_err(|err| anyhow::anyhow!("Err: {err}"))
    }

    pub async fn conn_dedicated(&self) -> anyhow::Result<redis::aio::MultiplexedConnection> {
        let mut conn = self
            .bb8_pool
            .dedicated_connection()
            .await
            .map_err(|err| anyhow::anyhow!("Err: {err}"))?;

        conn.set_response_timeout(std::time::Duration::from_secs(30));

        Ok(conn)
    }

    pub async fn sscan(&self, key: &str) -> anyhow::Result<SScanStream> {
        let conn = self.client.get_multiplexed_async_connection().await?;

        Ok(SScanStream::new(conn, key, 100, None))
    }

    pub async fn scard(&self, key: &str) -> anyhow::Result<usize> {
        let mut conn = self.client.get_multiplexed_async_connection().await?;

        conn.scard(key)
            .await
            .map_err(|err| anyhow::anyhow!("Err: {err}"))
    }

    pub async fn sadd(&self, key: &str, members: &[&str]) -> anyhow::Result<usize> {
        self.sadd_with_expiration(key, members, None).await
    }

    pub async fn sadd_with_expiration(
        &self,
        key: &str,
        members: &[&str],
        expiration: Option<time::Duration>,
    ) -> anyhow::Result<usize> {
        let mut conn = self.client.get_multiplexed_async_connection().await?;

        let result = conn
            .sadd(key, members)
            .await
            .map_err(|err| anyhow::anyhow!("Err: {err}"));

        if let Some(duration) = expiration {
            let _ = conn.expire(key, duration.as_seconds_f64() as i64).await;
        }

        result
    }

    pub async fn zcard(&self, key: &str) -> anyhow::Result<usize> {
        let mut conn = self.client.get_multiplexed_async_connection().await?;

        Ok(conn.zcard(key).await.unwrap_or_default())
    }

    pub async fn zrange(
        &self,
        key: &str,
        start: isize,
        stop: isize,
    ) -> anyhow::Result<Vec<String>> {
        let mut conn = self.client.get_multiplexed_async_connection().await?;

        conn.zrange(key, start, stop)
            .await
            .map_err(|err| anyhow::anyhow!("Err: {err}"))
    }

    pub async fn zrem(&self, key: &str, members: &[&str]) -> anyhow::Result<usize> {
        if members.is_empty() {
            return Ok(0);
        }

        let mut conn = self.client.get_multiplexed_async_connection().await?;

        conn.zrem(key, members)
            .await
            .map_err(|err| anyhow::anyhow!("Err: {err}"))
    }

    pub async fn zadd_auto(&self, key: &str, members: &[&str]) -> anyhow::Result<usize> {
        self.zadd_auto_with_expiration(key, members, None).await
    }

    pub async fn zadd_auto_with_expiration(
        &self,
        key: &str,
        members: &[&str],
        expiration: Option<time::Duration>,
    ) -> anyhow::Result<usize> {
        let mut conn = self.client.get_multiplexed_async_connection().await?;

        let zlen = conn.zcard(key).await.unwrap_or_default();
        let members = members
            .iter()
            .enumerate()
            .map(|(index, item)| (zlen + index, *item))
            .collect::<Vec<(usize, &str)>>();

        let opts = SortedSetAddOptions::add_only();
        let result = conn
            .zadd_multiple_options(key, members.as_ref(), &opts)
            .await
            .map_err(|err| anyhow::anyhow!("Err: {err}"));

        if let Some(duration) = expiration {
            let _ = conn.expire(key, duration.as_seconds_f64() as i64).await;
        }

        result
    }

    pub async fn rpush(&self, key: &str, items: &[&str]) -> anyhow::Result<usize> {
        self.rpush_with_expiration(key, items, None).await
    }

    pub async fn rpush_with_expiration(
        &self,
        key: &str,
        items: &[&str],
        expiration: Option<time::Duration>,
    ) -> anyhow::Result<usize> {
        let mut conn = self.client.get_multiplexed_async_connection().await?;

        let result = conn
            .rpush(key, items)
            .await
            .map_err(|err| anyhow::anyhow!("Err: {err}"));

        if let Some(duration) = expiration {
            let _ = conn.expire(key, duration.as_seconds_f64() as i64).await;
        }

        result
    }

    pub async fn lpush(&self, key: &str, items: &[&str]) -> anyhow::Result<usize> {
        self.lpush_with_expiration(key, items, None).await
    }

    pub async fn lpush_with_expiration(
        &self,
        key: &str,
        items: &[&str],
        expiration: Option<time::Duration>,
    ) -> anyhow::Result<usize> {
        let mut conn = self.client.get_multiplexed_async_connection().await?;

        let result = conn
            .lpush(key, items)
            .await
            .map_err(|err| anyhow::anyhow!("Err: {err}"));

        if let Some(duration) = expiration {
            let _ = conn.expire(key, duration.as_seconds_f64() as i64).await;
        }

        result
    }

    pub async fn lpop(&self, key: &str) -> anyhow::Result<String> {
        let mut conn = self.client.get_multiplexed_async_connection().await?;

        conn.lpop(key, None)
            .await
            .map_err(|err| anyhow::anyhow!("Err: {err}"))
    }

    pub async fn lpop_vec(
        &self,
        key: &str,
        count: Option<core::num::NonZeroUsize>,
    ) -> anyhow::Result<Vec<String>> {
        let mut conn = self.client.get_multiplexed_async_connection().await?;

        conn.lpop(key, count)
            .await
            .map_err(|err| anyhow::anyhow!("Err: {err}"))
    }

    pub async fn rpop(&self, key: &str) -> anyhow::Result<String> {
        let mut conn = self.client.get_multiplexed_async_connection().await?;

        conn.rpop(key, None)
            .await
            .map_err(|err| anyhow::anyhow!("Err: {err}"))
    }

    pub async fn rpop_vec(
        &self,
        key: &str,
        count: Option<core::num::NonZeroUsize>,
    ) -> anyhow::Result<Vec<String>> {
        let mut conn = self.client.get_multiplexed_async_connection().await?;

        conn.rpop(key, count)
            .await
            .map_err(|err| anyhow::anyhow!("Err: {err}"))
    }

    pub async fn llen(&self, key: &str) -> anyhow::Result<usize> {
        let mut conn = self.client.get_multiplexed_async_connection().await?;

        conn.llen(key)
            .await
            .map_err(|err| anyhow::anyhow!("Err: {err}"))
    }

    pub async fn lpos_first(&self, key: &str, value: &str) -> anyhow::Result<Option<usize>> {
        let mut conn = self.client.get_multiplexed_async_connection().await?;

        let opts = LposOptions::default().count(1).rank(-1).maxlen(0);
        let result: Vec<usize> = conn
            .lpos(key, value, opts)
            .await
            .map_err(|err| anyhow::anyhow!("Err: {err}"))?;

        Ok(result.first().cloned())
    }

    pub async fn lrem_first(&self, key: &str, value: &str) -> anyhow::Result<usize> {
        let mut conn = self.client.get_multiplexed_async_connection().await?;

        conn.lrem(key, 1, value)
            .await
            .map_err(|err| anyhow::anyhow!("Err: {err}"))
    }

    pub async fn smembers(&self, key: &str) -> anyhow::Result<HashSet<String>> {
        let mut conn = self.client.get_multiplexed_async_connection().await?;

        conn.smembers(key)
            .await
            .map_err(|err| anyhow::anyhow!("Err: {err}"))
    }

    pub async fn set_with_ttl(&self, key: &str, value: &str, ttl_seconds: u64) -> anyhow::Result<()> {
        let mut conn = self.conn().await?;

        conn.set_ex(key, value, ttl_seconds)
            .await
            .map_err(|err| anyhow::anyhow!("Err: {err}"))
    }

    pub async fn get_del(&self, key: &str) -> anyhow::Result<Option<String>> {
        let mut conn = self.conn().await?;

        conn.get_del(key)
            .await
            .map_err(|err| anyhow::anyhow!("Err: {err}"))
    }

    pub async fn exists(&self, key: &str) -> anyhow::Result<bool> {
        let mut conn = self.conn().await?;

        conn.exists(key)
            .await
            .map_err(|err| anyhow::anyhow!("Err: {err}"))
    }

    pub async fn del(&self, key: &str) -> anyhow::Result<usize> {
        let mut conn = self.conn().await?;

        conn.del(key)
            .await
            .map_err(|err| anyhow::anyhow!("Err: {err}"))
    }

    pub async fn ping(&self) -> anyhow::Result<String> {
        let mut conn = self.client.get_multiplexed_async_connection().await?;

        conn.ping()
            .await
            .map_err(|err| anyhow::anyhow!("Err: {err}"))
    }
}
