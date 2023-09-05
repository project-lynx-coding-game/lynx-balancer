use serde::{Serialize, Deserialize};

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

pub trait CacheProvider<K, V> {
    fn set(&mut self, key: K, value: V);
    fn get(&self, key: &K) -> Option<&V>;
    fn remove(&mut self, key: &K);
}
