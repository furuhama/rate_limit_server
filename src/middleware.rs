use axum::{
    body::Body,
    extract::State,
    http::{Request, Response, StatusCode},
    middleware::Next,
    response::IntoResponse,
};

use crate::rate_limiter::{RateLimitState, RateLimiter, SlidingWindowRateLimiter};

pub async fn rate_limit_middleware(
    State(state): State<RateLimitState>,
    req: Request<Body>,
    next: Next,
) -> Response<Body> {
    // クライアントのIPアドレスを取得
    let ip = req
        .headers()
        .get("x-forwarded-for")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("unknown");

    // レート制限のチェック
    let limiter = SlidingWindowRateLimiter::new(state.requests.clone());
    match limiter.check_rate_limit(ip).await {
        Ok(_) => {
            limiter.record_request(ip).await;
            next.run(req).await
        }
        Err(message) => (StatusCode::TOO_MANY_REQUESTS, message).into_response(),
    }
}
