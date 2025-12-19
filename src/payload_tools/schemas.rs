use serde_json::{Map, Value};

pub const FIELD_TYPES: &[&str] = &[
    "text",
    "textarea",
    "email",
    "code",
    "number",
    "date",
    "checkbox",
    "select",
    "relationship",
    "upload",
    "array",
    "blocks",
    "group",
    "row",
    "collapsible",
    "tabs",
    "richText",
    "json",
    "radio",
    "point",
];

fn expect_object<'a>(value: &'a Value, context: &str) -> Result<&'a Map<String, Value>, String> {
    value
        .as_object()
        .ok_or_else(|| format!("{context} must be an object"))
}

fn require_string(map: &Map<String, Value>, key: &str) -> Result<String, String> {
    map.get(key)
        .and_then(|v| v.as_str())
        .filter(|s| !s.is_empty())
        .map(|s| s.to_string())
        .ok_or_else(|| format!("Missing or invalid string property '{key}'"))
}

fn validate_fields_array(fields: &[Value]) -> Result<(), String> {
    for (index, field) in fields.iter().enumerate() {
        validate_field_schema(field)
            .map_err(|err| format!("Field at index {index} failed validation: {err}"))?;
    }
    Ok(())
}

pub fn validate_field_schema(value: &Value) -> Result<(), String> {
    let map = expect_object(value, "Field")?;

    require_string(map, "name")?;
    let field_type = require_string(map, "type")?;

    if !FIELD_TYPES.contains(&field_type.as_str()) {
        return Err(format!(
            "Unsupported field type '{field_type}'. Supported types: {}",
            FIELD_TYPES.join(", ")
        ));
    }

    if let Some(admin) = map.get("admin") {
        expect_object(admin, "Field.admin")?;
    }

    if let Some(access) = map.get("access") {
        expect_object(access, "Field.access")?;
    }

    match field_type.as_str() {
        "select" => {
            if let Some(options) = map.get("options") {
                let opts = options
                    .as_array()
                    .ok_or_else(|| "Field.options must be an array".to_string())?;
                if opts.is_empty() {
                    return Err("Field.options must include at least one option".to_string());
                }
            }
        }
        "relationship" => {
            if let Some(relation_to) = map.get("relationTo") {
                if !(relation_to.is_string() || relation_to.is_array()) {
                    return Err("Field.relationTo must be a string or array".to_string());
                }
            }
        }
        "array" | "group" | "tabs" => {
            if let Some(fields) = map.get("fields") {
                let arr = fields
                    .as_array()
                    .ok_or_else(|| "Field.fields must be an array".to_string())?;
                validate_fields_array(arr)?;
            }
        }
        _ => {}
    }

    Ok(())
}

pub fn validate_collection_schema(value: &Value) -> Result<(), String> {
    let map = expect_object(value, "Collection")?;
    require_string(map, "slug")?;

    let fields = map
        .get("fields")
        .and_then(|v| v.as_array())
        .ok_or_else(|| "Collection must include a 'fields' array".to_string())?;

    if fields.is_empty() {
        return Err("Collection.fields must contain at least one field".to_string());
    }

    validate_fields_array(fields)?;

    if let Some(admin) = map.get("admin") {
        expect_object(admin, "Collection.admin")?;
    }

    if let Some(access) = map.get("access") {
        expect_object(access, "Collection.access")?;
    }

    Ok(())
}

pub fn validate_global_schema(value: &Value) -> Result<(), String> {
    let map = expect_object(value, "Global")?;
    require_string(map, "slug")?;

    let fields = map
        .get("fields")
        .and_then(|v| v.as_array())
        .ok_or_else(|| "Global must include a 'fields' array".to_string())?;

    validate_fields_array(fields)?;

    if let Some(access) = map.get("access") {
        expect_object(access, "Global.access")?;
    }

    Ok(())
}

pub fn validate_config_schema(value: &Value) -> Result<(), String> {
    let map = expect_object(value, "Config")?;

    if let Some(collections) = map.get("collections") {
        let array = collections
            .as_array()
            .ok_or_else(|| "Config.collections must be an array".to_string())?;
        for (index, collection) in array.iter().enumerate() {
            validate_collection_schema(collection)
                .map_err(|err| format!("collections[{index}]: {err}"))?;
        }
    }

    if let Some(globals) = map.get("globals") {
        let array = globals
            .as_array()
            .ok_or_else(|| "Config.globals must be an array".to_string())?;
        for (index, global) in array.iter().enumerate() {
            validate_global_schema(global).map_err(|err| format!("globals[{index}]: {err}"))?;
        }
    }

    if let Some(admin) = map.get("admin") {
        expect_object(admin, "Config.admin")?;
    }

    if let Some(plugins) = map.get("plugins") {
        if !plugins.is_array() {
            return Err("Config.plugins must be an array".to_string());
        }
    }

    Ok(())
}
