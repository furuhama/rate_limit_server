# Rate Limit Server

A simple HTTP server with rate limiting functionality implemented in Rust using the Axum framework.

## Features

- HTTP server with rate limiting middleware
- IP-based rate limiting
- Configurable time window and request limits
- Thread-safe request tracking using `Arc<RwLock>`

## Rate Limiting Configuration

The server supports configuration through environment variables:

- `RATE_LIMIT_MAX_REQUESTS`: Maximum number of requests allowed per time window (default: 10)
- `RATE_LIMIT_WINDOW_SECONDS`: Time window in seconds (default: 60)

Example:
```bash
RATE_LIMIT_MAX_REQUESTS=20 RATE_LIMIT_WINDOW_SECONDS=30 cargo run
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

The rate limiting is implemented using a sliding window approach:
- Each IP address's requests are tracked in a thread-safe HashMap
- Old requests are automatically cleaned up
- The server uses Axum's middleware system for rate limiting

## License

MIT License
