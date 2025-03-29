use axum::{
    body::Body,
    extract::State,
    http::{Request, Response, StatusCode},
    middleware::Next,
    response::IntoResponse,
};

use crate::rate_limiter::{
    LockFreeRateLimitState, LockFreeSlidingWindowRateLimiter, RateLimitState, RateLimiter,
    RateLimiterEnum, SlidingWindowRateLimiter,
};

#[derive(Clone)]
pub enum RateLimitStateEnum {
    Standard(RateLimitState),
    LockFree(LockFreeRateLimitState),
}

pub async fn rate_limit_middleware(
    State(state): State<RateLimitStateEnum>,
    req: Request<Body>,
    next: Next,
) -> Response<Body> {
    // クライアントのIPアドレスを取得
    let ip = req
        .headers()
        .get("x-forwarded-for")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("unknown");

    // リクエスト情報のログ出力
    let path = req.uri().path();
    tracing::info!("Incoming request - IP: {}, Path: {}", ip, path);

    // レート制限のチェック
    let limiter = match state {
        RateLimitStateEnum::Standard(state) => {
            RateLimiterEnum::Standard(SlidingWindowRateLimiter::new(state.requests))
        }
        RateLimitStateEnum::LockFree(state) => {
            RateLimiterEnum::LockFree(LockFreeSlidingWindowRateLimiter::new(state.requests))
        }
    };

    match limiter.check_rate_limit(ip).await {
        Ok(_) => {
            limiter.record_request(ip).await;
            tracing::info!("Rate limit check passed for IP: {}", ip);
            next.run(req).await
        }
        Err(message) => {
            tracing::warn!("Rate limit exceeded for IP: {}", ip);
            (StatusCode::TOO_MANY_REQUESTS, message).into_response()
        }
    }
}
