# lynx-balancer

## TODO:

 * **CacheManager**
   * [X] Complete trait
   * [X] Local cache
   * [ ] Local cache cross-instance communication
   * [ ] Redis cache (Optionally manual/atomic)
   * [ ] Hybrid cache
 * **InstanceHost**
   * [X] Complete trait
   * [X] Kubernetes host based on pods
   * [ ] ~Docker host using local docker engine~
   * [ ] ~[Optional] Docker host in a separate container with autoscaling on a stateful set.~
   * [ ] Local process host
 * **PersistenceManager**
   * [ ] Complete trait
   * [ ] PostgreSQL implementation
 * Others:
   * [x] Commandline arguments parsing
   * [ ] [Optional] Config file
   * [X] [Optional] Proxy redirect 
   * [ ] Latency benchmark
