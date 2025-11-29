use rmcp::ServiceExt;
use rmcp::handler::server::router::Router;
use rmcp::model::{Implementation, InitializeResult, ProtocolVersion, ServerCapabilities};

use crate::{
    cli::CommandArguments,
    error::{ServiceError, ServiceResult},
    handler::MyServerHandler,
};

pub fn server_details() -> InitializeResult {
    InitializeResult {
        server_info: Implementation {
            name: "mcp-payloadcms-rs".to_string(),
            version: env!("CARGO_PKG_VERSION").to_string(),
            title: Some("Payload CMS MCP Server: Manage Payload CMS via MCP".to_string()),
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
    let handler = MyServerHandler::try_new()?;
    let router = Router::new(handler);
    router
        .serve((tokio::io::stdin(), tokio::io::stdout()))
        .await
        .map_err(|e| ServiceError::FromString(format!("Server error: {e}")))?;
    Ok(())
}
