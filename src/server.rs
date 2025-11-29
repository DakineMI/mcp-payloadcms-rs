// storage/planning removed: server is now payload-focused and local-only
use rmcp::RoleServer;
use rmcp::{
    handler::server::{router::tool::ToolRouter, wrapper::Parameters},
    model::*,
    tool, tool_handler, tool_router,
};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::json;
// server is now lightweight and doesn't require shared storage state
use crate::payload::{
    FileType as PayloadFileType, execute_sql_query, generate_template, query_validation_rules,
    scaffold_project, validate_payload_code, validate_scaffold_options,
};

// Payload tool argument types

#[derive(Clone, Debug, Deserialize, Serialize, JsonSchema)]
pub struct EchoArgs {
    pub message: String,
}

#[derive(Clone, Debug, Deserialize, Serialize, JsonSchema)]
pub struct ValidateArgs {
    pub code: String,
    #[serde(rename = "fileType")]
    pub file_type: String,
}

#[derive(Clone, Debug, Deserialize, Serialize, JsonSchema)]
pub struct QueryArgs {
    pub query: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "fileType")]
    pub file_type: Option<String>,
}

#[derive(Clone, Debug, Deserialize, Serialize, JsonSchema)]
pub struct McpQueryArgs {
    pub sql: String,
}

#[derive(Clone, Debug, Deserialize, Serialize, JsonSchema)]
pub struct GenerateTemplateArgs {
    #[serde(rename = "templateType")]
    pub template_type: String,
    #[serde(default)]
    pub options: Option<serde_json::Value>,
}

#[derive(Clone, Debug, Deserialize, Serialize, JsonSchema)]
pub struct GenericOptions {
    #[serde(flatten)]
    pub inner: serde_json::Value,
}

#[derive(Clone, Debug, Deserialize, Serialize, JsonSchema)]
pub struct ScaffoldArgs {
    #[serde(rename = "projectName")]
    pub project_name: String,
    #[serde(rename = "outputDir", skip_serializing_if = "Option::is_none")]
    pub output_dir: Option<String>,
}

#[derive(Clone)]
pub struct SoftwarePlanningServer {
    pub tool_router: ToolRouter<SoftwarePlanningServer>,
}

#[tool_router]
impl SoftwarePlanningServer {
    pub fn new() -> Self {
        Self {
            tool_router: Self::tool_router(),
        }
    }
    #[tool(description = "Echo a message")]
    async fn echo(
        &self,
        Parameters(args): Parameters<EchoArgs>,
    ) -> Result<CallToolResult, ErrorData> {
        Ok(CallToolResult::success(vec![Content::text(format!(
            "Tool echo: {}",
            args.message
        ))]))
    }

    #[tool(description = "Validate Payload CMS code")]
    async fn validate(
        &self,
        Parameters(args): Parameters<ValidateArgs>,
    ) -> Result<CallToolResult, ErrorData> {
        let ft = PayloadFileType::from(args.file_type.as_str());
        let result = validate_payload_code(&args.code, ft);
        let content = Content::text(serde_json::to_string_pretty(&result).unwrap());
        Ok(CallToolResult::success(vec![content]))
    }

    #[tool(description = "Query validation rules")]
    async fn query(
        &self,
        Parameters(args): Parameters<QueryArgs>,
    ) -> Result<CallToolResult, ErrorData> {
        let ft = args.file_type.as_deref().map(PayloadFileType::from);
        let rules = query_validation_rules(&args.query, ft);
        let content =
            Content::text(serde_json::to_string_pretty(&json!({ "rules": rules })).unwrap());
        Ok(CallToolResult::success(vec![content]))
    }

    #[tool(description = "Execute SQL-like query")]
    async fn mcp_query(
        &self,
        Parameters(args): Parameters<McpQueryArgs>,
    ) -> Result<CallToolResult, ErrorData> {
        match execute_sql_query(&args.sql) {
            Ok(results) => {
                let content = Content::text(
                    serde_json::to_string_pretty(&json!({ "results": results })).unwrap(),
                );
                Ok(CallToolResult::success(vec![content]))
            }
            Err(e) => Ok(CallToolResult::success(vec![Content::text(
                serde_json::to_string_pretty(&json!({ "error": e })).unwrap(),
            )])),
        }
    }

    #[tool(description = "Generate code template")]
    async fn generate_template_tool(
        &self,
        Parameters(args): Parameters<GenerateTemplateArgs>,
    ) -> Result<CallToolResult, ErrorData> {
        let opts = args.options.as_ref().unwrap_or(&serde_json::Value::Null);
        match generate_template(&args.template_type, opts) {
            Ok(code) => Ok(CallToolResult::success(vec![Content::text(code)])),
            Err(e) => Ok(CallToolResult::success(vec![Content::text(
                serde_json::to_string_pretty(&json!({ "error": e })).unwrap(),
            )])),
        }
    }

    #[tool(description = "Generate a complete collection template")]
    async fn generate_collection(
        &self,
        Parameters(options): Parameters<GenericOptions>,
    ) -> Result<CallToolResult, ErrorData> {
        let code = generate_template("collection", &options.inner)
            .unwrap_or_else(|e| format!("/* error: {} */", e));
        Ok(CallToolResult::success(vec![Content::text(code)]))
    }

    #[tool(description = "Generate a field template")]
    async fn generate_field(
        &self,
        Parameters(options): Parameters<GenericOptions>,
    ) -> Result<CallToolResult, ErrorData> {
        let code = generate_template("field", &options.inner)
            .unwrap_or_else(|e| format!("/* error: {} */", e));
        Ok(CallToolResult::success(vec![Content::text(code)]))
    }

    #[tool(description = "Scaffold a complete project")]
    async fn scaffold_project_tool(
        &self,
        Parameters(args): Parameters<ScaffoldArgs>,
    ) -> Result<CallToolResult, ErrorData> {
        let mut options = serde_json::Map::new();
        options.insert(
            "projectName".to_string(),
            serde_json::Value::String(args.project_name.clone()),
        );
        if let Some(dir) = args.output_dir {
            options.insert("outputDir".to_string(), serde_json::Value::String(dir));
        }
        let opts_value = serde_json::Value::Object(options);
        let (valid, errs) = validate_scaffold_options(&opts_value);
        if !valid {
            return Ok(CallToolResult::success(vec![Content::text(
                serde_json::to_string_pretty(
                    &json!({ "error": "Invalid scaffold options", "details": errs }),
                )
                .unwrap(),
            )]));
        }
        match scaffold_project(&opts_value) {
            Ok(j) => Ok(CallToolResult::success(vec![Content::json(j)?])),
            Err(e) => Ok(CallToolResult::success(vec![Content::text(
                serde_json::to_string_pretty(&json!({ "error": e })).unwrap(),
            )])),
        }
    }
}

#[tool_handler]
impl rmcp::ServerHandler for SoftwarePlanningServer {
    fn get_info(&self) -> ServerInfo {
        ServerInfo {
            protocol_version: ProtocolVersion::V_2024_11_05,
            capabilities: ServerCapabilities::builder()
                .enable_tools()
                .enable_resources()
                .build(),
            server_info: Implementation::from_build_env(),
            instructions: Some("Payload MCP server - use tools to interact".to_string()),
            // No struct update needed; all fields set explicitly.
        }
    }

    async fn list_resources(
        &self,
        _request: Option<PaginatedRequestParam>,
        _ctx: rmcp::service::RequestContext<RoleServer>,
    ) -> Result<ListResourcesResult, ErrorData> {
        Ok(ListResourcesResult {
            resources: vec![
                RawResource::new("payload://templates", "Payload Templates").no_annotation(),
                RawResource::new("payload://scaffold", "Scaffolded Projects").no_annotation(),
            ],
            next_cursor: None,
        })
    }

    async fn read_resource(
        &self,
        ReadResourceRequestParam { uri }: ReadResourceRequestParam,
        _ctx: rmcp::service::RequestContext<RoleServer>,
    ) -> Result<ReadResourceResult, ErrorData> {
        match uri.as_str() {
            "payload://templates" => Ok(ReadResourceResult { contents: vec![ResourceContents::text(serde_json::to_string(&json!({"info":"templates","description":"Generate templates with the payload tools"})).unwrap(), uri)], }),
            "payload://scaffold" => Ok(ReadResourceResult { contents: vec![ResourceContents::text(serde_json::to_string(&json!({"info":"scaffold","description":"Scaffold projects via the scaffold_project tool"})).unwrap(), uri)], }),
            _ => Err(ErrorData::resource_not_found(
                "Unknown resource URI",
                Some(json!({ "uri": uri })),
            )),
        }
    }

    async fn list_resource_templates(
        &self,
        _request: Option<PaginatedRequestParam>,
        _ctx: rmcp::service::RequestContext<RoleServer>,
    ) -> Result<ListResourceTemplatesResult, ErrorData> {
        Ok(ListResourceTemplatesResult {
            next_cursor: None,
            resource_templates: Vec::new(),
        })
    }
}
