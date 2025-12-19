use std::net::SocketAddr;

use clap::{Args, Parser, Subcommand};

use crate::metadata::{PKG_DESCRIPTION, PKG_NAME, PKG_VERSION};

#[derive(Parser, Debug, Clone)]
#[command(name = PKG_NAME)]
#[command(version = PKG_VERSION)]
#[command(about = PKG_DESCRIPTION, long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Command,
}

#[derive(Subcommand, Debug, Clone)]
pub enum Command {
    /// Start the MCP server
    Start(CommandArguments),
    /// Show the resolved transport configuration
    Status,
    /// Gracefully stop a running server (using pid file)
    Shutdown,
    /// Print version information
    Version,
    /// Print setup guidance for a client (prefers HTTP endpoints)
    Setup,
    /// Open an interactive config editor for settings.json
    Config,
}

#[derive(Args, Debug, Clone)]
pub struct CommandArguments {
    /// Server name (used in initialize)
    #[arg(skip = crate::metadata::PKG_NAME.to_string())]
    pub server_name: String,

    /// Server description/title (used in initialize)
    #[arg(skip = crate::metadata::PKG_DESCRIPTION.to_string())]
    pub server_description: String,

    /// Enable stdio transport
    #[arg(long, env = "MCP_ENABLE_STDIO", default_value_t = true)]
    pub enable_stdio: bool,

    /// Enable TCP transport
    #[arg(long, env = "MCP_ENABLE_TCP", default_value_t = false)]
    pub enable_tcp: bool,

    /// Enable Unix socket transport (unix only)
    #[arg(long, env = "MCP_ENABLE_UNIX", default_value_t = false)]
    pub enable_unix: bool,

    /// Enable streamable HTTP+SSE transport
    #[arg(long, env = "MCP_ENABLE_HTTP", default_value_t = true)]
    pub enable_http: bool,

    /// Enable dedicated SSE transport
    #[arg(long, env = "MCP_ENABLE_SSE", default_value_t = true)]
    pub enable_sse: bool,

    /// TCP bind address (for std TCP transport)
    #[arg(long, env = "MCP_TCP_ADDR", default_value = "127.0.0.1:0")]
    pub tcp_addr: String,

    /// HTTP/SSE bind address (streamable HTTP)
    #[arg(long, env = "MCP_HTTP_ADDR", default_value = "0.0.0.0:0")]
    pub http_addr: String,

    /// Dedicated SSE bind address
    #[arg(long, env = "MCP_SSE_ADDR", default_value = "0.0.0.0:0")]
    pub sse_addr: String,

    /// Websocket bind address
    #[arg(long, env = "MCP_WS_ADDR", default_value = "0.0.0.0:0")]
    pub ws_addr: String,

    /// Enable websocket transport
    #[arg(long, env = "MCP_ENABLE_WS", default_value_t = false)]
    pub enable_ws: bool,

    /// Unix socket path (unix only)
    #[arg(long, env = "MCP_UNIX_PATH", default_value = "/tmp/mcp-server.sock")]
    pub unix_path: String,

    /// PID file path for shutdown coordination
    #[arg(long, env = "MCP_PID_FILE", default_value = "/tmp/mcp-server-template-rs.pid")]
    pub pid_file: String,

    /// Runtime info file (used by status/shutdown commands)
    #[arg(long, env = "MCP_RUNTIME_INFO_FILE", default_value = "/tmp/mcp-server-template-rs.runtime.json")]
    pub runtime_info_file: String,

    /// Run in foreground (skip background/daemon spawn)
    #[arg(long, env = "MCP_FOREGROUND", default_value_t = false, hide = true)]
    pub foreground: bool,
}

impl CommandArguments {
    pub fn default_settings() -> Self {
        Self {
            server_name: crate::metadata::PKG_NAME.to_string(),
            server_description: crate::metadata::PKG_DESCRIPTION.to_string(),
            enable_stdio: true,
            enable_tcp: false,
            enable_unix: false,
            enable_http: true,
            enable_sse: true,
            enable_ws: false,
            tcp_addr: "127.0.0.1:0".to_string(),
            http_addr: "0.0.0.0:0".to_string(),
            sse_addr: "0.0.0.0:0".to_string(),
            ws_addr: "0.0.0.0:0".to_string(),
            unix_path: "/tmp/mcp-server.sock".to_string(),
            pid_file: "/tmp/mcp-server-template-rs.pid".to_string(),
            runtime_info_file: "/tmp/mcp-server-template-rs.runtime.json".to_string(),
            foreground: false,
        }
    }

    /// Validate CLI/environment-derived arguments.
    pub fn validate(&self) -> Result<(), String> {
        if !self.enable_stdio
            && !self.enable_tcp
            && !self.enable_unix
            && !self.enable_http
            && !self.enable_sse
            && !self.enable_ws
        {
            return Err(
                "Enable at least one transport (stdio, tcp, unix, http, or sse)".to_string(),
            );
        }

        if self.enable_tcp {
            self.tcp_addr
                .parse::<SocketAddr>()
                .map_err(|e| format!("Invalid MCP_TCP_ADDR '{}': {e}", self.tcp_addr))?;
        }
        if self.enable_http {
            self.http_addr
                .parse::<SocketAddr>()
                .map_err(|e| format!("Invalid MCP_HTTP_ADDR '{}': {e}", self.http_addr))?;
        }
        if self.enable_sse {
            self.sse_addr
                .parse::<SocketAddr>()
                .map_err(|e| format!("Invalid MCP_SSE_ADDR '{}': {e}", self.sse_addr))?;
        }
        if self.enable_ws {
            self.ws_addr
                .parse::<SocketAddr>()
                .map_err(|e| format!("Invalid MCP_WS_ADDR '{}': {e}", self.ws_addr))?;
        }
        if self.enable_unix && self.unix_path.trim().is_empty() {
            return Err("MCP_UNIX_PATH cannot be empty when unix transport is enabled".to_string());
        }
        Ok(())
    }
}