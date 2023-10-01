use std::collections::HashMap;
use std::hash::Hash;
use std::str::FromStr;

use async_trait::async_trait;
use tracing::{error, info};

use crate::cache_provider::CacheProvider;

pub struct LocalCache<K: Eq + Hash, V> {
    cache_server_url: Option<String>,
    map: HashMap<K, V>,
}

impl<K: Eq + Hash, V> LocalCache<K, V> {
    pub fn new(cache_server_url: Option<String>) -> LocalCache<K, V> {
        LocalCache {
            cache_server_url: cache_server_url,
            map: HashMap::new(),
        }
    }
}

static MAX_ITERS: u32 = 32;

#[async_trait(?Send)]
impl<K: Sync + Send + Eq + Hash + ToString, V: Sync + Send + Clone + FromStr> CacheProvider<K, V>
    for LocalCache<K, V>
{
    async fn set(&mut self, key: K, value: V) {
        self.map.insert(key, value);
    }

    async fn get(&mut self, key: K) -> Option<V> {
        self.map.get(&key).map(V::to_owned)
    }

    async fn get_or_query(&mut self, key: K) -> Option<V> {
        if let Some(v) = self.map.get(&key) {
            return Some(v.to_owned());
        }

        let cache_server_url;
        if let Some(url) = &self.cache_server_url {
            cache_server_url = url;
        } else {
            error!("Tried `get_or_query` on LocalCache without `cache_server_url`!");
            return None;
        }

        let mut iters = 0;
        while iters < MAX_ITERS {
            let client = awc::Client::new();
            let params = [("key", "bar")];
            let final_url =
                "http://".to_owned() + &cache_server_url + "/cache/get?key=" + &key.to_string();
            let request = client.get(final_url).send_form(&params);
            let response = request.await;

            let mut body = None;
            if let Some(response_body) = response
                .ok()
                .filter(|response| response.status() == awc::http::StatusCode::OK)
                .map(|mut response| response.body())
            {
                body = Some(response_body.await)
            }

            let value = body
                .map(|result| result.ok())
                .flatten()
                .map(|bytes| String::from_utf8(bytes.to_vec()).ok())
                .flatten()
                .map(|string| V::from_str(&string).ok())
                .flatten();
            if value.is_some() {
                info!("Found value after {} iterations", iters);
                return value;
            }

            iters += 1;
        }
        error!("Could not fetch from cache after {} attempts", MAX_ITERS);
        None
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
        let mut cache = LocalCache::<String, i32>::new(None);
        executor::block_on(cache.set("apples".to_string(), 5));
        executor::block_on(cache.set("strawberries".to_string(), -142));
        executor::block_on(cache.set("apples".to_string(), 3));
        assert_eq!(cache.map.get(&"apples".to_string()).unwrap().to_owned(), 3);
    }

    #[test]
    fn test_get_some() {
        let mut cache = LocalCache::<String, i32>::new(None);
        executor::block_on(cache.set("apples".to_string(), 5));
        executor::block_on(cache.set("strawberries".to_string(), -142));
        executor::block_on(cache.set("apples".to_string(), 3));
        assert_eq!(executor::block_on(cache.get("apples".to_string())), Some(3));
    }

    #[test]
    fn test_get_none() {
        let mut cache = LocalCache::<String, i32>::new(None);
        executor::block_on(cache.set("apples".to_string(), 5));
        executor::block_on(cache.set("strawberries".to_string(), -142));
        executor::block_on(cache.set("apples".to_string(), 3));
        assert_eq!(executor::block_on(cache.get("lemons".to_string())), None);
    }

    #[test]
    fn test_remove() {
        let mut cache = LocalCache::<String, i32>::new(None);
        executor::block_on(cache.set("apples".to_string(), 5));
        executor::block_on(cache.set("strawberries".to_string(), -142));
        executor::block_on(cache.set("apples".to_string(), 3));
        executor::block_on(cache.remove("apples".to_string()));
        assert_eq!(executor::block_on(cache.get("apples".to_string())), None);
    }
}
