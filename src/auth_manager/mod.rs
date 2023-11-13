pub mod redis_auth_manager;

use async_trait::async_trait;
use redis::{aio::Connection, RedisError, RedisResult};
use redis::{AsyncCommands, FromRedisValue, ToRedisArgs};
use serde::{Deserialize, Serialize};

#[async_trait]
pub trait AuthManager {
    async fn login(
        &mut self,
        username: String,
        password: String,
    ) -> Result<String, Box<dyn std::error::Error>>;
    async fn register(
        &mut self,
        username: String,
        password: String,
    ) -> Result<String, Box<dyn std::error::Error>>;
}
