use redis::{Client, RedisError, AsyncCommands, aio::MultiplexedConnection};
use serde::{Serialize, Deserialize, de::DeserializeOwned};

#[derive(Debug, thiserror::Error)]
pub enum RedisErrorWrapper {
    #[error("Redis connection error: {0}")]
    Connection(String),
    #[error("Redis operation error: {0}")]
    Operation(String),
    #[error("Serialization error: {0}")]
    Serialization(String),
}

impl From<RedisError> for RedisErrorWrapper {
    fn from(err: RedisError) -> Self {
        RedisErrorWrapper::Operation(err.to_string())
    }
}

impl From<serde_json::Error> for RedisErrorWrapper {
    fn from(err: serde_json::Error) -> Self {
        RedisErrorWrapper::Serialization(err.to_string())
    }
}

#[derive(Clone)]
pub struct RedisService {
    client: Client,
}

impl RedisService {
    pub fn new(redis_url: &str) -> Result<Self, RedisErrorWrapper> {
        let client = Client::open(redis_url)
            .map_err(|e| RedisErrorWrapper::Connection(e.to_string()))?;
        Ok(Self { client })
    }

    /// Get async multiplexed connection
    pub async fn get_async_connection(&self) -> Result<MultiplexedConnection, RedisErrorWrapper> {
        self.client.get_multiplexed_async_connection()
            .await
            .map_err(|e| RedisErrorWrapper::Connection(e.to_string()))
    }

    /// Set a value with expiration
    pub async fn set_with_expiration<T: Serialize>(
        &self,
        key: &str,
        value: &T,
        seconds: u64,
    ) -> Result<(), RedisErrorWrapper> {
        let mut conn = self.get_async_connection().await?;
        let json = serde_json::to_string(value)?;
        conn.set_ex(key, json, seconds).await?;
        Ok(())
    }

    /// Get a value
    pub async fn get<T: DeserializeOwned>(
        &self,
        key: &str,
    ) -> Result<Option<T>, RedisErrorWrapper> {
        let mut conn = self.get_async_connection().await?;
        let result: Option<String> = conn.get(key).await?;
        match result {
            Some(json) => {
                let value: T = serde_json::from_str(&json)?;
                Ok(Some(value))
            }
            None => Ok(None),
        }
    }

    /// Delete a key
    pub async fn delete(&self, key: &str) -> Result<(), RedisErrorWrapper> {
        let mut conn = self.get_async_connection().await?;
        conn.del(key).await?;
        Ok(())
    }

    /// Check if key exists
    pub async fn exists(&self, key: &str) -> Result<bool, RedisErrorWrapper> {
        let mut conn = self.get_async_connection().await?;
        let result: bool = conn.exists(key).await?;
        Ok(result)
    }

    /// Increment a counter
    pub async fn increment(&self, key: &str) -> Result<i64, RedisErrorWrapper> {
        let mut conn = self.get_async_connection().await?;
        let result: i64 = conn.incr(key, 1).await?;
        Ok(result)
    }

    /// Set expiration on existing key
    pub async fn expire(&self, key: &str, seconds: u64) -> Result<(), RedisErrorWrapper> {
        let mut conn = self.get_async_connection().await?;
        conn.expire(key, seconds as i64).await?;
        Ok(())
    }

    /// Get with pattern (keys)
    pub async fn keys(&self, pattern: &str) -> Result<Vec<String>, RedisErrorWrapper> {
        let mut conn = self.get_async_connection().await?;
        let keys: Vec<String> = conn.keys(pattern).await?;
        Ok(keys)
    }

    /// Clear all keys matching pattern
    pub async fn clear_pattern(&self, pattern: &str) -> Result<(), RedisErrorWrapper> {
        let keys = self.keys(pattern).await?;
        if !keys.is_empty() {
            let mut conn = self.get_async_connection().await?;
            conn.del(&keys).await?;
        }
        Ok(())
    }
}
