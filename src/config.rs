use std::env;
use std::sync::LazyLock;

const DEFAULT_MAX_REQUESTS: u32 = 3;
const DEFAULT_WINDOW_SECONDS: u64 = 5;
const DEFAULT_RATE_LIMITER_TYPE: &str = "lock_free"; // デフォルトはロックフリー実装

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum RateLimiterType {
    Standard,
    LockFree,
}

impl RateLimiterType {
    pub fn from_env() -> Self {
        match env::var("RATE_LIMITER_TYPE").as_deref() {
            Ok("standard") => Self::Standard,
            Ok("lock_free") | _ => Self::LockFree,
        }
    }
}

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

pub static RATE_LIMITER_TYPE: LazyLock<RateLimiterType> = LazyLock::new(RateLimiterType::from_env);

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
