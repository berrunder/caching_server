# Caching proxy
Simple implementation of reverse proxy caching server in Rust for learning purpose.

Caches all incoming GET request by HTTP method and path. Uses LRU or LFU caching algorithm, default size of cache is 4096 items. Spawns 4 threads by default to process incoming requests.

### Configuration

To configure backend server address, you need to create config.txt with its address in format `127.0.0.1:3000`

To configure number of threads, use CACHING_SERVER_N_THREADS environment variable

To configure maximum cache size, use CACHING_SERVER_MAX_CACHE_SIZE environment variable

To configure caching alghoritm, use `-a` or `--algo` command-line option. Possible values: `LFU`, `LRU` (default `LRU`)