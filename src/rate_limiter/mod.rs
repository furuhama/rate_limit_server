use std::time::Instant;

#[derive(Debug, Clone)]
pub struct RequestState {
    pub count: u32,
    pub last_updated: Instant,
}

// レート制限のトレイト
pub trait RateLimiter: Clone {
    async fn check_rate_limit(&self, ip: &str) -> Result<(), String>;
    async fn record_request(&self, ip: &str);
}

mod lock_free;
mod standard;

pub use lock_free::*;
pub use standard::*;

#[derive(Clone)]
pub enum RateLimiterEnum {
    Standard(SlidingWindowRateLimiter),
    LockFree(LockFreeSlidingWindowRateLimiter),
}

impl RateLimiterEnum {
    pub async fn check_rate_limit(&self, ip: &str) -> Result<(), String> {
        match self {
            Self::Standard(limiter) => limiter.check_rate_limit(ip).await,
            Self::LockFree(limiter) => limiter.check_rate_limit(ip).await,
        }
    }

    pub async fn record_request(&self, ip: &str) {
        match self {
            Self::Standard(limiter) => limiter.record_request(ip).await,
            Self::LockFree(limiter) => limiter.record_request(ip).await,
        }
    }
}
