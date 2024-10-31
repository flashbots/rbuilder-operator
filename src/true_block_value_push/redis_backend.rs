use redis::Commands;

use super::best_true_value_pusher::Backend;

pub struct RedisBackend {
    redis: redis::Client,
    channel_name: String,
}

impl RedisBackend {
    pub fn new(redis: redis::Client, channel_name: String) -> Self {
        Self {
            redis,
            channel_name,
        }
    }
}

impl Backend for RedisBackend {
    type Connection = redis::Connection;
    type BackendError = redis::RedisError;

    fn connect(&self) -> Result<Self::Connection, Self::BackendError> {
        self.redis.get_connection()
    }

    fn publish(
        &self,
        connection: &mut Self::Connection,
        data: &str,
    ) -> Result<(), Self::BackendError> {
        connection.publish(&self.channel_name, &data)
    }
}
