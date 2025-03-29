use axum::{
    Router,
    body::Body,
    extract::State,
    http::{Request, Response, StatusCode},
    middleware::Next,
    response::IntoResponse,
    routing::get,
};
use std::{
    collections::HashMap,
    env,
    net::SocketAddr,
    sync::Arc,
    sync::LazyLock,
    time::{Duration, Instant},
};
use tokio::sync::RwLock;
use tower::ServiceBuilder;

const DEFAULT_MAX_REQUESTS: u32 = 10;
const DEFAULT_WINDOW_SECONDS: u64 = 60;

#[derive(Clone)]
struct RateLimitConfig {
    max_requests: u32,
    window_seconds: u64,
}

impl Default for RateLimitConfig {
    fn default() -> Self {
        Self {
            max_requests: DEFAULT_MAX_REQUESTS,
            window_seconds: DEFAULT_WINDOW_SECONDS,
        }
    }
}

static RATE_LIMIT_CONFIG: LazyLock<RateLimitConfig> = LazyLock::new(|| RateLimitConfig {
    max_requests: env::var("RATE_LIMIT_MAX_REQUESTS")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(DEFAULT_MAX_REQUESTS),
    window_seconds: env::var("RATE_LIMIT_WINDOW_SECONDS")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(DEFAULT_WINDOW_SECONDS),
});

#[derive(Clone)]
struct RateLimitState {
    requests: Arc<RwLock<HashMap<String, Vec<Instant>>>>,
}

// レート制限のトレイト
trait RateLimiter {
    async fn check_rate_limit(&self, ip: &str) -> Result<(), String>;
    async fn record_request(&self, ip: &str);
}

// スライディングウィンドウ方式のレート制限
struct SlidingWindowRateLimiter {
    requests: Arc<RwLock<HashMap<String, Vec<Instant>>>>,
    config: &'static RateLimitConfig,
}

impl SlidingWindowRateLimiter {
    fn new(requests: Arc<RwLock<HashMap<String, Vec<Instant>>>>) -> Self {
        Self {
            requests,
            config: &RATE_LIMIT_CONFIG,
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

#[tokio::main]
async fn main() {
    // ロギングの初期化
    tracing_subscriber::fmt::init();

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

async fn rate_limit_middleware(
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

async fn handler() -> &'static str {
    "Hello, World!"
}
