use dashmap::DashMap;
use std::{
    sync::Arc,
    time::{Duration, Instant},
};

use super::{RateLimiter, RequestState};
use crate::config::RateLimitConfig;

#[derive(Clone)]
pub struct LockFreeRateLimitState {
    pub requests: Arc<DashMap<String, RequestState>>,
}

impl LockFreeRateLimitState {
    pub fn new() -> Self {
        Self {
            requests: Arc::new(DashMap::new()),
        }
    }
}

#[derive(Clone)]
pub struct LockFreeSlidingWindowRateLimiter {
    requests: Arc<DashMap<String, RequestState>>,
    config: &'static RateLimitConfig,
}

impl LockFreeSlidingWindowRateLimiter {
    pub fn new(requests: Arc<DashMap<String, RequestState>>) -> Self {
        Self {
            requests,
            config: &crate::config::RATE_LIMIT_CONFIG,
        }
    }
}

impl RateLimiter for LockFreeSlidingWindowRateLimiter {
    async fn check_rate_limit(&self, ip: &str) -> Result<(), String> {
        let now = Instant::now();
        let window = Duration::from_secs(self.config.window_seconds);

        // Check request count while tolerating race conditions
        if let Some(mut entry) = self.requests.get_mut(ip) {
            let duration_since_last = now.duration_since(entry.last_updated);

            // Reset counter if window is exceeded
            if duration_since_last >= window {
                entry.count = 0;
                entry.last_updated = now;
            }

            if entry.count >= self.config.max_requests {
                return Err(format!(
                    "Rate limit exceeded. Maximum {} requests per {} seconds.",
                    self.config.max_requests, self.config.window_seconds
                ));
            }
        }

        Ok(())
    }

    async fn record_request(&self, ip: &str) {
        let now = Instant::now();
        self.requests
            .entry(ip.to_string())
            .and_modify(|state| {
                state.count += 1;
                state.last_updated = now;
            })
            .or_insert(RequestState {
                count: 1,
                last_updated: now,
            });
    }
}
