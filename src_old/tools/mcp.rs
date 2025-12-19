use schemars::JsonSchema;
use serde::Deserialize;
use serde_json::{json, Value};

use crate::tools::{
    generator::{generate_template, TemplateType},
    query::{get_validation_rules_with_examples, query_validation_rules},
    scaffolder::{
        scaffold_project, validate_scaffold_options, ScaffoldFile, ScaffoldFileStructure,
        ScaffoldOptions,
    },
    sql::execute_sql_query,
    types::FileType,
    validator::validate_payload_code,
};
use rmcp::model::{CallToolResult, Content, Tool};
use rmcp::ErrorData;

#[derive(Debug, Deserialize, JsonSchema)]
pub struct EchoParams {
    pub message: String,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct ValidateParams {
    pub code: String,
    pub file_type: FileType,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct QueryParams {
    pub query: String,
    pub file_type: Option<FileType>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct SqlParams {
    pub sql: String,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct GenerateTemplateParams {
    pub template_type: TemplateType,
    pub options: Value,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct GenerateCollectionParams {
    pub slug: String,
    pub fields: Option<Value>,
    pub auth: Option<bool>,
    pub timestamps: Option<bool>,
    pub admin: Option<Value>,
    pub hooks: Option<bool>,
    pub access: Option<bool>,
    pub versions: Option<bool>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct GenerateFieldParams {
    pub name: String,
    #[serde(rename = "type")]
    pub field_type: String,
    pub required: Option<bool>,
    pub unique: Option<bool>,
    pub localized: Option<bool>,
    pub access: Option<bool>,
    pub admin: Option<Value>,
    pub validation: Option<bool>,
    pub default_value: Option<Value>,
}

pub fn tool_definitions() -> Vec<Tool> {
    vec![
        Tool::new(
            "echo",
            "Echo a message back to the caller",
            rmcp::handler::server::tool::cached_schema_for_type::<EchoParams>(),
        ),
        Tool::new(
            "validate",
            "Validate Payload CMS code (collection, field, global, config)",
            rmcp::handler::server::tool::cached_schema_for_type::<ValidateParams>(),
        ),
        Tool::new(
            "query",
            "Query validation rules and best practices",
            rmcp::handler::server::tool::cached_schema_for_type::<QueryParams>(),
        ),
        Tool::new(
            "mcp_query",
            "Execute SQL-like queries against validation rules",
            rmcp::handler::server::tool::cached_schema_for_type::<SqlParams>(),
        ),
        Tool::new(
            "generate_template",
            "Generate Payload CMS code templates",
            rmcp::handler::server::tool::cached_schema_for_type::<GenerateTemplateParams>(),
        ),
        Tool::new(
            "generate_collection",
            "Generate a Payload CMS collection template",
            rmcp::handler::server::tool::cached_schema_for_type::<GenerateCollectionParams>(),
        ),
        Tool::new(
            "generate_field",
            "Generate a Payload CMS field template",
            rmcp::handler::server::tool::cached_schema_for_type::<GenerateFieldParams>(),
        ),
        Tool::new(
            "scaffold_project",
            "Scaffold a complete Payload CMS 3 project structure",
            rmcp::handler::server::tool::cached_schema_for_type::<ScaffoldOptions>(),
        ),
    ]
}

pub async fn run_tool(name: &str, args: Value) -> Result<CallToolResult, ErrorData> {
    match name {
        "echo" => {
            let params: EchoParams = serde_json::from_value(args)
                .map_err(|err| ErrorData::invalid_params(err.to_string(), None))?;
            Ok(CallToolResult::success(vec![Content::text(format!(
                "Tool echo: {}",
                params.message
            ))]))
        }
        "validate" => {
            let params: ValidateParams = serde_json::from_value(args)
                .map_err(|err| ErrorData::invalid_params(err.to_string(), None))?;
            let result = validate_payload_code(&params.code, params.file_type);
            Ok(CallToolResult::structured(json!(result)))
        }
        "query" => {
            let params: QueryParams = serde_json::from_value(args)
                .map_err(|err| ErrorData::invalid_params(err.to_string(), None))?;
            let rules = if params.query.trim().is_empty() {
                get_validation_rules_with_examples(None, params.file_type)
            } else {
                query_validation_rules(&params.query, params.file_type)
            };
            Ok(CallToolResult::structured(json!({ "rules": rules })))
        }
        "mcp_query" => {
            let params: SqlParams = serde_json::from_value(args)
                .map_err(|err| ErrorData::invalid_params(err.to_string(), None))?;
            match execute_sql_query(&params.sql) {
                Ok(results) => Ok(CallToolResult::structured(json!({ "results": results }))),
                Err(err) => Ok(CallToolResult::structured_error(json!({ "error": err }))),
            }
        }
        "generate_template" => {
            let params: GenerateTemplateParams = serde_json::from_value(args)
                .map_err(|err| ErrorData::invalid_params(err.to_string(), None))?;
            match generate_template(params.template_type, &params.options) {
                Ok(code) => Ok(CallToolResult::structured(json!({ "code": code }))),
                Err(err) => Ok(CallToolResult::structured_error(json!({ "error": err }))),
            }
        }
        "generate_collection" => {
            let params: GenerateCollectionParams = serde_json::from_value(args)
                .map_err(|err| ErrorData::invalid_params(err.to_string(), None))?;
            let mut options = serde_json::Map::new();
            options.insert("slug".into(), json!(params.slug));
            if let Some(fields) = params.fields {
                options.insert("fields".into(), fields);
            }
            if let Some(auth) = params.auth {
                options.insert("auth".into(), json!(auth));
            }
            if let Some(ts) = params.timestamps {
                options.insert("timestamps".into(), json!(ts));
            }
            if let Some(admin) = params.admin {
                options.insert("admin".into(), admin);
            }
            if let Some(hooks) = params.hooks {
                options.insert("hooks".into(), json!(hooks));
            }
            if let Some(access) = params.access {
                options.insert("access".into(), json!(access));
            }
            if let Some(versions) = params.versions {
                options.insert("versions".into(), json!(versions));
            }

            match generate_template(TemplateType::Collection, &Value::Object(options)) {
                Ok(code) => Ok(CallToolResult::structured(json!({ "code": code }))),
                Err(err) => Ok(CallToolResult::structured_error(json!({ "error": err }))),
            }
        }
        "generate_field" => {
            let params: GenerateFieldParams = serde_json::from_value(args)
                .map_err(|err| ErrorData::invalid_params(err.to_string(), None))?;
            let mut options = serde_json::Map::new();
            options.insert("name".into(), json!(params.name));
            options.insert("type".into(), json!(params.field_type));
            if let Some(required) = params.required {
                options.insert("required".into(), json!(required));
            }
            if let Some(unique) = params.unique {
                options.insert("unique".into(), json!(unique));
            }
            if let Some(localized) = params.localized {
                options.insert("localized".into(), json!(localized));
            }
            if let Some(access) = params.access {
                options.insert("access".into(), json!(access));
            }
            if let Some(admin) = params.admin {
                options.insert("admin".into(), admin);
            }
            if let Some(validation) = params.validation {
                options.insert("validation".into(), json!(validation));
            }
            if let Some(default_value) = params.default_value {
                options.insert("defaultValue".into(), default_value);
            }

            match generate_template(TemplateType::Field, &Value::Object(options)) {
                Ok(code) => Ok(CallToolResult::structured(json!({ "code": code }))),
                Err(err) => Ok(CallToolResult::structured_error(json!({ "error": err }))),
            }
        }
        "scaffold_project" => {
            let params: ScaffoldOptions = serde_json::from_value(args)
                .map_err(|err| ErrorData::invalid_params(err.to_string(), None))?;

            if let Err(errors) = validate_scaffold_options(&params) {
                return Ok(CallToolResult::structured_error(json!({ "errors": errors })));
            }

            let scaffold = scaffold_project(&params);
            let file_structure = scaffold_to_json(scaffold);
            Ok(CallToolResult::structured(json!({
                "message": format!("Successfully scaffolded Payload CMS project: {}", params.project_name),
                "fileStructure": file_structure
            })))
        }
        _ => Err(ErrorData::invalid_params(
            format!("Unknown tool: {name}"),
            None,
        )),
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
