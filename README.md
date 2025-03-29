# Rate Limit Server

A simple HTTP server with rate limiting functionality implemented in Rust using the Axum framework.

## Features

- HTTP server with rate limiting middleware
- IP-based rate limiting
- Configurable time window and request limits
- Thread-safe request tracking using `Arc<RwLock>`

## Rate Limiting Configuration

The server supports configuration through environment variables:

- `RATE_LIMIT_MAX_REQUESTS`: Maximum number of requests allowed per time window (default: 3)
- `RATE_LIMIT_WINDOW_SECONDS`: Time window in seconds (default: 5)

Example:
```bash
RATE_LIMIT_MAX_REQUESTS=20 RATE_LIMIT_WINDOW_SECONDS=60 cargo run
```

This will set the rate limit to 20 requests per 30 seconds.

## Testing

You can test the server using curl or a web browser:

```bash
# Send a request
curl http://localhost:3000

# Test rate limiting (send multiple requests quickly)
while true; do curl localhost:3000; sleep 1; done
```

## Implementation Details

The server provides two different rate limiting implementations that can be switched using environment variables:

### Standard Implementation (RwLock-based)
- Uses `Arc<RwLock<HashMap>>` for thread-safe request tracking
- Provides strict rate limiting with precise request counting
- Suitable for scenarios where exact rate limiting is required
- Enable with: `RATE_LIMITER_TYPE=standard cargo run`

### Lock-Free Implementation (DashMap-based)
- Uses `DashMap` for high-throughput concurrent access
- Trades strict rate limiting for better performance
- Suitable for high-traffic scenarios where approximate rate limiting is acceptable
- Enable with: `RATE_LIMITER_TYPE=lock_free cargo run` (default)

Both implementations use a sliding window approach:
- Each IP address's requests are tracked separately
- Old requests are automatically cleaned up
- The server uses Axum's middleware system for rate limiting

### Configuration Example

```bash
# Run with standard implementation and custom limits
RATE_LIMITER_TYPE=standard RATE_LIMIT_MAX_REQUESTS=20 RATE_LIMIT_WINDOW_SECONDS=60 cargo run

# Run with lock-free implementation (default)
RATE_LIMITER_TYPE=lock_free cargo run
```

## License

MIT License
