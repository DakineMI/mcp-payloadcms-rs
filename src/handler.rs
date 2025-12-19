use std::{future::ready, sync::Arc};

use rmcp::{
    handler::server::{ServerHandler, tool::ToolRouter, wrapper::Parameters},
    model::{PaginatedRequestParam as ListResourcesRequest, CallToolResult},
    service::{RequestContext, RoleServer},
    tool, tool_handler, tool_router,
    ErrorData,
};
use serde_json::{json, Value};

use crate::{
    server::ServerState,
    payload_tools::{
        mcp::{
            EchoParams, ValidateParams, QueryParams, SqlParams,
            GenerateTemplateParams, GenerateCollectionParams, GenerateFieldParams,
            ConnectPayloadParams, GetCollectionParams, ListCollectionsParams, ValidateAgainstLiveParams,
        },
        client::create_payload_client,
        scaffolder::{
            scaffold_project, validate_scaffold_options, ScaffoldFile, ScaffoldFileStructure,
            ScaffoldOptions,
        },
        validator::validate_payload_code,
        query::{get_validation_rules_with_examples, query_validation_rules},
        sql::execute_sql_query,
        generator::{generate_template, TemplateType},
    },
};

pub struct ToolBoxHandler {
    tool_router: ToolRouter<Self>,
}

impl ToolBoxHandler {
    pub fn new(_state: Arc<ServerState>) -> Self {
        Self {
            tool_router: Self::tool_router(),
        }
    }

    pub fn instructions() -> Option<String> {
        Some(include_str!("../docs/instructions.md").to_string())
    }
}

fn scaffold_to_json(map: ScaffoldFileStructure) -> Value {
    let mut out = serde_json::Map::new();
    for (k, v) in map {
        match v {
            ScaffoldFile::File(content) => {
                out.insert(k, json!(content));
            }
            ScaffoldFile::Directory(dir) => {
                out.insert(k, scaffold_to_json(dir));
            }
        }
    }
    Value::Object(out)
}

#[tool_router]
impl ToolBoxHandler {
    #[tool(name = "echo", description = "Echo a message back to the caller")]
    fn echo(&self, Parameters(params): Parameters<EchoParams>) -> String {
        format!("Tool echo: {}", params.message)
    }

    #[tool(name = "validate", description = "Validate Payload CMS code")]
    fn validate(&self, Parameters(params): Parameters<ValidateParams>) -> Result<CallToolResult, ErrorData> {
        let result = validate_payload_code(&params.code, params.file_type);
        Ok(CallToolResult::structured(json!(result)))
    }

    #[tool(name = "query", description = "Query validation rules")]
    fn query(&self, Parameters(params): Parameters<QueryParams>) -> Result<CallToolResult, ErrorData> {
        let rules = if params.query.trim().is_empty() {
            get_validation_rules_with_examples(None, params.file_type)
        } else {
            query_validation_rules(&params.query, params.file_type)
        };
        Ok(CallToolResult::structured(json!({ "rules": rules })))
    }

    #[tool(name = "mcp_query", description = "Execute SQL-like queries")]
    fn mcp_query(&self, Parameters(params): Parameters<SqlParams>) -> Result<CallToolResult, ErrorData> {
        match execute_sql_query(&params.sql) {
            Ok(results) => Ok(CallToolResult::structured(json!({ "results": results }))),
            Err(err) => Err(ErrorData::internal_error(err, None)),
        }
    }

    #[tool(name = "generate_template", description = "Generate Payload CMS code templates")]
    fn generate_template(&self, Parameters(params): Parameters<GenerateTemplateParams>) -> Result<CallToolResult, ErrorData> {
        match generate_template(params.template_type, &params.options) {
            Ok(code) => Ok(CallToolResult::structured(json!({ "code": code }))),
            Err(err) => Err(ErrorData::internal_error(err, None)),
        }
    }

    #[tool(name = "generate_collection", description = "Generate a Payload CMS collection template")]
    fn generate_collection(&self, Parameters(params): Parameters<GenerateCollectionParams>) -> Result<CallToolResult, ErrorData> {
        let mut options = serde_json::Map::new();
        options.insert("slug".into(), json!(params.slug));
        if let Some(fields) = params.fields { options.insert("fields".into(), fields); }
        if let Some(auth) = params.auth { options.insert("auth".into(), json!(auth)); }
        if let Some(ts) = params.timestamps { options.insert("timestamps".into(), json!(ts)); }
        if let Some(admin) = params.admin { options.insert("admin".into(), admin); }
        if let Some(hooks) = params.hooks { options.insert("hooks".into(), json!(hooks)); }
        if let Some(access) = params.access { options.insert("access".into(), json!(access)); }
        if let Some(versions) = params.versions { options.insert("versions".into(), json!(versions)); }

        match generate_template(TemplateType::Collection, &Value::Object(options)) {
            Ok(code) => Ok(CallToolResult::structured(json!({ "code": code }))),
            Err(err) => Err(ErrorData::internal_error(err, None)),
        }
    }

    #[tool(name = "generate_field", description = "Generate a Payload CMS field template")]
    fn generate_field(&self, Parameters(params): Parameters<GenerateFieldParams>) -> Result<CallToolResult, ErrorData> {
        let mut options = serde_json::Map::new();
        options.insert("name".into(), json!(params.name));
        options.insert("type".into(), json!(params.field_type));
        if let Some(required) = params.required { options.insert("required".into(), json!(required)); }
        if let Some(unique) = params.unique { options.insert("unique".into(), json!(unique)); }
        if let Some(localized) = params.localized { options.insert("localized".into(), json!(localized)); }
        if let Some(access) = params.access { options.insert("access".into(), json!(access)); }
        if let Some(admin) = params.admin { options.insert("admin".into(), admin); }
        if let Some(validation) = params.validation { options.insert("validation".into(), json!(validation)); }
        if let Some(default_value) = params.default_value { options.insert("defaultValue".into(), default_value); }

        match generate_template(TemplateType::Field, &Value::Object(options)) {
            Ok(code) => Ok(CallToolResult::structured(json!({ "code": code }))),
            Err(err) => Err(ErrorData::internal_error(err, None)),
        }
    }

    #[tool(name = "scaffold_project", description = "Scaffold a complete Payload CMS 3 project structure")]
    fn scaffold_project(&self, Parameters(params): Parameters<ScaffoldOptions>) -> Result<CallToolResult, ErrorData> {
        if let Err(errors) = validate_scaffold_options(&params) {
            return Err(ErrorData::invalid_params("Invalid scaffold options", Some(json!({ "errors": errors }))));
        }

        let scaffold = scaffold_project(&params);
        let file_structure = scaffold_to_json(scaffold);
        Ok(CallToolResult::structured(json!({
            "message": format!("Successfully scaffolded Payload CMS project: {}", params.project_name),
            "fileStructure": file_structure
        })))
    }

    #[tool(name = "connect_payload", description = "Connect to a live Payload CMS instance and test the connection")]
    async fn connect_payload(&self, Parameters(params): Parameters<ConnectPayloadParams>) -> Result<CallToolResult, ErrorData> {
        match create_payload_client(&params.connection_string, params.api_key) {
            Ok(client) => {
                match client.test_connection() {
                    Ok(info) => Ok(CallToolResult::structured(json!({
                        "success": true,
                        "server_info": info
                    }))),
                    Err(err) => Ok(CallToolResult::structured(json!({
                        "success": false,
                        "error": err.to_string()
                    })))
                }
            }
            Err(err) => Ok(CallToolResult::structured(json!({
                "success": false,
                "error": err.to_string()
            })))
        }
    }

    #[tool(name = "get_collection_schema", description = "Get collection schema from a live Payload CMS instance")]
    async fn get_collection_schema(&self, Parameters(params): Parameters<GetCollectionParams>) -> Result<CallToolResult, ErrorData> {
        match create_payload_client(&params.connection_string, params.api_key) {
            Ok(client) => {
                match client.get_collection(&params.slug) {
                    Ok(collection) => Ok(CallToolResult::structured(json!({
                        "success": true,
                        "collection": collection
                    }))),
                    Err(err) => Ok(CallToolResult::structured(json!({
                        "success": false,
                        "error": err.to_string()
                    })))
                }
            }
            Err(err) => Ok(CallToolResult::structured(json!({
                "success": false,
                "error": err.to_string()
            })))
        }
    }

    #[tool(name = "list_collections", description = "List all collections from a live Payload CMS instance")]
    async fn list_collections(&self, Parameters(params): Parameters<ListCollectionsParams>) -> Result<CallToolResult, ErrorData> {
        match create_payload_client(&params.connection_string, params.api_key) {
            Ok(client) => {
                match client.list_collections() {
                    Ok(collections) => Ok(CallToolResult::structured(json!({
                        "success": true,
                        "collections": collections
                    }))),
                    Err(err) => Ok(CallToolResult::structured(json!({
                        "success": false,
                        "error": err.to_string()
                    })))
                }
            }
            Err(err) => Ok(CallToolResult::structured(json!({
                "success": false,
                "error": err.to_string()
            })))
        }
    }

    #[tool(name = "validate_against_live", description = "Validate a collection configuration against a live Payload instance")]
    async fn validate_against_live(&self, Parameters(params): Parameters<ValidateAgainstLiveParams>) -> Result<CallToolResult, ErrorData> {
        match create_payload_client(&params.connection_string, params.api_key) {
            Ok(client) => {
                match client.validate_collection_config(&params.slug, &params.config) {
                    Ok(issues) => Ok(CallToolResult::structured(json!({
                        "success": true,
                        "issues": issues
                    }))),
                    Err(err) => Ok(CallToolResult::structured(json!({
                        "success": false,
                        "error": err.to_string()
                    })))
                }
            }
            Err(err) => Ok(CallToolResult::structured(json!({
                "success": false,
                "error": err.to_string()
            })))
        }
    }
}

#[tool_handler(router = self.tool_router)]
impl ServerHandler for ToolBoxHandler {
    fn ping(
        &self,
        _ctx: RequestContext<RoleServer>,
    ) -> impl std::future::Future<Output = Result<(), rmcp::ErrorData>> + Send {
        ready(Ok(()))
    }

    async fn list_resources(
        &self,
        _req: Option<ListResourcesRequest>,
        _ctx: RequestContext<RoleServer>,
    ) -> Result<rmcp::model::ListResourcesResult, rmcp::ErrorData> {
        use rmcp::model::{Annotated, RawResource};
        Ok(rmcp::model::ListResourcesResult {
            resources: vec![Annotated {
                raw: RawResource {
                    uri: "file://instructions".to_string(),
                    name: "MCP Server Instructions".to_string(),
                    title: Some("MCP Server Instructions".to_string()),
                    description: Some("Usage instructions for this MCP server crate".to_string()),
                    mime_type: Some("text/plain".to_string()),
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
                    Self::instructions().unwrap_or_default(),
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
}
