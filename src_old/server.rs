use rmcp::ServiceExt;
use rmcp::handler::server::router::Router;
use rmcp::model::{Implementation, InitializeResult, ProtocolVersion, ServerCapabilities};

use crate::{
    cli::CommandArguments,
    error::{ServiceError, ServiceResult},
    handler::MyServerHandler,
};
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::net::TcpListener;
#[cfg(unix)]
use tokio::net::UnixListener;
use tokio::task::JoinSet;
use rmcp::transport::streamable_http_server::{
    session::local::LocalSessionManager, StreamableHttpServerConfig, StreamableHttpService,
};
use rmcp::transport::sse_server::{SseServer, SseServerConfig};
use hyper_util::{
    rt::{TokioExecutor, TokioIo},
    server::conn::auto::Builder as HyperBuilder,
    service::TowerToHyperService,
};
use tokio_util::sync::CancellationToken;

pub fn server_details() -> InitializeResult {
    InitializeResult {
        server_info: Implementation {
            name: crate::metadata::PKG_NAME.to_string(),
            version: crate::metadata::PKG_VERSION.to_string(),
            title: Some(crate::metadata::PKG_DESCRIPTION.to_string()),
            icons: None,
            website_url: None,
        },
        capabilities: ServerCapabilities {
            experimental: None,
            logging: None,
            prompts: None,
            resources: None,
            tools: Some(rmcp::model::ToolsCapability::default()),
            completions: None,
        },
        instructions: MyServerHandler::instructions_content(),
        protocol_version: ProtocolVersion::default(),
    }
}

pub async fn start_server(_args: CommandArguments) -> ServiceResult<()> {
    let mut tasks: JoinSet<ServiceResult<()>> = JoinSet::new();

    // stdio transport
    tasks.spawn(async {
        let handler = MyServerHandler::try_new()?;
        let router = Router::new(handler);
        router
            .serve((tokio::io::stdin(), tokio::io::stdout()))
            .await
            .map_err(|e| ServiceError::FromString(format!("Stdio server error: {e}")))?;
        Ok(())
    });

    // plain TCP transport
    tasks.spawn(async {
        let addr: SocketAddr = std::env::var("MCP_TCP_ADDR")
            .unwrap_or_else(|_| "127.0.0.1:3003".to_string())
            .parse()
            .map_err(|e| ServiceError::FromString(format!("Invalid MCP_TCP_ADDR: {e}")))?;
        let listener = TcpListener::bind(addr)
            .await
            .map_err(|e| ServiceError::FromString(format!("TCP listen error: {e}")))?;
        loop {
            let (stream, _) = listener
                .accept()
                .await
                .map_err(|e| ServiceError::FromString(format!("TCP accept error: {e}")))?;
            tokio::spawn(async move {
                let handler = match MyServerHandler::try_new() {
                    Ok(h) => h,
                    Err(e) => {
                        tracing::warn!("Failed to init handler for TCP connection: {e}");
                        return;
                    }
                };
                let router = Router::new(handler);
                let (read, write) = stream.into_split();
                if let Err(err) = router
                    .serve((read, write))
                    .await
                    .map_err(|e| ServiceError::FromString(format!("TCP server error: {e}")))
                {
                    tracing::warn!("TCP connection error: {err}");
                }
            });
        }
    });

    // Unix socket transport (optional)
    #[cfg(unix)]
    tasks.spawn(async {
        use std::path::Path;
        let path = std::env::var("MCP_UNIX_PATH").unwrap_or_else(|_| "/tmp/mcp-payloadcms.sock".to_string());
        if Path::new(&path).exists() {
            let _ = std::fs::remove_file(&path);
        }
        let listener = UnixListener::bind(&path)
            .map_err(|e| ServiceError::FromString(format!("Unix socket bind error: {e}")))?;
        loop {
            let (stream, _) = listener
                .accept()
                .await
                .map_err(|e| ServiceError::FromString(format!("Unix socket accept error: {e}")))?;
            tokio::spawn(async move {
                let handler = match MyServerHandler::try_new() {
                    Ok(h) => h,
                    Err(e) => {
                        tracing::warn!("Failed to init handler for Unix socket: {e}");
                        return;
                    }
                };
                let router = Router::new(handler);
                let (read, write) = stream.into_split();
                if let Err(err) = router
                    .serve((read, write))
                    .await
                    .map_err(|e| ServiceError::FromString(format!("Unix socket server error: {e}")))
                {
                    tracing::warn!("Unix socket connection error: {err}");
                }
            });
        }
    });

    // Streamable HTTP + SSE transport
    tasks.spawn(async {
        let addr: SocketAddr = std::env::var("MCP_HTTP_ADDR")
            .unwrap_or_else(|_| "0.0.0.0:3001".to_string())
            .parse()
            .map_err(|e| ServiceError::FromString(format!("Invalid MCP_HTTP_ADDR: {e}")))?;

        let service = StreamableHttpService::new(
            || {
                MyServerHandler::try_new()
                    .map(Router::new)
                    .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, format!("{e}")))
            },
            Arc::new(LocalSessionManager::default()),
            StreamableHttpServerConfig::default(),
        );

        let listener = TcpListener::bind(addr)
            .await
            .map_err(|e| ServiceError::FromString(format!("HTTP/SSE listen error: {e}")))?;

        loop {
            let (stream, _) = listener
                .accept()
                .await
                .map_err(|e| ServiceError::FromString(format!("HTTP/SSE accept error: {e}")))?;
            let svc = service.clone();
            tokio::spawn(async move {
                let io = TokioIo::new(stream);
                let hyper_svc = TowerToHyperService::new(svc);
                if let Err(err) = HyperBuilder::new(TokioExecutor::new())
                    .serve_connection(io, hyper_svc)
                    .await
                {
                    tracing::warn!("HTTP/SSE connection error: {err}");
                }
            });
        }
    });

    // Dedicated SSE transport
    tasks.spawn(async {
        let addr: SocketAddr = std::env::var("MCP_SSE_ADDR")
            .unwrap_or_else(|_| "0.0.0.0:3002".to_string())
            .parse()
            .map_err(|e| ServiceError::FromString(format!("Invalid MCP_SSE_ADDR: {e}")))?;

        let config = SseServerConfig {
            bind: addr,
            sse_path: "/sse".to_string(),
            post_path: "/message".to_string(),
            ct: CancellationToken::new(),
            sse_keep_alive: Some(std::time::Duration::from_secs(15)),
        };

        let sse_server = SseServer::serve_with_config(config)
            .await
            .map_err(|e| ServiceError::FromString(format!("SSE server setup error: {e}")))?;

        sse_server.with_service_directly(|| {
            let handler = MyServerHandler::try_new()
                .expect("Failed to init handler for SSE service");
            Router::new(handler)
        });

        Ok(())
    });

    while let Some(res) = tasks.join_next().await {
        res.map_err(|e| ServiceError::FromString(format!("Task join error: {e}")))??;
    }

    Ok(())
}
