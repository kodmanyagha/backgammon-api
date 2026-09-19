use anyhow::Result;
use futures::task::{Context, Poll};
use futures::TryFuture;
use futures::{future::BoxFuture, FutureExt, Stream};
use redis::aio::MultiplexedConnection;
use std::pin::Pin;

pub struct SScanStream {
    conn: MultiplexedConnection,
    key: String,
    cursor: u64,
    count: usize,
    pattern: Option<String>,
    done: bool,
    in_flight: Option<BoxFuture<'static, Result<(u64, Vec<String>)>>>,
}

impl SScanStream {
    pub fn new(
        conn: MultiplexedConnection,
        key: &str,
        count: usize,
        pattern: Option<&str>,
    ) -> Self {
        Self {
            conn,
            key: key.into(),
            cursor: 0,
            count,
            pattern: pattern.map(|p| p.into()),
            done: false,
            in_flight: None,
        }
    }

    fn make_fetch_future(&self, cursor: u64) -> BoxFuture<'static, Result<(u64, Vec<String>)>> {
        let key = self.key.clone();
        let pattern = self.pattern.clone();
        let count = self.count;
        let conn = self.conn.clone();

        async move {
            let mut conn = conn.clone();
            let mut cmd = redis::cmd("SSCAN");
            cmd.arg(&key).arg(cursor);
            if let Some(ref pat) = pattern {
                cmd.arg("MATCH").arg(pat);
            }
            cmd.arg("COUNT").arg(count);
            let (next_cursor, members): (u64, Vec<String>) = cmd
                .query_async(&mut conn)
                .await
                .map_err(|e| anyhow::anyhow!("SSCAN error: {}", e))?;
            Ok((next_cursor, members))
        }
        .boxed()
    }
}

impl Stream for SScanStream {
    type Item = Result<Vec<String>>;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        if self.done {
            return Poll::Ready(None);
        }

        if self.in_flight.is_none() {
            let fut = self.make_fetch_future(self.cursor);
            self.in_flight.replace(fut);
        }

        if let Some(mut fut) = self.in_flight.take() {
            let pinned = Pin::new(&mut fut);
            match pinned.try_poll(cx) {
                Poll::Ready(Ok((next_cursor, members))) => {
                    if next_cursor == 0 {
                        self.done = true;
                    } else {
                        self.cursor = next_cursor;
                    }
                    Poll::Ready(Some(Ok(members)))
                }
                Poll::Ready(Err(e)) => {
                    self.done = true;
                    Poll::Ready(Some(Err(e)))
                }
                Poll::Pending => {
                    self.in_flight = Some(fut);
                    Poll::Pending
                }
            }
        } else {
            Poll::Pending
        }
    }
}
