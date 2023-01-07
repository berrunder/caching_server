# Caching proxy
Simple implementation of reverse proxy caching server in Rust for learning purpose.

Caches all incoming request by HTTP method and path. Uses LRU caching algorithm, default size of cache is 4096 items. Spawns 4 threads to process incoming requests.

### Configuration

To configure backend server address, you need to create config.txt with its address in format `127.0.0.1:3000`