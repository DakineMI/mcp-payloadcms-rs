use regex::Regex;
use serde_json::{Value, json};
use std::collections::HashMap;
use std::fs;
use std::io::Write;
use std::path::PathBuf;
use tempfile::tempdir;

#[derive(Clone, Debug)]
pub enum FileType {
    Collection,
    Field,
    Global,
    Config,
    Unknown,
}

impl From<&str> for FileType {
    fn from(s: &str) -> Self {
        match s {
            "collection" => FileType::Collection,
            "field" => FileType::Field,
            "global" => FileType::Global,
            "config" => FileType::Config,
            _ => FileType::Unknown,
        }
    }
}

#[derive(Clone, Debug, serde::Serialize)]
pub struct ValidationResult {
    pub is_valid: bool,
    pub errors: Vec<String>,
}

/// Validate Payload-like code with heuristics (no JS runtime).
pub fn validate_payload_code(code: &str, _file_type: FileType) -> ValidationResult {
    let mut errors = Vec::new();

    if code.trim().is_empty() {
        errors.push("Code is empty".to_string());
    }

    // Balanced brackets check
    let mut stack: Vec<char> = Vec::new();
    for ch in code.chars() {
        match ch {
            '{' | '(' | '[' => stack.push(ch),
            '}' => {
                if stack.pop() != Some('{') {
                    errors.push("Unbalanced '}'".to_string());
                    break;
                }
            }
            ')' => {
                if stack.pop() != Some('(') {
                    errors.push("Unbalanced ')'".to_string());
                    break;
                }
            }
            ']' => {
                if stack.pop() != Some('[') {
                    errors.push("Unbalanced ']'".to_string());
                    break;
                }
            }
            _ => {}
        }
    }
    if !stack.is_empty() {
        errors.push("Unbalanced opening bracket(s)".to_string());
    }

    // Dangerous patterns
    if code.contains("eval(") || code.contains("new Function(") {
        errors.push(
            "Use of `eval` or `new Function` detected — avoid dynamic code execution".to_string(),
        );
    }

    // Trailing comma heuristic
    if Regex::new(r",\s*[}\]]").unwrap().is_match(code) {
        errors.push(
            "Found trailing commas near an object/array end — check syntax for target environment"
                .to_string(),
        );
    }

    ValidationResult {
        is_valid: errors.is_empty(),
        errors,
    }
}

/// Infer simple validation rules from a free-text query string.
pub fn query_validation_rules(query: &str, _file_type: Option<FileType>) -> Vec<String> {
    let mut rules = Vec::new();
    let q = query.to_lowercase();
    if q.contains("required") || q.contains("is required") {
        rules.push("required: true".to_string());
    }
    if q.contains("unique") {
        rules.push("unique: true".to_string());
    }
    if q.contains("email") {
        rules.push("validation: { pattern: /\\S+@\\S+\\.\\S+/ }".to_string());
    }
    if q.contains("max length") || q.contains("maxlength") {
        rules.push("validation: { maxLength: <number> }".to_string());
    }
    if rules.is_empty() {
        rules.push("no specific rules inferred; check the query text for requirements".to_string());
    }
    rules
}

/// Execute a safe, local 'SQL-like' query. Returns an echo-style result (no DB access).
pub fn execute_sql_query(sql: &str) -> Result<Value, String> {
    let lower = sql.to_lowercase();
    if lower.contains("drop ")
        || lower.contains("delete ")
        || lower.contains("insert ")
        || lower.contains("update ")
    {
        return Err("Modifying queries are not supported in local-only mode".to_string());
    }
    Ok(json!({
        "query": sql,
        "rows": [],
        "notes": "This is a local-only emulation of query execution (no DB)."
    }))
}

/// Generate code templates for various template types.
pub fn generate_template(template_type: &str, options: &Value) -> Result<String, String> {
    let opts = options.as_object().cloned().unwrap_or_default();
    match template_type {
        "collection" => {
            let slug = opts
                .get("slug")
                .and_then(|v| v.as_str())
                .unwrap_or("my_collection");
            let title_field = opts
                .get("titleField")
                .and_then(|v| v.as_str())
                .unwrap_or("title");
            let code = format!(
                "module.exports = {{\n  slug: '{}',\n  fields: [\n    {{ name: '{}', type: 'text' }}\n  ]\n}};\n",
                slug, title_field
            );
            Ok(code)
        }
        "field" => {
            let name = opts
                .get("name")
                .and_then(|v| v.as_str())
                .unwrap_or("myField");
            let ftype = opts.get("type").and_then(|v| v.as_str()).unwrap_or("text");
            let code = format!("{{ name: '{}', type: '{}' }}\n", name, ftype);
            Ok(code)
        }
        "global" => Ok("// global definition stub\nmodule.exports = {};\n".to_string()),
        "config" => Ok("// config stub\nmodule.exports = {};\n".to_string()),
        "hook" => {
            Ok("// hook stub\nmodule.exports = (req, res, next) => { next(); };\n".to_string())
        }
        _ => Err(format!("Unsupported template type: {}", template_type)),
    }
}

/// Validate scaffold options minimally (projectName required)
pub fn validate_scaffold_options(options: &Value) -> (bool, Vec<String>) {
    let mut errors = Vec::new();
    if let Some(name) = options.get("projectName").and_then(|v| v.as_str()) {
        if name.trim().is_empty() {
            errors.push("projectName cannot be empty".to_string());
        }
    } else {
        errors.push("projectName is required".to_string());
    }
    (errors.is_empty(), errors)
}

/// Scaffold a minimal project on disk. Returns JSON describing created files.
pub fn scaffold_project(options: &Value) -> Result<Value, String> {
    let (ok, errs) = validate_scaffold_options(options);
    if !ok {
        return Err(format!("Invalid scaffold options: {:?}", errs));
    }

    let project_name = options.get("projectName").and_then(|v| v.as_str()).unwrap();
    let output_dir_opt = options.get("outputDir").and_then(|v| v.as_str());

    let base_dir = if let Some(p) = output_dir_opt {
        PathBuf::from(p).join(project_name)
    } else {
        let tmp = tempdir().map_err(|e| format!("tempdir error: {}", e))?;
        tmp.path().join(project_name)
    };

    fs::create_dir_all(base_dir.join("src")).map_err(|e| format!("mkdir error: {}", e))?;
    let mut created = HashMap::<String, String>::new();

    let readme_path = base_dir.join("README.md");
    let mut readme = fs::File::create(&readme_path).map_err(|e| format!("create file: {}", e))?;
    let readme_contents = format!("# {}\n\nScaffolded by payloadcmsmcp-rs\n", project_name);
    readme
        .write_all(readme_contents.as_bytes())
        .map_err(|e| format!("write file: {}", e))?;
    created.insert(
        "README.md".to_string(),
        readme_path.to_string_lossy().to_string(),
    );

    let index_path = base_dir.join("src").join("index.js");
    let mut index = fs::File::create(&index_path).map_err(|e| format!("create file: {}", e))?;
    let index_contents = "// entry point\nconsole.log('Hello from scaffolded project');\n";
    index
        .write_all(index_contents.as_bytes())
        .map_err(|e| format!("write file: {}", e))?;
    created.insert(
        "src/index.js".to_string(),
        index_path.to_string_lossy().to_string(),
    );

    let file_structure = json!(created);
    Ok(json!({
        "projectName": project_name,
        "basePath": base_dir.to_string_lossy().to_string(),
        "fileStructure": file_structure
    }))
}
