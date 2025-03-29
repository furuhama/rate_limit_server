use axum::{Router, routing::get};
use std::{collections::HashMap, net::SocketAddr, sync::Arc};
use tokio::sync::RwLock;
use tower::ServiceBuilder;

mod config;
mod middleware;
mod rate_limiter;

use config::RATE_LIMIT_CONFIG;
use middleware::rate_limit_middleware;
use rate_limiter::RateLimitState;

async fn handler() -> &'static str {
    "Hello, World!"
}

#[tokio::main]
async fn main() {
    // ロギングの初期化
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .with_target(false)
        .with_thread_ids(true)
        .with_file(true)
        .with_line_number(true)
        .init();

    // レート制限の状態管理
    let state = RateLimitState {
        requests: Arc::new(RwLock::new(HashMap::new())),
    };

    // ミドルウェアの構築
    let middleware = ServiceBuilder::new().layer(axum::middleware::from_fn_with_state(
        state.clone(),
        rate_limit_middleware,
    ));

    // ルーターの構築
    let app = Router::new()
        .route("/", get(handler))
        .layer(middleware)
        .with_state(state);

    // サーバーの起動
    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    tracing::info!("listening on {}", addr);
    tracing::info!(
        "rate limit config: {} requests per {} seconds",
        RATE_LIMIT_CONFIG.max_requests,
        RATE_LIMIT_CONFIG.window_seconds
    );
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
