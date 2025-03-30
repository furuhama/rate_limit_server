use axum::{Router, routing::get};
use std::net::SocketAddr;
use tower::ServiceBuilder;

mod config;
mod middleware;
mod rate_limiter;

use config::{RATE_LIMIT_CONFIG, RATE_LIMITER_TYPE, RateLimiterType};
use middleware::RateLimitStateEnum;
use rate_limiter::{LockFreeRateLimitState, RateLimitState};
use std::{collections::HashMap, sync::Arc};
use tokio::sync::RwLock;

async fn handler() -> &'static str {
    "Hello, World!"
}

#[tokio::main]
async fn main() {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .with_target(false)
        .with_thread_ids(true)
        .with_file(true)
        .with_line_number(true)
        .init();

    // Select rate limiter implementation based on environment variable
    let state = match *RATE_LIMITER_TYPE {
        RateLimiterType::Standard => {
            tracing::info!("Using standard rate limiter");
            RateLimitStateEnum::Standard(RateLimitState {
                requests: Arc::new(RwLock::new(HashMap::new())),
            })
        }
        RateLimiterType::LockFree => {
            tracing::info!("Using lock-free rate limiter");
            RateLimitStateEnum::LockFree(LockFreeRateLimitState::new())
        }
    };

    let middleware = ServiceBuilder::new().layer(axum::middleware::from_fn_with_state(
        state.clone(),
        middleware::rate_limit_middleware,
    ));

    let app = Router::new()
        .route("/", get(handler))
        .layer(middleware)
        .with_state(state);

    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    tracing::info!("listening on {}", addr);
    tracing::info!("rate limiter type: {:?}", *RATE_LIMITER_TYPE);
    tracing::info!(
        "rate limit config: {} requests per {} seconds",
        RATE_LIMIT_CONFIG.max_requests,
        RATE_LIMIT_CONFIG.window_seconds
    );
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
