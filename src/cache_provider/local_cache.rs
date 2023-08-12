use std::collections::HashMap;
use std::hash::Hash;

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

impl<K: Eq + Hash, V> CacheProvider<K, V> for LocalCache<K, V> {
    fn set(&mut self, key: K, value: V) {
        self.map.insert(key, value);
    }

    fn get(&self, key: &K) -> Option<&V> {
        self.map.get(key)
    }

    fn remove(&mut self, key: &K) {
        self.map.remove(key);
    }
}
#[cfg(test)]
mod tests {
    // Note this useful idiom: importing names from outer (for mod tests) scope.
    use super::*;

    #[test]
    fn test_set() {
        let mut cache = LocalCache::<String, i32>::new();
        cache.set("apples".to_string(), 5);
        cache.set("strawberries".to_string(), -142);
        cache.set("apples".to_string(), 3);
        assert_eq!(cache.map.get(&"apples".to_string()).unwrap().to_owned(), 3);
    }

    #[test]
    fn test_get_some() {
        let mut cache = LocalCache::<String, i32>::new();
        cache.set("apples".to_string(), 5);
        cache.set("strawberries".to_string(), -142);
        cache.set("apples".to_string(), 3);
        assert_eq!(cache.get(&"apples".to_string()), Some::<&i32>(&3));
    }

    #[test]
    fn test_get_none() {
        let mut cache = LocalCache::<String, i32>::new();
        cache.set("apples".to_string(), 5);
        cache.set("strawberries".to_string(), -142);
        cache.set("apples".to_string(), 3);
        assert_eq!(cache.get(&"lemons".to_string()), None);
    }

    #[test]
    fn test_remove() {
        let mut cache = LocalCache::<String, i32>::new();
        cache.set("apples".to_string(), 5);
        cache.set("strawberries".to_string(), -142);
        cache.set("apples".to_string(), 3);
        cache.remove(&"apples".to_string());
        assert_eq!(cache.get(&"apples".to_string()), None);
    }
}
