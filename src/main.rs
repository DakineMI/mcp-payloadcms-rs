mod server;
mod payload;

use crate::server::SoftwarePlanningServer;
use hyper_util::{
    rt::{TokioExecutor, TokioIo},
    server::conn::auto::Builder,
    service::TowerToHyperService,
};
use rmcp::transport::sse_server::SseServer;
use rmcp::transport::streamable_http_server::{
    StreamableHttpService, session::local::LocalSessionManager,
};
use rmcp::{ServiceExt, transport::stdio};
use std::net::SocketAddr;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| "info".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    tracing::info!("Starting MCP Payload MCP Server (Rust)");

    // Stdio transport - single instance
    let service = SoftwarePlanningServer::new();
    let std_service = service.clone().serve(stdio()).await?;
    let std_handle = tokio::spawn(async move {
        let _ = std_service.waiting().await;
    });

    // HTTP streamable transport using rmcp tower StreamableHttpService + hyper_util
    let http_service = TowerToHyperService::new(StreamableHttpService::new(
        || Ok(SoftwarePlanningServer::new()),
        LocalSessionManager::default().into(),
        Default::default(),
    ));
    let http_listener = tokio::net::TcpListener::bind("0.0.0.0:8080").await?;
    let http_handle = tokio::spawn(async move {
        loop {
            let (stream, _) = match http_listener.accept().await {
                Ok((s, a)) => (s, a),
                Err(_) => continue,
            };
            let io = TokioIo::new(stream);
            let service = http_service.clone();
            tokio::spawn(async move {
                let _ = Builder::new(TokioExecutor::default())
                    .serve_connection(io, service)
                    .await;
            });
        }
    });

    // SSE
    let sse_listener_addr: SocketAddr = "0.0.0.0:8081".parse()?;
    let sse = SseServer::serve(sse_listener_addr).await?;
    let _sse_disable = sse.with_service_directly(|| SoftwarePlanningServer::new());

    // Wait for the std or http tasks to finish
    let _ = tokio::join!(std_handle, http_handle);

    Ok(())
}
