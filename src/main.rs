mod http;

use std::net::SocketAddr;
use tracing_subscriber::EnvFilter;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "iconator=info,tower_http=debug".into()),
        )
        .init();

    let app = http::router();
    let addr: SocketAddr = std::env::var("BIND_ADDR")
        .unwrap_or_else(|_| "0.0.0.0:3000".to_string())
        .parse()
        .expect("invalid BIND_ADDR");

    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .expect("failed to bind to address");
    tracing::info!("iconator listening on {addr}");
    axum::serve(listener, app).await.expect("server error");
}
