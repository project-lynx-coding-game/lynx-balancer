pub trait CacheProvider<K, V> {
    pub fn set(key: K, value: V);
    pub fn get(key: K) -> Option<V>;
}
