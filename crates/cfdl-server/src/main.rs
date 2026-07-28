//! Entry point for the CFDL API server.
//!
//! Binds to `CFDL_SERVER_ADDR` (default `0.0.0.0:8080`).

use std::net::SocketAddr;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| "info".into()),
        )
        .init();

    let addr: SocketAddr = std::env::var("CFDL_SERVER_ADDR")
        .unwrap_or_else(|_| "0.0.0.0:8080".to_string())
        .parse()
        .expect("valid CFDL_SERVER_ADDR");

    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .expect("bind server address");
    tracing::info!("cfdl-server listening on http://{addr} (docs at /docs)");

    axum::serve(listener, cfdl_server::app())
        .with_graceful_shutdown(shutdown_signal())
        .await
        .expect("server error");
}

async fn shutdown_signal() {
    let _ = tokio::signal::ctrl_c().await;
    tracing::info!("shutdown signal received");
}
