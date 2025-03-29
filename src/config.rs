use std::env;
use std::sync::LazyLock;

const DEFAULT_MAX_REQUESTS: u32 = 10;
const DEFAULT_WINDOW_SECONDS: u64 = 60;

#[derive(Clone)]
pub struct RateLimitConfig {
    pub max_requests: u32,
    pub window_seconds: u64,
}

impl Default for RateLimitConfig {
    fn default() -> Self {
        Self {
            max_requests: DEFAULT_MAX_REQUESTS,
            window_seconds: DEFAULT_WINDOW_SECONDS,
        }
    }
}

pub static RATE_LIMIT_CONFIG: LazyLock<RateLimitConfig> = LazyLock::new(|| RateLimitConfig {
    max_requests: env::var("RATE_LIMIT_MAX_REQUESTS")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(DEFAULT_MAX_REQUESTS),
    window_seconds: env::var("RATE_LIMIT_WINDOW_SECONDS")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(DEFAULT_WINDOW_SECONDS),
});
