use r2d2::ManageConnection;
use redis::{Client, Connection, RedisError};

#[derive(Debug, Clone)]
pub struct RedisManager {
    client: Client,
}

impl RedisManager {
    pub fn new(redis_url: &str) -> Result<Self, RedisError> {
        let client = Client::open(redis_url)?;
        Ok(Self { client })
    }
}

impl ManageConnection for RedisManager {
    type Connection = Connection;
    type Error = RedisError;

    fn connect(&self) -> Result<Self::Connection, Self::Error> {
        self.client.get_connection()
    }

    fn is_valid(&self, conn: &mut Self::Connection) -> Result<(), Self::Error> {
        redis::cmd("PING").query(conn)
    }

    fn has_broken(&self, _conn: &mut Self::Connection) -> bool {
        false
    }
}
