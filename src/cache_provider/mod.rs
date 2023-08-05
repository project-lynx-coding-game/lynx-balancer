pub mod local_cache;

pub trait CacheProvider<K, V> {
    fn set(&mut self, key: K, value: V);
    fn get(&self, key: &K) -> Option<&V>;
    fn remove(&mut self, key: &K);
}
