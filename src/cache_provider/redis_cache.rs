use async_trait::async_trait;
use redis::{aio::Connection, RedisError, RedisResult};
use redis::{AsyncCommands, FromRedisValue, ToRedisArgs};
use tracing::{error};

use crate::cache_provider::CacheProvider;

pub struct RedisCache {
    con: Connection,
}

impl RedisCache {
    pub async fn new(url: String) -> RedisCache {
        let client = redis::Client::open(url).unwrap();
        let con = client.get_async_connection().await.unwrap();

        RedisCache { con: con }
    }
}

#[async_trait(?Send)]
impl<
        K: Sync + Send + ToRedisArgs + 'static,
        V: Sync + Send + ToRedisArgs + FromRedisValue + 'static,
    > CacheProvider<K, V> for RedisCache
{
    async fn set(&mut self, key: K, value: V) {
        let ret: Result<(), RedisError> = self.con.set(key, value).await;
        match ret {
            Ok(_) => (),
            Err(e) => error!("Set failed at: {}", e),
        }
    }

    async fn get(&mut self, key: K) -> Option<V> {
        let ret: RedisResult<V> = self.con.get(key).await;
        match ret {
            Ok(value) => Some(value),
            Err(e) => {
                error!("Get failed at: {}", e);
                None
            }
        }
    }

    async fn get_or_query(&mut self, key: K) -> Option<V> {
        self.get(key).await
    }

    async fn remove(&mut self, key: K) {
        let ret: Result<(), RedisError> = self.con.del(key).await;
        match ret {
            Ok(_) => (),
            Err(e) => {
                error!("Get failed at: {}", e);
            }
        }
    }
}
#[cfg(test)]
mod tests {
    use serial_test::serial;
    use std::env;

    // Note this useful idiom: importing names from outer (for mod tests) scope.
    use super::*;

    async fn get_cache() -> Box<dyn CacheProvider<String, i32>> {
        let password;
        match env::var("REDIS_PASSWORD") {
            Ok(v) => password = v,
            Err(_) => panic!("$REDIS_PASSWORD is not set!"),
        };

        let url = "redis://default:".to_string() + &password + "@127.0.0.1:6379";
        Box::new(RedisCache::new(url).await)
    }

    #[tokio::test]
    #[ignore]
    #[serial]
    async fn test_set() {
        let mut cache = get_cache().await;
        cache.set("apples".to_string(), 5).await;
        cache.set("strawberries".to_string(), -142).await;
        cache.set("apples".to_string(), 3).await;
    }

    #[tokio::test]
    #[ignore]
    #[serial]
    async fn test_get_some() {
        let mut cache = get_cache().await;
        cache.set("apples".to_string(), 5).await;
        cache.set("strawberries".to_string(), -142).await;
        cache.set("apples".to_string(), 3).await;
        assert_eq!(cache.get("apples".to_string()).await, Some(3));
    }

    #[tokio::test]
    #[ignore]
    #[serial]
    async fn test_get_none() {
        let mut cache = get_cache().await;
        cache.set("apples".to_string(), 5).await;
        cache.set("strawberries".to_string(), -142).await;
        cache.set("apples".to_string(), 3).await;
        assert_eq!(cache.get("lemons".to_string()).await, None);
    }

    #[tokio::test]
    #[ignore]
    #[serial]
    async fn test_remove() {
        let mut cache = get_cache().await;
        cache.set("apples".to_string(), 5).await;
        cache.set("strawberries".to_string(), -142).await;
        cache.set("apples".to_string(), 3).await;
        cache.remove("apples".to_string()).await;
        assert_eq!(cache.get("apples".to_string()).await, None);
    }
}
