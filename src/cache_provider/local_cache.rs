use std::collections::HashMap;
use std::hash::Hash;

use async_trait::async_trait;

use crate::cache_provider::CacheProvider;

pub struct LocalCache<K: Eq + Hash, V> {
    map: HashMap<K, V>,
}

impl<K: Eq + Hash, V> LocalCache<K, V> {
    pub fn new() -> LocalCache<K, V> {
        LocalCache {
            map: HashMap::new(),
        }
    }
}

#[async_trait]
impl<K: Sync+Send+Eq + Hash, V:Sync+Send+Clone> CacheProvider<K, V> for LocalCache<K, V> {
    async fn set(&mut self, key: K, value: V) {
        self.map.insert(key, value);
    }

    async fn get(&mut self, key: K) -> Option<V> {
        self.map.get(&key).map(V::to_owned)
    }

    async fn remove(&mut self, key: K) {
        self.map.remove(&key);
    }
}
#[cfg(test)]
mod tests {
    use futures::executor;

    // Note this useful idiom: importing names from outer (for mod tests) scope.
    use super::*;

    #[test]
    fn test_set() {
        let mut cache = LocalCache::<String, i32>::new();
        executor::block_on(cache.set("apples".to_string(), 5));
        executor::block_on(cache.set("strawberries".to_string(), -142));
        executor::block_on(cache.set("apples".to_string(), 3));
        assert_eq!(cache.map.get(&"apples".to_string()).unwrap().to_owned(), 3);
    }

    #[test]
    fn test_get_some() {
        let mut cache = LocalCache::<String, i32>::new();
        executor::block_on(cache.set("apples".to_string(), 5));
        executor::block_on(cache.set("strawberries".to_string(), -142));
        executor::block_on(cache.set("apples".to_string(), 3));
        assert_eq!(executor::block_on(cache.get("apples".to_string())), Some(3));
    }

    #[test]
    fn test_get_none() {
        let mut cache = LocalCache::<String, i32>::new();
        executor::block_on(cache.set("apples".to_string(), 5));
        executor::block_on(cache.set("strawberries".to_string(), -142));
        executor::block_on(cache.set("apples".to_string(), 3));
        assert_eq!(executor::block_on(cache.get("lemons".to_string())), None);
    }

    #[test]
    fn test_remove() {
        let mut cache = LocalCache::<String, i32>::new();
        executor::block_on(cache.set("apples".to_string(), 5));
        executor::block_on(cache.set("strawberries".to_string(), -142));
        executor::block_on(cache.set("apples".to_string(), 3));
        executor::block_on(cache.remove("apples".to_string()));
        assert_eq!(executor::block_on(cache.get("apples".to_string())), None);
    }
}
