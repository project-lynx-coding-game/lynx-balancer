# CacheProvider

## RedisCache

Few remarks worth remembering:
 * Using the cache requires `redis_url`. Use following commandline to set it up:
```bash
kubectl port-forward my-redis-master-0 6379:6379
# -- Different terminal session --
export REDIS_PASSWORD=$(kubectl get secret --namespace lynx-balancer my-redis -o jsonpath="{.data.redis-password}" | base64 -d)
cargo run -- --redis-url=redis://default:${REDIS_PASSWORD}@127.0.0.1:6379
```
 * Tests are hidden by `#[ignore]`, so they do not break CI on github. To run the tests use:
```bash
kubectl port-forward my-redis-master-0 6379:6379
# -- Different terminal session --
export REDIS_PASSWORD=$(kubectl get secret --namespace lynx-balancer my-redis -o jsonpath="{.data.redis-password}" | base64 -d)
cargo test -- --ignored
```