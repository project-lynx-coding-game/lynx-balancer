use async_trait::async_trait;
use serde::{Deserialize, Serialize};

pub mod local_cache;

#[derive(Debug, Serialize, Deserialize)]
pub struct CacheGetRequest<K> {
    pub key: K,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CacheSetRequest<K, V> {
    pub key: K,
    pub value: V,
}

#[async_trait]
pub trait CacheProvider<K, V> {
    async fn set(&mut self, key: K, value: V);
    async fn get(&mut self, key: K) -> Option<V>;
    async fn remove(&mut self, key: K);
}
