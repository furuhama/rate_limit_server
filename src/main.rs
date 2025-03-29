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
    net::SocketAddr,
    sync::Arc,
    time::{Duration, Instant},
};
use tokio::sync::RwLock;
use tower::ServiceBuilder;

#[derive(Clone)]
struct RateLimitState {
    requests: Arc<RwLock<HashMap<String, Vec<Instant>>>>,
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
        .unwrap_or("unknown")
        .to_string();

    // レート制限のチェック
    let mut requests = state.requests.write().await;
    let now = Instant::now();
    let window = Duration::from_secs(60); // 1分間のウィンドウ
    let max_requests = 10; // 最大リクエスト数

    // 古いリクエストを削除
    if let Some(timestamps) = requests.get_mut(&ip) {
        timestamps.retain(|&time| now.duration_since(time) <= window);
    }

    // 現在のリクエスト数を取得
    let current_requests = requests.get(&ip).map(|v| v.len()).unwrap_or(0);

    if current_requests >= max_requests {
        return (
            StatusCode::TOO_MANY_REQUESTS,
            "Rate limit exceeded. Please try again later.",
        )
            .into_response();
    }

    // 新しいリクエストを記録
    requests.entry(ip).or_insert_with(Vec::new).push(now);

    // 次のミドルウェアまたはハンドラーを実行
    next.run(req).await
}

async fn handler() -> &'static str {
    "Hello, World!"
}
