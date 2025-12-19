use serde_json::Value;

use crate::payload_tools::schemas::{
    validate_collection_schema, validate_config_schema, validate_field_schema, validate_global_schema,
};
use crate::payload_tools::types::{
    Examples, FileType, Reference, Suggestion, ValidationResult, ValidationRule,
};

fn parse_payload_object(code: &str) -> Result<Value, String> {
    serde_json::from_str(code.trim()).map_err(|err| format!("Failed to parse code as JSON: {err}"))
}

fn naming_conventions(name: &str) -> Vec<String> {
    let mut errors: Vec<String> = Vec::new();
    if name.contains(' ') {
        errors.push(format!(
            "Name \"{name}\" should not contain spaces. Use camelCase or snake_case instead."
        ));
    }
    if name.chars().any(|c| c.is_uppercase()) && name.contains('_') {
        errors.push(format!(
            "Name \"{name}\" mixes camelCase and snake_case. Choose one convention."
        ));
    }
    errors
}

fn reserved_words(name: &str) -> Vec<String> {
    let reserved = [
        "constructor",
        "prototype",
        "__proto__",
        "toString",
        "toJSON",
        "valueOf",
    ];
    if reserved.contains(&name) {
        vec![format!(
            "Name \"{name}\" is a reserved JavaScript word and should be avoided."
        )]
    } else {
        Vec::new()
    }
}

fn collection_reference() -> Reference {
    Reference {
        title: "Payload CMS Collections Documentation".to_string(),
        url: "https://payloadcms.com/docs/configuration/collections".to_string(),
    }
}

fn field_reference() -> Reference {
    Reference {
        title: "Payload CMS Fields Documentation".to_string(),
        url: "https://payloadcms.com/docs/fields/overview".to_string(),
    }
}

fn global_reference() -> Reference {
    Reference {
        title: "Payload CMS Globals Documentation".to_string(),
        url: "https://payloadcms.com/docs/configuration/globals".to_string(),
    }
}

fn config_reference() -> Reference {
    Reference {
        title: "Payload CMS Configuration Documentation".to_string(),
        url: "https://payloadcms.com/docs/configuration/overview".to_string(),
    }
}

pub fn validation_rules() -> Vec<ValidationRule> {
    vec![
        ValidationRule {
            id: "naming-conventions".to_string(),
            name: "Naming Conventions".to_string(),
            description: "Names should follow consistent conventions (camelCase or snake_case)"
                .to_string(),
            category: "best-practices".to_string(),
            file_types: vec![
                FileType::Collection,
                FileType::Field,
                FileType::Global,
                FileType::Config,
            ],
            examples: Examples {
                valid: vec!["myField".to_string(), "my_field".to_string()],
                invalid: vec![
                    "my field".to_string(),
                    "my-field".to_string(),
                    "my_Field".to_string(),
                ],
            },
        },
        ValidationRule {
            id: "reserved-words".to_string(),
            name: "Reserved Words".to_string(),
            description: "Avoid using JavaScript reserved words for names".to_string(),
            category: "best-practices".to_string(),
            file_types: vec![
                FileType::Collection,
                FileType::Field,
                FileType::Global,
                FileType::Config,
            ],
            examples: Examples {
                valid: vec![
                    "title".to_string(),
                    "content".to_string(),
                    "author".to_string(),
                ],
                invalid: vec![
                    "constructor".to_string(),
                    "prototype".to_string(),
                    "__proto__".to_string(),
                ],
            },
        },
        ValidationRule {
            id: "access-control".to_string(),
            name: "Access Control".to_string(),
            description: "Define access control for collections and fields".to_string(),
            category: "security".to_string(),
            file_types: vec![FileType::Collection, FileType::Field, FileType::Global],
            examples: Examples {
                valid: vec!["access: { read: () => true, update: () => true }".to_string()],
                invalid: vec!["// No access control defined".to_string()],
            },
        },
        ValidationRule {
            id: "sensitive-fields".to_string(),
            name: "Sensitive Fields Protection".to_string(),
            description: "Sensitive fields should have explicit read access control".to_string(),
            category: "security".to_string(),
            file_types: vec![FileType::Field],
            examples: Examples {
                valid: vec![
                    r#"{ name: "password", type: "text", access: { read: () => false } }"#.into()
                ],
                invalid: vec![r#"{ name: "password", type: "text" }"#.into()],
            },
        },
        ValidationRule {
            id: "indexed-fields".to_string(),
            name: "Indexed Fields".to_string(),
            description: "Fields used for searching or filtering should be indexed".to_string(),
            category: "performance".to_string(),
            file_types: vec![FileType::Field],
            examples: Examples {
                valid: vec![r#"{ name: "email", type: "email", index: true }"#.into()],
                invalid: vec![r#"{ name: "email", type: "email" }"#.into()],
            },
        },
        ValidationRule {
            id: "relationship-depth".to_string(),
            name: "Relationship Depth".to_string(),
            description: "Relationship fields should have a maxDepth to prevent deep queries"
                .to_string(),
            category: "performance".to_string(),
            file_types: vec![FileType::Field],
            examples: Examples {
                valid: vec![r#"{ type: "relationship", relationTo: "posts", maxDepth: 1 }"#.into()],
                invalid: vec![r#"{ type: "relationship", relationTo: "posts" }"#.into()],
            },
        },
        ValidationRule {
            id: "field-validation".to_string(),
            name: "Field Validation".to_string(),
            description: "Required fields should have validation".to_string(),
            category: "data-integrity".to_string(),
            file_types: vec![FileType::Field],
            examples: Examples {
                valid: vec![
                    r#"{ name: "title", type: "text", required: true, validate: (value) => value ? true : "Required" }"#.into()
                ],
                invalid: vec![r#"{ name: "title", type: "text", required: true }"#.into()],
            },
        },
        ValidationRule {
            id: "timestamps".to_string(),
            name: "Timestamps".to_string(),
            description: "Collections should have timestamps enabled".to_string(),
            category: "best-practices".to_string(),
            file_types: vec![FileType::Collection],
            examples: Examples {
                valid: vec![r#"{ slug: "posts", timestamps: true }"#.into()],
                invalid: vec![r#"{ slug: "posts" }"#.into()],
            },
        },
        ValidationRule {
            id: "admin-ui".to_string(),
            name: "Admin UI Configuration".to_string(),
            description: "Collections should specify which field to use as title in admin UI"
                .to_string(),
            category: "usability".to_string(),
            file_types: vec![FileType::Collection],
            examples: Examples {
                valid: vec![r#"{ admin: { useAsTitle: "title" } }"#.into()],
                invalid: vec![r#"{ admin: {} }"#.into()],
            },
        },
    ]
}

pub fn validate_collection(code: &str) -> ValidationResult {
    let references = vec![collection_reference()];
    let value = match parse_payload_object(code) {
        Ok(value) => value,
        Err(err) => {
            return ValidationResult {
                is_valid: false,
                errors: vec![err],
                warnings: Vec::new(),
                suggestions: Vec::new(),
                references,
            };
        }
    };

    if let Err(err) = validate_collection_schema(&value) {
        return ValidationResult {
            is_valid: false,
            errors: vec![err],
            warnings: Vec::new(),
            suggestions: Vec::new(),
            references,
        };
    }

    let mut errors: Vec<String> = Vec::new();
    let mut warnings = Vec::new();
    let mut suggestions = Vec::new();

    if let Some(slug) = value.get("slug").and_then(|v| v.as_str()) {
        errors.extend(naming_conventions(slug));
        errors.extend(reserved_words(slug));
    }

    if let Some(fields) = value.get("fields").and_then(|v| v.as_array()) {
        for field in fields {
            if let Some(name) = field.get("name").and_then(|v| v.as_str()) {
                errors.extend(naming_conventions(name));
                errors.extend(reserved_words(name));
            }

            let field_name = field
                .get("name")
                .and_then(|v| v.as_str())
                .unwrap_or_default()
                .to_ascii_lowercase();
            if (field_name.contains("password")
                || field_name.contains("token")
                || field_name.contains("secret"))
                && field
                    .get("access")
                    .and_then(|a| a.get("read"))
                    .is_none()
            {
                warnings.push(format!(
                    "Sensitive field \"{}\" should have explicit read access control.",
                    field_name
                ));
            }

            let field_type = field.get("type").and_then(|v| v.as_str()).unwrap_or("");
            if matches!(field_type, "text" | "email" | "textarea") {
                if field.get("unique").and_then(|v| v.as_bool()).unwrap_or(false)
                    && !field.get("index").and_then(|v| v.as_bool()).unwrap_or(false)
                {
                    warnings.push(format!(
                        "Field \"{}\" is unique but not indexed. Consider adding 'index: true' for better performance.",
                        field.get("name").and_then(|v| v.as_str()).unwrap_or("field")
                    ));
                }
            }
        }
    }

    if value.get("access").is_none() {
        warnings.push(
            "No access control defined. This might expose data to unauthorized users.".to_string(),
        );
    }

    if value
        .get("admin")
        .and_then(|a| a.get("useAsTitle"))
        .is_none()
    {
        suggestions.push(Suggestion {
            message:
                "Consider adding 'useAsTitle' to specify which field to use as the title in the admin UI."
                    .to_string(),
            code: Some("admin: { useAsTitle: 'title' }".to_string()),
        });
    }

    if !value
        .get("timestamps")
        .and_then(|v| v.as_bool())
        .unwrap_or(false)
    {
        suggestions.push(Suggestion {
            message: "Consider enabling timestamps to automatically track creation and update times."
                .to_string(),
            code: Some("timestamps: true".to_string()),
        });
    }

    ValidationResult {
        is_valid: errors.is_empty(),
        errors,
        warnings,
        suggestions,
        references,
    }
}

pub fn validate_field(code: &str) -> ValidationResult {
    let references = vec![field_reference()];
    let value = match parse_payload_object(code) {
        Ok(value) => value,
        Err(err) => {
            return ValidationResult {
                is_valid: false,
                errors: vec![err],
                warnings: Vec::new(),
                suggestions: Vec::new(),
                references,
            };
        }
    };

    if let Err(err) = validate_field_schema(&value) {
        return ValidationResult {
            is_valid: false,
            errors: vec![err],
            warnings: Vec::new(),
            suggestions: Vec::new(),
            references,
        };
    }

    let mut errors: Vec<String> = Vec::new();
    let mut warnings = Vec::new();
    let mut suggestions = Vec::new();

    if let Some(name) = value.get("name").and_then(|v| v.as_str()) {
        errors.extend(naming_conventions(name));
        errors.extend(reserved_words(name));
    }

    let field_type = value
        .get("type")
        .and_then(|v| v.as_str())
        .unwrap_or_default();
    if field_type == "relationship" && value.get("maxDepth").is_none() {
        warnings.push(
            "Relationship field without maxDepth could lead to deep queries. Consider adding a maxDepth limit."
                .to_string(),
        );
        suggestions.push(Suggestion {
            message: "Add maxDepth to limit relationship depth".to_string(),
            code: Some("maxDepth: 1".to_string()),
        });
    }

    if field_type == "text"
        && value.get("required").and_then(|v| v.as_bool()).unwrap_or(false)
        && value.get("validate").is_none()
    {
        suggestions.push(Suggestion {
            message: "Consider adding validation for required text fields".to_string(),
            code: Some(
                "validate: (value) => {\n  if (!value || value.trim() === '') {\n    return 'This field is required';\n  }\n  return true;\n}"
                    .to_string(),
            ),
        });
    }

    ValidationResult {
        is_valid: errors.is_empty(),
        errors,
        warnings,
        suggestions,
        references,
    }
}

pub fn validate_global(code: &str) -> ValidationResult {
    let references = vec![global_reference()];
    let value = match parse_payload_object(code) {
        Ok(value) => value,
        Err(err) => {
            return ValidationResult {
                is_valid: false,
                errors: vec![err],
                warnings: Vec::new(),
                suggestions: Vec::new(),
                references,
            };
        }
    };

    if let Err(err) = validate_global_schema(&value) {
        return ValidationResult {
            is_valid: false,
            errors: vec![err],
            warnings: Vec::new(),
            suggestions: Vec::new(),
            references,
        };
    }

    let mut errors = Vec::new();
    let mut warnings = Vec::new();

    if let Some(slug) = value.get("slug").and_then(|v| v.as_str()) {
        errors.extend(naming_conventions(slug));
        errors.extend(reserved_words(slug));
    }

    if let Some(fields) = value.get("fields").and_then(|v| v.as_array()) {
        for field in fields {
            if let Some(name) = field.get("name").and_then(|v| v.as_str()) {
                errors.extend(naming_conventions(name));
                errors.extend(reserved_words(name));
            }
        }
    }

    if value.get("access").is_none() {
        warnings.push(
            "No access control defined. This might expose data to unauthorized users.".to_string(),
        );
    }

    ValidationResult {
        is_valid: errors.is_empty(),
        errors,
        warnings,
        suggestions: Vec::new(),
        references,
    }
}

pub fn validate_config(code: &str) -> ValidationResult {
    let references = vec![config_reference()];
    let value = match parse_payload_object(code) {
        Ok(value) => value,
        Err(err) => {
            return ValidationResult {
                is_valid: false,
                errors: vec![err],
                warnings: Vec::new(),
                suggestions: Vec::new(),
                references,
            };
        }
    };

    if let Err(err) = validate_config_schema(&value) {
        return ValidationResult {
            is_valid: false,
            errors: vec![err],
            warnings: Vec::new(),
            suggestions: Vec::new(),
            references,
        };
    }

    let errors: Vec<String> = Vec::new();
    let mut warnings = Vec::new();
    let mut suggestions = Vec::new();

    if value.get("serverURL").is_none() {
        warnings.push("Missing serverURL in config. This is required for proper URL generation."
            .to_string());
        suggestions.push(Suggestion {
            message: "Add serverURL to your config".to_string(),
            code: Some("serverURL: 'http://localhost:3000'".to_string()),
        });
    }

    if value.get("admin").is_none() {
        suggestions.push(Suggestion {
            message: "Consider configuring the admin panel".to_string(),
            code: Some(
                "admin: {\n  user: 'users',\n  meta: {\n    titleSuffix: '- My Payload App',\n    favicon: '/favicon.ico',\n  }\n}"
                    .to_string(),
            ),
        });
    }

    ValidationResult {
        is_valid: errors.is_empty(),
        errors,
        warnings,
        suggestions,
        references,
    }
}

pub fn validate_payload_code(code: &str, file_type: FileType) -> ValidationResult {
    match file_type {
        FileType::Collection => validate_collection(code),
        FileType::Field => validate_field(code),
        FileType::Global => validate_global(code),
        FileType::Config => validate_config(code),
    }
}
