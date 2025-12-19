use std::{
    fs,
    net::SocketAddr,
    sync::Arc,
    time::{Duration, SystemTime},
};

use hyper_util::{
    rt::{TokioExecutor, TokioIo},
    server::conn::auto::Builder as HyperBuilder,
    service::TowerToHyperService,
};
use pin_project_lite::pin_project;
use rmcp::{
    ServiceExt,
    handler::server::router::Router,
    model::{Implementation, InitializeResult, ProtocolVersion, ServerCapabilities},
    transport::{
        sse_server::{SseServer, SseServerConfig},
        streamable_http_server::{
            StreamableHttpServerConfig, StreamableHttpService, session::local::LocalSessionManager,
        },
    },
};
use serde::{Deserialize, Serialize};
#[cfg(unix)]
use tokio::net::UnixListener;
use tokio::{net::TcpListener, task::JoinSet};
use tokio_tungstenite::tungstenite;
use tokio_util::sync::CancellationToken;

use crate::{cli::CommandArguments, error::{ServiceError, ServiceResult}, handler::ToolBoxHandler};

#[derive(Clone)]
pub struct TransportState {
    pub stdio: bool,
    pub tcp: Option<SocketAddr>,
    pub unix_path: Option<String>,
    pub http: Option<SocketAddr>,
    pub sse: Option<SocketAddr>,
    pub ws: Option<SocketAddr>,
}

impl TransportState {
    pub fn from_args(args: &CommandArguments) -> Result<Self, crate::error::ServiceError> {
        let tcp = if args.enable_tcp {
            Some(args.tcp_addr.parse().map_err(|e| {
                crate::error::ServiceError::FromString(format!("Invalid MCP_TCP_ADDR: {e}"))
            })?)
        } else {
            None
        };
        let http = if args.enable_http {
            Some(args.http_addr.parse().map_err(|e| {
                crate::error::ServiceError::FromString(format!("Invalid MCP_HTTP_ADDR: {e}"))
            })?)
        } else {
            None
        };
        let sse = if args.enable_sse {
            Some(args.sse_addr.parse().map_err(|e| {
                crate::error::ServiceError::FromString(format!("Invalid MCP_SSE_ADDR: {e}"))
            })?)
        } else {
            None
        };
        let ws = if args.enable_ws {
            Some(args.ws_addr.parse().map_err(|e| {
                crate::error::ServiceError::FromString(format!("Invalid MCP_WS_ADDR: {e}"))
            })?)
        } else {
            None
        };

        Ok(Self {
            stdio: args.enable_stdio,
            tcp,
            unix_path: args.enable_unix.then_some(args.unix_path.clone()),
            http,
            sse,
            ws,
        })
    }

    pub fn any_enabled(&self) -> bool {
        self.stdio
            || self.tcp.is_some()
            || self.unix_path.is_some()
            || self.http.is_some()
            || self.sse.is_some()
            || self.ws.is_some()
    }

    pub fn active_endpoints(&self) -> Vec<String> {
        let mut endpoints = Vec::new();
        if self.stdio {
            endpoints.push("stdio".to_string());
        }
        if let Some(addr) = &self.tcp {
            endpoints.push(format!("tcp@{addr}"));
        }
        if let Some(path) = &self.unix_path {
            endpoints.push(format!("unix@{path}"));
        }
        if let Some(addr) = &self.http {
            endpoints.push(format!("http+streamable-sse@{addr}"));
        }
        if let Some(addr) = &self.sse {
            endpoints.push(format!("sse@{addr}"));
        }
        if let Some(addr) = &self.ws {
            endpoints.push(format!("ws@{addr}"));
        }
        endpoints
    }
}

#[derive(Clone)]
pub struct ServerState {
    pub started_at: SystemTime,
    pub version: String,
    pub transports: TransportState,
    pub name: String,
    pub description: String,
}

impl ServerState {
    pub fn new(transports: TransportState, name: String, description: String) -> Self {
        Self {
            started_at: SystemTime::now(),
            version: env!("CARGO_PKG_VERSION").to_string(),
            transports,
            name,
            description,
        }
    }

    pub fn uptime(&self) -> Duration {
        SystemTime::now()
            .duration_since(self.started_at)
            .unwrap_or_else(|_| Duration::from_secs(0))
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RuntimeInfo {
    pub pid: u32,
    pub endpoints: Vec<String>,
}

fn write_runtime_info(path: &str, info: &RuntimeInfo) -> ServiceResult<()> {
    let json = serde_json::to_string_pretty(info)
        .map_err(|e| ServiceError::FromString(format!("Serialize runtime info error: {e}")))?;
    fs::write(path, json)
        .map_err(|e| ServiceError::FromString(format!("Write runtime info error: {e}")))?;
    Ok(())
}

pub fn server_details(state: &ServerState) -> InitializeResult {
    InitializeResult {
        server_info: Implementation {
            name: state.name.clone(),
            version: state.version.clone(),
            title: Some(state.description.clone()),
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
        instructions: ToolBoxHandler::instructions(),
        protocol_version: ProtocolVersion::default(),
    }
}

fn make_service(state: Arc<ServerState>) -> std::io::Result<Router<ToolBoxHandler>> {
    Ok(Router::new(ToolBoxHandler::new(state)))
}

pub async fn start_server(args: CommandArguments) -> ServiceResult<()> {
    let mut tasks: JoinSet<ServiceResult<()>> = JoinSet::new();
    let mut endpoints: Vec<String> = Vec::new();

    let transports = TransportState::from_args(&args)?;
    if !transports.any_enabled() {
        return Err(crate::error::ServiceError::FromString(
            "No transports enabled; toggle MCP_ENABLE_* env vars or CLI flags".to_string(),
        ));
    }
    let state = Arc::new(ServerState::new(
        transports,
        args.server_name.clone(),
        args.server_description.clone(),
    ));
    tracing::info!(
        "Starting MCP server v{} on {}",
        state.version,
        state.transports.active_endpoints().join(", ")
    );
    endpoints.extend(state.transports.active_endpoints());

    // stdio
    if state.transports.stdio {
        let state = state.clone();
        endpoints.push("stdio".to_string());
        tasks.spawn(async move {
            tracing::info!("stdio transport listening on stdin/stdout");
            let router = make_service(state)
                .map_err(|e| crate::error::ServiceError::FromString(e.to_string()))?;
            router
                .serve((tokio::io::stdin(), tokio::io::stdout()))
                .await
                .map_err(|e| crate::error::ServiceError::FromString(format!("Stdio error: {e}")))?;
            Ok(())
        });
    }

    // TCP
    if let Some(tcp_addr) = state.transports.tcp {
        let state = state.clone();
        tasks.spawn(async move {
            tracing::info!("TCP transport binding to {tcp_addr}");
            let listener = TcpListener::bind(tcp_addr).await.map_err(|e| {
                crate::error::ServiceError::FromString(format!("TCP bind error: {e}"))
            })?;
            if let Ok(actual) = listener.local_addr() {
                endpoints_lock_push(&state, format!("tcp@{actual}"));
            }
            loop {
                let (stream, _) = listener.accept().await.map_err(|e| {
                    crate::error::ServiceError::FromString(format!("TCP accept error: {e}"))
                })?;
                let state_for_conn = state.clone();
                tokio::spawn(async move {
                    match make_service(state_for_conn) {
                        Ok(router) => {
                            if let Err(err) = router.serve(stream).await.map_err(|e| {
                                crate::error::ServiceError::FromString(format!(
                                    "TCP server error: {e}"
                                ))
                            }) {
                                tracing::warn!("TCP connection error: {err}");
                            }
                        }
                        Err(e) => tracing::warn!("Failed to init service for TCP: {e}"),
                    }
                });
            }
        });
    }

    // Unix socket
    #[cfg(unix)]
    if let Some(unix_path) = state.transports.unix_path.clone() {
        let state = state.clone();
        tasks.spawn(async move {
            use std::path::Path;
            let path = unix_path;
            if Path::new(&path).exists() {
                let _ = std::fs::remove_file(&path);
            }
            tracing::info!("Unix transport binding to {path}");
            let listener = UnixListener::bind(&path).map_err(|e| {
                crate::error::ServiceError::FromString(format!("Unix bind error: {e}"))
            })?;
            loop {
                let (stream, _) = listener.accept().await.map_err(|e| {
                    crate::error::ServiceError::FromString(format!("Unix accept error: {e}"))
                })?;
                let state_for_conn = state.clone();
                tokio::spawn(async move {
                    match make_service(state_for_conn) {
                        Ok(router) => {
                            if let Err(err) = router.serve(stream).await.map_err(|e| {
                                crate::error::ServiceError::FromString(format!(
                                    "Unix server error: {e}"
                                ))
                            }) {
                                tracing::warn!("Unix connection error: {err}");
                            }
                        }
                        Err(e) => tracing::warn!("Failed to init service for Unix: {e}"),
                    }
                });
            }
        });
    }

    // Streamable HTTP+SSE
    if let Some(http_addr) = state.transports.http {
        let state_for_service = state.clone();
        tasks.spawn(async move {
            let state_for_factory = state_for_service.clone();
            let service = StreamableHttpService::new(
                move || {
                    make_service(state_for_factory.clone())
                        .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, format!("{e}")))
                },
                Arc::new(LocalSessionManager::default()),
                StreamableHttpServerConfig::default(),
            );

            let listener = TcpListener::bind(http_addr).await.map_err(|e| {
                crate::error::ServiceError::FromString(format!("Streamable HTTP bind error: {e}"))
            })?;
            let actual_addr = listener.local_addr().ok();
            if let Some(addr) = actual_addr {
                endpoints_lock_push(&state_for_service, format!("streamable-http@{addr}"));
            }
            tracing::info!(
                "Streamable HTTP listening on {}",
                actual_addr
                    .map(|a| a.to_string())
                    .unwrap_or_else(|| http_addr.to_string())
            );

            loop {
                let (stream, _) = listener.accept().await.map_err(|e| {
                    crate::error::ServiceError::FromString(format!(
                        "Streamable HTTP accept error: {e}"
                    ))
                })?;
                let svc = service.clone();
                tokio::spawn(async move {
                    let io = TokioIo::new(stream);
                    let hyper_svc = TowerToHyperService::new(svc);
                    if let Err(err) = HyperBuilder::new(TokioExecutor::new())
                        .serve_connection(io, hyper_svc)
                        .await
                    {
                        tracing::warn!("Streamable HTTP connection error: {err}");
                    }
                });
            }
        });
    }

    // Dedicated SSE
    if let Some(sse_addr) = state.transports.sse {
        let state = state.clone();
        tasks.spawn(async move {
            let config = SseServerConfig {
                bind: sse_addr,
                sse_path: "/sse".to_string(),
                post_path: "/message".to_string(),
                ct: CancellationToken::new(),
                sse_keep_alive: Some(std::time::Duration::from_secs(15)),
            };

            let sse_server = SseServer::serve_with_config(config).await.map_err(|e| {
                crate::error::ServiceError::FromString(format!("SSE server setup error: {e}"))
            })?;
            tracing::info!("SSE transport binding to {sse_addr}");

            sse_server.with_service_directly(move || {
                make_service(state.clone()).expect("Failed to init service for SSE")
            });

            Ok(())
        });
    }

    // Websocket
    if let Some(ws_addr) = state.transports.ws {
        let state = state.clone();
        tasks.spawn(async move {
            let listener = TcpListener::bind(ws_addr).await.map_err(|e| {
                crate::error::ServiceError::FromString(format!("Websocket bind error: {e}"))
            })?;
            if let Ok(actual) = listener.local_addr() {
                tracing::info!("Websocket transport listening on {actual}");
                endpoints_lock_push(&state, format!("ws@{actual}"));
            } else {
                tracing::info!("Websocket transport binding to {ws_addr}");
            }
            loop {
                let (stream, peer) = listener.accept().await.map_err(|e| {
                    crate::error::ServiceError::FromString(format!("Websocket accept error: {e}"))
                })?;
                let state_for_conn = state.clone();
                tokio::spawn(async move {
                    match tokio_tungstenite::accept_async(stream).await {
                        Ok(ws_stream) => {
                            let transport = WebsocketTransport::new(ws_stream);
                            match make_service(state_for_conn) {
                                Ok(router) => {
                                    if let Err(err) = router.serve(transport).await.map_err(|e| {
                                        crate::error::ServiceError::FromString(format!(
                                            "Websocket server error: {e}"
                                        ))
                                    }) {
                                        tracing::warn!(
                                            "Websocket connection error (peer {peer}): {err}"
                                        );
                                    }
                                }
                                Err(e) => {
                                    tracing::warn!("Failed to init service for Websocket: {e}")
                                }
                            }
                        }
                        Err(err) => tracing::warn!("Websocket handshake error from {peer}: {err}"),
                    }
                });
            }
        });
    }

    while let Some(res) = tasks.join_next().await {
        res.map_err(|e| crate::error::ServiceError::FromString(format!("Task join error: {e}")))??;
    }

    // Write runtime info (best-effort)
    let info = RuntimeInfo {
        pid: std::process::id(),
        endpoints,
    };
    let _ = write_runtime_info(&args.runtime_info_file, &info);

    Ok(())
}

fn endpoints_lock_push(state: &ServerState, _ep: String) {
    let _ = state; // endpoints are collected in outer scope; this is a placeholder to keep signature symmetrical
}

pin_project! {
    struct WebsocketTransport<S> {
        #[pin]
        stream: S,
    }
}

impl<S> WebsocketTransport<S> {
    fn new(stream: S) -> Self {
        Self { stream }
    }
}

impl<S> futures::Stream for WebsocketTransport<S>
where
    S: futures::Stream<Item = Result<tungstenite::Message, tungstenite::Error>> + Unpin,
{
    type Item = rmcp::service::RxJsonRpcMessage<rmcp::RoleServer>;

    fn poll_next(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Option<Self::Item>> {
        let this = self.as_mut().project();
        match this.stream.poll_next(cx) {
            std::task::Poll::Ready(Some(Ok(message))) => {
                let message = match message {
                    tungstenite::Message::Text(json) => json,
                    _ => return self.poll_next(cx),
                };
                let parsed = match serde_json::from_str::<
                    rmcp::service::RxJsonRpcMessage<rmcp::RoleServer>,
                >(&message)
                {
                    Ok(msg) => msg,
                    Err(err) => {
                        tracing::warn!("Websocket JSON parse error: {err}");
                        return self.poll_next(cx);
                    }
                };
                std::task::Poll::Ready(Some(parsed))
            }
            std::task::Poll::Ready(Some(Err(err))) => {
                tracing::warn!("Websocket read error: {err}");
                self.poll_next(cx)
            }
            std::task::Poll::Ready(None) => std::task::Poll::Ready(None),
            std::task::Poll::Pending => std::task::Poll::Pending,
        }
    }
}

impl<S> futures::Sink<rmcp::service::TxJsonRpcMessage<rmcp::RoleServer>> for WebsocketTransport<S>
where
    S: futures::Sink<tungstenite::Message, Error = tungstenite::Error> + Unpin,
{
    type Error = rmcp::ErrorData;

    fn poll_ready(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<(), Self::Error>> {
        self.as_mut()
            .project()
            .stream
            .poll_ready(cx)
            .map_err(|err: tungstenite::Error| {
                rmcp::ErrorData::internal_error(err.to_string(), None)
            })
    }

    fn start_send(
        mut self: std::pin::Pin<&mut Self>,
        item: rmcp::service::TxJsonRpcMessage<rmcp::RoleServer>,
    ) -> Result<(), Self::Error> {
        let msg = serde_json::to_string(&item).map_err(|err: serde_json::Error| {
            rmcp::ErrorData::internal_error(err.to_string(), None)
        })?;
        self.as_mut()
            .project()
            .stream
            .start_send(tungstenite::Message::Text(msg.into()))
            .map_err(|err: tungstenite::Error| {
                rmcp::ErrorData::internal_error(err.to_string(), None)
            })
    }

    fn poll_flush(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<(), Self::Error>> {
        self.as_mut()
            .project()
            .stream
            .poll_flush(cx)
            .map_err(|err: tungstenite::Error| {
                rmcp::ErrorData::internal_error(err.to_string(), None)
            })
    }

    fn poll_close(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<(), Self::Error>> {
        self.as_mut()
            .project()
            .stream
            .poll_close(cx)
            .map_err(|err| rmcp::ErrorData::internal_error(err.to_string(), None))
    }
}
