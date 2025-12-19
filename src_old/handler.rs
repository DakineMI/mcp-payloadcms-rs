// <-- handler.rs: single clean ServerHandler implementation
use crate::tools::NotificationTools;
use crate::{error::ServiceResult, notification_service::NotificationService};
use rmcp::ErrorData as CallToolError;
use rmcp::handler::server::ServerHandler;
use rmcp::model::{
    CallToolRequestParam as CallToolRequest, CallToolResult, ListToolsResult,
    PaginatedRequestParam as ListToolsRequest,
};
use rmcp::service::{RequestContext, RoleServer};
use std::future::ready;

pub struct MyServerHandler {
    notification_service: NotificationService,
}

impl MyServerHandler {
    pub fn try_new() -> ServiceResult<Self> {
        let notification_service = NotificationService::new();
        Ok(Self {
            notification_service,
        })
    }
    fn get_instructions_content() -> String {
        include_str!("../docs/tool-instructions.md").to_string()
    }

    fn create_tool_parse_error(error: impl std::fmt::Display, tool_name: &str) -> CallToolError {
        let error_msg = format!("JSON validation failed for tool '{tool_name}' - {error}");
        CallToolError::invalid_params(error_msg, None)
    }
}

impl ServerHandler for MyServerHandler {
    fn ping(
        &self,
        _ctx: RequestContext<RoleServer>,
    ) -> impl std::future::Future<Output = Result<(), rmcp::ErrorData>> + Send {
        ready(Ok(()))
    }

    async fn list_resources(
        &self,
        _req: Option<rmcp::model::PaginatedRequestParam>,
        _ctx: RequestContext<RoleServer>,
    ) -> Result<rmcp::model::ListResourcesResult, rmcp::ErrorData> {
        use rmcp::model::{Annotated, RawResource};
        Ok(rmcp::model::ListResourcesResult {
            resources: vec![Annotated {
                raw: RawResource {
                    uri: "file://instructions".to_string(),
                    name: "notify Tools Usage Instructions".to_string(),
                    title: Some("notify Tools Guide".to_string()),
                    description: Some("Guide to notify tools".to_string()),
                    mime_type: Some("text/markdown".to_string()),
                    size: None,
                    icons: None,
                },
                annotations: None,
            }],
            next_cursor: None,
        })
    }

    async fn read_resource(
        &self,
        req: rmcp::model::ReadResourceRequestParam,
        _ctx: RequestContext<RoleServer>,
    ) -> Result<rmcp::model::ReadResourceResult, rmcp::ErrorData> {
        if req.uri == "file://instructions" {
            Ok(rmcp::model::ReadResourceResult {
                contents: vec![rmcp::model::ResourceContents::text(
                    Self::get_instructions_content(),
                    "file://instructions",
                )],
            })
        } else {
            Err(rmcp::ErrorData::invalid_params(
                format!("Unknown resource URI: {}", req.uri),
                None,
            ))
        }
    }

    fn list_tools(
        &self,
        _req: Option<ListToolsRequest>,
        _ctx: RequestContext<RoleServer>,
    ) -> impl std::future::Future<Output = Result<ListToolsResult, rmcp::ErrorData>> + Send {
        ready(Ok(ListToolsResult {
            tools: NotificationTools::tools(),
            next_cursor: None,
        }))
    }

    async fn call_tool(
        &self,
        request: CallToolRequest,
        _ctx: RequestContext<RoleServer>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        let tool_name = request.name.clone();
        let tool_params: NotificationTools = match NotificationTools::try_from(request.clone()) {
            Ok(params) => params,
            Err(err) => {
                return Err(MyServerHandler::create_tool_parse_error(err, &tool_name));
            }
        };

        tool_params.run(&self.notification_service).await
    }
}
