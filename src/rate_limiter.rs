use dashmap::DashMap;
use std::{
    collections::HashMap,
    sync::Arc,
    time::{Duration, Instant},
};
use tokio::sync::RwLock;

use crate::config::RateLimitConfig;

// ロックフリーなレート制限のためのデータ構造
#[derive(Debug, Clone)]
pub struct RequestState {
    count: u32,
    last_updated: Instant,
}

#[derive(Clone)]
pub struct RateLimitState {
    pub requests: Arc<RwLock<HashMap<String, Vec<Instant>>>>,
}

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

// レート制限のトレイト
pub trait RateLimiter: Clone {
    async fn check_rate_limit(&self, ip: &str) -> Result<(), String>;
    async fn record_request(&self, ip: &str);
}

// ロックフリーなスライディングウィンドウ方式のレート制限
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

        // レースコンディションを許容しつつ、リクエスト数をチェック
        if let Some(mut entry) = self.requests.get_mut(ip) {
            let duration_since_last = now.duration_since(entry.last_updated);

            // ウィンドウを超えた場合はカウンターをリセット
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
