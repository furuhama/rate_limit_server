use std::{
    collections::HashMap,
    sync::Arc,
    time::{Duration, Instant},
};
use tokio::sync::RwLock;

use super::RateLimiter;
use crate::config::RateLimitConfig;

#[derive(Clone)]
pub struct RateLimitState {
    pub requests: Arc<RwLock<HashMap<String, Vec<Instant>>>>,
}

// スライディングウィンドウ方式のレート制限
#[derive(Clone)]
pub struct SlidingWindowRateLimiter {
    requests: Arc<RwLock<HashMap<String, Vec<Instant>>>>,
    config: &'static RateLimitConfig,
}

impl SlidingWindowRateLimiter {
    pub fn new(requests: Arc<RwLock<HashMap<String, Vec<Instant>>>>) -> Self {
        Self {
            requests,
            config: &crate::config::RATE_LIMIT_CONFIG,
        }
    }
}

impl RateLimiter for SlidingWindowRateLimiter {
    async fn check_rate_limit(&self, ip: &str) -> Result<(), String> {
        let mut requests = self.requests.write().await;
        let now = Instant::now();
        let window = Duration::from_secs(self.config.window_seconds);

        // 古いリクエストを削除
        if let Some(timestamps) = requests.get_mut(ip) {
            timestamps.retain(|&time| now.duration_since(time) <= window);
        }

        // 現在のリクエスト数を取得
        let current_requests = requests.get(ip).map(|v| v.len()).unwrap_or(0);

        if current_requests >= self.config.max_requests as usize {
            Err(format!(
                "Rate limit exceeded. Maximum {} requests per {} seconds.",
                self.config.max_requests, self.config.window_seconds
            ))
        } else {
            Ok(())
        }
    }

    async fn record_request(&self, ip: &str) {
        let mut requests = self.requests.write().await;
        requests
            .entry(ip.to_string())
            .or_insert_with(Vec::new)
            .push(Instant::now());
    }
}
